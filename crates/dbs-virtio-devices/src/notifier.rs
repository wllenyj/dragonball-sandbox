// Copyright 2019 Alibaba Cloud. All rights reserved.
// SPDX-License-Identifier: Apache-2.0 AND BSD-3-Clause

//! Wrappers over `InterruptNotifier` to support virtio device interrupt management.

use std::sync::Arc;

use dbs_interrupt::{
    InterruptIndex, InterruptNotifier, InterruptSourceGroup, InterruptSourceType,
    InterruptStatusRegister32, LegacyNotifier, MsiNotifier,
};

use crate::{VIRTIO_INTR_CONFIG, VIRTIO_INTR_VRING};

/// Create an interrupt notifier for virtio device change events.
pub fn create_device_notifier(
    group: Arc<Box<dyn InterruptSourceGroup>>,
    intr_status: Arc<InterruptStatusRegister32>,
    intr_index: InterruptIndex,
) -> Arc<dyn InterruptNotifier> {
    match group.interrupt_type() {
        InterruptSourceType::LegacyIrq => {
            Arc::new(LegacyNotifier::new(group, intr_status, VIRTIO_INTR_CONFIG))
        }
        InterruptSourceType::MsiIrq => Arc::new(MsiNotifier::new(group, intr_index)),
    }
}

/// Create an interrupt notifier for virtio queue notification events.
pub fn create_queue_notifier(
    group: Arc<Box<dyn InterruptSourceGroup>>,
    intr_status: Arc<InterruptStatusRegister32>,
    intr_index: InterruptIndex,
) -> Arc<dyn InterruptNotifier> {
    match group.interrupt_type() {
        InterruptSourceType::LegacyIrq => {
            Arc::new(LegacyNotifier::new(group, intr_status, VIRTIO_INTR_VRING))
        }
        InterruptSourceType::MsiIrq => Arc::new(MsiNotifier::new(group, intr_index)),
    }
}
