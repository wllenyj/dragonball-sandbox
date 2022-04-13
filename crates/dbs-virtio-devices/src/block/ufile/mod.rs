// Copyright (C) 2019 Alibaba Cloud. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

mod localfile;
pub use self::localfile::LocalFile;

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

    /// Get the raw event fd for data plane.
    fn get_data_evt_fd(&self) -> RawFd;

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
