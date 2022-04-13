// Copyright (C) 2019 Alibaba Cloud. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

mod localfile;
pub use self::localfile::LocalFile;

pub mod aio;
pub mod io_uring;

use std::fs::File;
use std::io::{self, Read, Seek, Write};
use std::os::unix::io::RawFd;

use vmm_sys_util::eventfd::EventFd;

use crate::block::IoDataDesc;

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

    /// Submit asynchronous Read IO requests.
    fn io_read_submit(
        &mut self,
        offset: i64,
        iovecs: &mut Vec<IoDataDesc>,
        user_data: u16,
    ) -> io::Result<usize>;

    /// Submit asynchronous Write IO requests.
    fn io_write_submit(
        &mut self,
        offset: i64,
        iovecs: &mut Vec<IoDataDesc>,
        user_data: u16,
    ) -> io::Result<usize>;

    /// Poll for completed asynchronous IO requests.
    ///
    /// For currently supported LocalFile backend, it must not return temporary errors
    /// and may only return permanent errors. So the virtio-blk driver layer will not try to
    /// recover and only pass errors up onto the device manager. When changing the error handling
    /// policy, please do help to update BlockEpollHandler::io_complete().
    fn io_complete(&mut self) -> io::Result<Vec<(u16, u32)>>;
}

/// Traits for the backend IO engine, such as aio or io-uring.
pub trait IoEngine {
    /// Returns the EventFd that will notify when something is ready.
    fn event_fd(&self) -> &EventFd;

    /// Submit asynchronous Read requests.
    fn readv(
        &mut self,
        offset: i64,
        iovecs: &mut Vec<IoDataDesc>,
        user_data: u64,
    ) -> io::Result<usize>;

    /// Submit asynchronous Write requests.
    fn writev(
        &mut self,
        offset: i64,
        iovecs: &mut Vec<IoDataDesc>,
        user_data: u64,
    ) -> io::Result<usize>;

    /// Poll for completed asynchronous IO requests.
    ///
    /// Return the vector of (user data, result code).
    /// NOTE: complete need to drain io events fd.
    fn complete(&mut self) -> io::Result<Vec<(u64, i64)>>;
}
