// Copyright (C) 2019 Alibaba Cloud. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use std::fs::File;
use std::io::{self, Read, Seek, Write};
use std::os::unix::io::RawFd;

use super::request::IoDataDesc;

/// Traits for the virtio-blk driver to access backend storage devices, such as localfile.
pub trait Ufile: Read + Write + Seek + Send {
    /// Get disk capacity in bytes.
    fn get_capacity(&self) -> u64;

    /// Get max size in a segment.
    fn get_max_size(&self) -> u32;

    /// Generate a unique device id for the virtio-blk device.
    fn get_device_id(&self) -> io::Result<String>;

    /// Update the backend storage devices. Currently only supported by localfile.
    fn update_disk_image(&mut self, _file: File) -> io::Result<()> {
        // return ENOSYS by default
        Err(io::Error::from_raw_os_error(38))
    }

    /// Some IO context can't be kept in the new process (e.g. TDC).
    /// So we must destroy it before exiting.
    fn destroy_io_context(&mut self) -> io::Result<()> {
        Ok(())
    }

    /// Get the raw event fd for data plane.
    fn get_data_evt_fd(&self) -> RawFd;

    /// Reset the raw event fd for data plane to invalid.
    fn reset_data_evt_fd(&mut self) {}

    /// Get the raw event fd for the optional control plain.
    /// Typically the control plain raw_fd is used to recover broken connection for the backend.
    fn get_control_evt_fd(&self) -> Option<RawFd> {
        None
    }

    /// Reset the raw event fd for the optional control plain to invalid.
    fn reset_control_evt_fd(&mut self) {}

    /// Submit asynchronous IO requests.
    fn io_submit(
        &mut self,
        opcode: u32,
        offset: u64,
        iovecs: &mut Vec<IoDataDesc>,
        aio_data: u16,
    ) -> io::Result<usize>;

    /// Poll for completed asynchronous IO requests.
    ///
    /// For currently supported LocalFile and TdcFile backend, it must not return temporary errors
    /// and may only return permanent errors. So the virtio-blk driver layer will not try to
    /// recover and only pass errors up onto the device manager. When changing the error handling
    /// policy, please do help to update BlockEpollHandler::io_complete().
    fn io_complete(&mut self) -> io::Result<Vec<(u16, u32)>>;

    /// Resubmit IO requests for reconnection.
    fn io_resubmit(&mut self) -> io::Result<()> {
        Ok(())
    }

    /// Reconnect to the backend device when the connection is broken.
    fn reconnect(&mut self, _max_loop: i32) -> io::Result<()> {
        // return ENOSYS by default
        Err(io::Error::from_raw_os_error(38))
    }

    /// Restore (reconnect in fact) to the IO backend.
    fn restore_io_context(&mut self) -> io::Result<()> {
        Ok(())
    }

    /// Is IO context available.
    fn is_io_context_ready(&self) -> bool;
}

