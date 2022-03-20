// Copyright (C) 2019-2020 Alibaba Cloud. All rights reserved.
// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use std::ops::Deref;

use log::error;
use virtio_bindings::bindings::virtio_blk::*;
use virtio_queue::{Descriptor, DescriptorChain};
use vm_memory::{ByteValued, Bytes, GuestAddress, GuestMemory};

use crate::{Error, Result};

/// Type of request from driver to device.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RequestType {
    /// Read request.
    In,
    /// Write request.
    Out,
    /// Flush request.
    Flush,
    /// Get device ID request.
    GetDeviceID,
    /// Unsupported request.
    Unsupported(u32),
}

impl From<u32> for RequestType {
    fn from(value: u32) -> Self {
        match value {
            VIRTIO_BLK_T_IN => RequestType::In,
            VIRTIO_BLK_T_OUT => RequestType::Out,
            VIRTIO_BLK_T_FLUSH => RequestType::Flush,
            VIRTIO_BLK_T_GET_ID => RequestType::GetDeviceID,
            t => RequestType::Unsupported(t),
        }
    }
}

/// The request header represents the mandatory fields of each block device request.
///
/// A request header contains the following fields:
///   * request_type: an u32 value mapping to a read, write or flush operation.
///   * reserved: 32 bits are reserved for future extensions of the Virtio Spec.
///   * sector: an u64 value representing the offset where a read/write is to occur.
///
/// The header simplifies reading the request from memory as all request follow
/// the same memory layout.
#[derive(Copy, Clone, Default)]
#[repr(C)]
struct RequestHeader {
    request_type: u32,
    _reserved: u32,
    sector: u64,
}

// Safe because RequestHeader only contains plain data.
unsafe impl ByteValued for RequestHeader {}

impl RequestHeader {
    /// Reads the request header from GuestMemory starting at `addr`.
    ///
    /// Virtio 1.0 specifies that the data is transmitted by the driver in little-endian
    /// format. Firecracker currently runs only on little endian platforms so we don't
    /// need to do an explicit little endian read as all reads are little endian by default.
    /// When running on a big endian platform, this code should not compile, and support
    /// for explicit little endian reads is required.
    #[cfg(target_endian = "little")]
    fn read_from<M: GuestMemory + ?Sized>(memory: &M, addr: GuestAddress) -> Result<Self> {
        memory.read_obj(addr).map_err(Error::GuestMemory)
    }
}

/// IO Data descriptor.
#[derive(Clone, Debug)]
#[repr(C)]
pub struct IoDataDesc {
    pub data_addr: u64,
    pub data_len: usize,
}

/// The block request.
#[derive(Clone, Debug)]
pub struct Request {
    /// The type of the request.
    request_type: RequestType,
    /// The offset of the request.
    sector: u64,
    status_addr: GuestAddress,
    request_index: u16,
}

impl Request {
    /// Parses a `desc_chain` and returns the associated `Request`.
    pub(crate) fn parse<M>(
        desc_chain: &mut DescriptorChain<M>,
        data_descs: &mut Vec<IoDataDesc>,
        max_size: u32,
    ) -> Result<Self>
    where
        M: Deref,
        M::Target: GuestMemory,
    {
        let desc = desc_chain.next().ok_or(Error::DescriptorChainTooShort)?;
        // The head contains the request type which MUST be readable.
        if desc.is_write_only() {
            return Err(Error::UnexpectedWriteOnlyDescriptor);
        }

        let request_header = RequestHeader::read_from(desc_chain.memory(), desc.addr())?;
        let mut req = Request {
            request_type: RequestType::from(request_header.request_type),
            sector: request_header.sector,
            status_addr: GuestAddress(0),
            request_index: desc_chain.head_index(),
        };
        let status_desc;
        let mut desc = desc_chain
            .next()
            .ok_or(Error::DescriptorChainTooShort)
            .map_err(|e| {
                error!("virtio-blk: Request {:?} has only head descriptor", req);
                e
            })?;
        if !desc.has_next() {
            status_desc = desc;
            // Only flush requests are allowed to skip the data descriptor.
            if req.request_type != RequestType::Flush {
                error!("virtio-blk: Request {:?} need a data descriptor", req);
                return Err(Error::DescriptorChainTooShort);
            }
        } else {
            while desc.has_next() {
                req.check_request(desc, max_size)?;
                data_descs.push(IoDataDesc {
                    data_addr: desc.addr().0,
                    data_len: desc.len() as usize,
                });
                desc = desc_chain
                    .next()
                    .ok_or(Error::DescriptorChainTooShort)
                    .map_err(|e| {
                        error!("virtio-blk: descriptor chain corrupted");
                        e
                    })?;
            }
            status_desc = desc;
        }

        // The status MUST always be writable and the guest address must be accessible.
        if !status_desc.is_write_only() {
            return Err(Error::UnexpectedReadOnlyDescriptor);
        }
        if status_desc.len() < 1 {
            return Err(Error::DescriptorLengthTooSmall);
        }
        if !desc_chain.memory().address_in_range(status_desc.addr()) {
            return Err(Error::InvalidGuestAddress(status_desc.addr()));
        }
        req.status_addr = status_desc.addr();

        Ok(req)
    }

    fn check_request(&self, desc: Descriptor, max_size: u32) -> Result<()> {
        match self.request_type {
            RequestType::Out => {
                if desc.is_write_only() {
                    error!(
                        "virtio-blk: Request {:?} sees unexpected write-only descriptor",
                        self
                    );
                    return Err(Error::UnexpectedWriteOnlyDescriptor);
                } else if desc.len() > max_size {
                    error!(
                        "virtio-blk: Request {:?} size is greater than disk size ({} > {})",
                        self,
                        desc.len(),
                        max_size
                    );
                    return Err(Error::DescriptorLengthTooBig);
                }
            }
            RequestType::In => {
                if !desc.is_write_only() {
                    error!(
                        "virtio-blk: Request {:?} sees unexpected read-only descriptor for read",
                        self
                    );
                    return Err(Error::UnexpectedReadOnlyDescriptor);
                } else if desc.len() > max_size {
                    error!(
                        "virtio-blk: Request {:?} size is greater than disk size ({} > {})",
                        self,
                        desc.len(),
                        max_size
                    );
                    return Err(Error::DescriptorLengthTooBig);
                }
            }
            RequestType::GetDeviceID if !desc.is_write_only() => {
                error!(
                    "virtio-blk: Request {:?} sees unexpected read-only descriptor for GetDeviceID",
                    self
                );
                return Err(Error::UnexpectedReadOnlyDescriptor);
            }
            _ => {}
        }
        Ok(())
    }

    fn update_status<M: GuestMemory + ?Sized>(&self, mem: &M, status: u32) {
        // Safe to unwrap because we have validated request.status_addr in parse()
        mem.write_obj(status as u8, self.status_addr).unwrap();
    }

    // Return total IO length of all segments. Assume the req has been checked and is valid.
    fn data_len(&self, data_descs: &[IoDataDesc]) -> u32 {
        let mut len = 0;
        for d in data_descs {
            len += d.data_len;
        }
        len as u32
    }
}
