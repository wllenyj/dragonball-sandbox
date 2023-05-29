// Copyright 2023 Alibaba Cloud. All rights reserved.
// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::missing_safety_doc)]

use std::fmt::{Debug, Formatter, Result};
use std::num::Wrapping;

use vmm_sys_util::fam::{FamStruct, FamStructWrapper};
use vmm_sys_util::generate_fam_struct_impl;

use dbs_versionize::{Version, VersionMap, Versionize, VersionizeError, VersionizeResult};

use dbs_versionize_tests::TestState;

#[test]
fn test_hardcoded_struct_deserialization() {
    // We are testing representation compatibility between versions, at the `versionize`
    // crate level, by checking that only the newly added/removed fields changes between
    // versions are reflected in the hardcoded snapshot.

    #[rustfmt::skip]
    let v1_hardcoded_snapshot: &[u8] = &[
        // usize field (8 bytes), u16 field (2 bytes) +
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x00,
        // u64 (8 bytes), i8 (1 byte), i32 (4 bytes) +
        0xCD, 0xAB, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x20, 0x00, 0x00, 0x00,
        // f32 (4 bytes), f64 (8 bytes), char (1 bytes) +
        0x00, 0x00, 0x00, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x50, 0x40, 0x61,
        // String len (8 bytes) +
        0x0b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // actual String (11 bytes in our case) +
        0x73, 0x6F, 0x6D, 0x65, 0x5F, 0x73, 0x74, 0x72, 0x69, 0x6E, 0x67,
        // enum variant number (4 bytes) + value of that variant (in this case it is
        // of u32 type -> 4 bytes) +
        0x01, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
        // Option variant (1 byte) + value of variant (u8 -> 1 byte) +
        0x01, 0x81,
        // Box: String len (8 bytes) + actual String (17 bytes in this case).
        0x11, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x73, 0x6F, 0x6D, 0x65, 0x5F,
        0x6F, 0x74, 0x68, 0x65, 0x72, 0x5F, 0x73, 0x74, 0x72, 0x69, 0x6E, 0x67,
    ];

    // At version 2 isize (8 bytes), i64 (8 bytes) and bool (1 byte) fields will be also
    // present. At v2 there is also a new variant available for enum, so we can store that in
    // memory and it occupies 4 more bytes than the one stored at v1.
    #[rustfmt::skip]
    let v2_hardcoded_snapshot: &[u8] = &[
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        // New isize field.
        0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x04, 0x00,
        0xCD, 0xAB, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x20, 0x00, 0x00, 0x00,
        // New i64 field.
        0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x50, 0x40, 0x61,
        // New bool field.
        0x01,
        0x0B, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x73, 0x6F, 0x6D, 0x65, 0x5F, 0x73, 0x74, 0x72, 0x69, 0x6E, 0x67,
        // New available enum variant.
        0x02, 0x00, 0x00, 0x00, 0x0E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x81,
        0x11, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x73, 0x6F, 0x6D, 0x65, 0x5F,
        0x6F, 0x74, 0x68, 0x65, 0x72, 0x5F, 0x73, 0x74, 0x72, 0x69, 0x6E, 0x67,
    ];

    // At version 3, u64 and i64 disappear (16 bytes) and Vec (8 + 4 = 12 bytes) and Wrapping
    // (4 bytes) fields are available.
    #[rustfmt::skip]
    let v3_hardcoded_snapshot: &[u8] = &[
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x04, 0x00,
        0xFF, 0x20, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x50, 0x40, 0x61,
        0x01,
        0x0b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x73, 0x6F, 0x6D, 0x65, 0x5F, 0x73, 0x74, 0x72, 0x69, 0x6E, 0x67,
        0x02, 0x00, 0x00, 0x00, 0x0E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x81,
        // Vec len (8 bytes) + actual Vec (4 bytes).
        0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x61, 0x61, 0x61, 0x61,
        0x11, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x73, 0x6F, 0x6D, 0x65, 0x5F,
        0x6F, 0x74, 0x68, 0x65, 0x72, 0x5F, 0x73, 0x74, 0x72, 0x69, 0x6E, 0x67,
        // Wrapping over an u32 (4 bytes).
        0xFF, 0x00, 0x00, 0x00,
    ];

    // At version 4, isize and Vec disappear (20 bytes): 0x6F - 0x14 = 0x5B.
    #[rustfmt::skip]
    let v4_hardcoded_snapshot: &[u8] = &[
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x04, 0x00,
        0xFF, 0x20, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x50, 0x40, 0x61,
        0x01,
        0x0b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x73, 0x6F, 0x6D, 0x65, 0x5F, 0x73, 0x74, 0x72, 0x69, 0x6E, 0x67,
        0x02, 0x00, 0x00, 0x00, 0x0e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, 0x81,
        0x11, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x73, 0x6F, 0x6D, 0x65, 0x5F,
        0x6F, 0x74, 0x68, 0x65, 0x72, 0x5F, 0x73, 0x74, 0x72, 0x69, 0x6E, 0x67,
        0xFF, 0x00, 0x00, 0x00,
    ];

    #[derive(Clone, Debug, PartialEq, Versionize)]
    pub struct TestStruct {
        usize_1: usize,
        #[version(start = "0.2.0", end = "0.4.0", default_fn = "default_isize")]
        isize_1: isize,
        u16_1: u16,
        #[version(end = "0.3.0", default_fn = "default_u64")]
        u64_1: u64,
        i8_1: i8,
        #[version(start = "0.2.0", end = "0.2.0")]
        i16_1: i16,
        i32_1: i32,
        #[version(start = "0.2.0", end = "0.3.0", default_fn = "default_i64")]
        i64_1: i64,
        f32_1: f32,
        f64_1: f64,
        char_1: char,
        #[version(start = "0.2.0", default_fn = "default_bool")]
        bool_1: bool,
        string_1: String,
        enum_1: TestState,
        option_1: Option<u8>,
        #[version(start = "0.3.0", end = "0.4.0", default_fn = "default_vec")]
        vec_1: Vec<char>,
        #[allow(clippy::box_collection)] // we want to explicitly test Box
        box_1: Box<String>,
        #[version(start = "0.3.0")]
        wrapping_1: Wrapping<u32>,
    }

    impl TestStruct {
        fn default_isize(_source_version: &Version) -> isize {
            12isize
        }

        fn default_u64(_source_version: &Version) -> u64 {
            0x0Du64
        }

        fn default_i64(_source_version: &Version) -> i64 {
            0x0Ei64
        }

        fn default_bool(_source_version: &Version) -> bool {
            false
        }

        fn default_vec(_source_version: &Version) -> Vec<char> {
            vec!['v'; 8]
        }
    }

    let mut vm = VersionMap::new();
    vm.set_crate_version(env!("CARGO_PKG_NAME"), "0.1.0")
        .unwrap();

    let mut snapshot_blob = v1_hardcoded_snapshot;

    let mut restored_state =
        <TestStruct as Versionize>::deserialize(&mut snapshot_blob, &vm).unwrap();

    // We expect isize, i16, i64, bool, Vec and Wrapping fields to have the default values at v1.
    let mut expected_state = TestStruct {
        usize_1: 1,
        isize_1: 12,
        u16_1: 4,
        u64_1: 0xABCDu64,
        i8_1: -1,
        i16_1: 0,
        i32_1: 32,
        i64_1: 0x0Ei64,
        f32_1: 0.5,
        f64_1: 64.5,
        char_1: 'a',
        bool_1: false,
        string_1: "some_string".to_owned(),
        enum_1: TestState::One(2),
        option_1: Some(129),
        vec_1: vec!['v'; 8],
        box_1: Box::new("some_other_string".to_owned()),
        wrapping_1: Wrapping(0u32),
    };
    assert_eq!(restored_state, expected_state);

    snapshot_blob = v2_hardcoded_snapshot;

    let mut vm = VersionMap::new();
    vm.set_crate_version(env!("CARGO_PKG_NAME"), "0.2.0")
        .unwrap();
    restored_state = <TestStruct as Versionize>::deserialize(&mut snapshot_blob, &vm).unwrap();

    // We expect only i16, Vec and Wrapping fields to have the default values at v2.
    expected_state = TestStruct {
        usize_1: 1,
        isize_1: 2,
        u16_1: 4,
        u64_1: 0xABCDu64,
        i8_1: -1,
        i16_1: 0,
        i32_1: 32,
        i64_1: 0xFFFFi64,
        f32_1: 0.5,
        f64_1: 64.5,
        char_1: 'a',
        bool_1: true,
        string_1: "some_string".to_owned(),
        enum_1: TestState::Two(14),
        option_1: Some(129),
        vec_1: vec!['v'; 8],
        box_1: Box::new("some_other_string".to_owned()),
        wrapping_1: Wrapping(0u32),
    };
    assert_eq!(restored_state, expected_state);

    snapshot_blob = v3_hardcoded_snapshot;

    let mut vm = VersionMap::new();
    vm.set_crate_version(env!("CARGO_PKG_NAME"), "0.3.0")
        .unwrap();
    restored_state = <TestStruct as Versionize>::deserialize(&mut snapshot_blob, &vm).unwrap();

    // We expect u64, i16 and i64 fields to have the default values at v3.
    expected_state = TestStruct {
        usize_1: 1,
        isize_1: 2,
        u16_1: 4,
        u64_1: 0x0Du64,
        i8_1: -1,
        i16_1: 0,
        i32_1: 32,
        i64_1: 0x0Ei64,
        f32_1: 0.5,
        f64_1: 64.5,
        char_1: 'a',
        bool_1: true,
        string_1: "some_string".to_owned(),
        enum_1: TestState::Two(14),
        option_1: Some(129),
        vec_1: vec!['a'; 4],
        box_1: Box::new("some_other_string".to_owned()),
        wrapping_1: Wrapping(255u32),
    };
    assert_eq!(restored_state, expected_state);

    snapshot_blob = v4_hardcoded_snapshot;

    let mut vm = VersionMap::new();
    vm.set_crate_version(env!("CARGO_PKG_NAME"), "0.4.0")
        .unwrap();
    restored_state = <TestStruct as Versionize>::deserialize(&mut snapshot_blob, &vm).unwrap();

    // We expect isize, u64, i16, i64 and Vec fields to have the default values at v4.
    expected_state = TestStruct {
        usize_1: 1,
        isize_1: 12,
        u16_1: 4,
        u64_1: 0x0Du64,
        i8_1: -1,
        i16_1: 0,
        i32_1: 32,
        i64_1: 0x0Ei64,
        f32_1: 0.5,
        f64_1: 64.5,
        char_1: 'a',
        bool_1: true,
        string_1: "some_string".to_owned(),
        enum_1: TestState::Two(14),
        option_1: Some(129),
        vec_1: vec!['v'; 8],
        box_1: Box::new("some_other_string".to_owned()),
        wrapping_1: Wrapping(255u32),
    };
    assert_eq!(restored_state, expected_state);
}

#[derive(Clone, Debug, PartialEq, Eq, Versionize)]
pub struct A {
    a: u32,
    #[version(start = "0.1.0", end = "0.2.0")]
    b: Option<TestState>,
    #[version(start = "0.2.0", default_fn = "default_c")]
    c: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Versionize)]
pub struct X {
    x: bool,
    a_1: A,
    #[version(end = "0.3.0", default_fn = "default_y")]
    y: Box<usize>,
    #[version(start = "0.3.0", default_fn = "default_z")]
    z: Vec<u8>,
}

impl A {
    fn default_c(_source_version: &Version) -> String {
        "some_string".to_owned()
    }
}

impl X {
    fn default_y(_source_version: &Version) -> Box<usize> {
        Box::from(4)
    }

    fn default_z(_source_version: &Version) -> Vec<u8> {
        vec![16, 4]
    }
}

#[test]
fn test_nested_structs_deserialization() {
    #[rustfmt::skip]
    let v1_hardcoded_snapshot: &[u8] = &[
        // Bool field (1 byte) from X, `a` field from A (4 bytes) +
        0x00, 0x10, 0x00, 0x00, 0x00,
        // `b` field from A: Option type (1 byte), inner enum variant number (4 bytes) +
        // + value of that variant (4 bytes) +
        0x01, 0x01, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00,
        // `y` field from A (8 bytes).
        0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    #[rustfmt::skip]
    let v2_hardcoded_snapshot: &[u8] = &[
        // Bool field (1 byte) from X, `a` field from A (4 bytes) +
        0x00, 0x10, 0x00, 0x00, 0x00,
        // `c` field from X: String len (8 bytes) + actual String;
        // the Option field is not available at v2.
        0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x72, 0x61, 0x6E, 0x64, 0x6F, 0x6D,
        // `y` field from A (8 bytes).
        0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    #[rustfmt::skip]
    let v3_hardcoded_snapshot: &[u8] = &[
        0x00, 0x10, 0x00, 0x00, 0x00,
        0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x72, 0x61, 0x6E, 0x64, 0x6F, 0x6D,
        // `z` field from A (8 bytes).
        0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x18, 0x18,
    ];

    let mut snapshot_blob = v1_hardcoded_snapshot;

    let mut vm = VersionMap::new();
    vm.set_crate_version(env!("CARGO_PKG_NAME"), "0.1.0")
        .unwrap();
    let mut restored_state = <X as Versionize>::deserialize(&mut snapshot_blob, &vm).unwrap();
    // We expect `z` and `c` fields to have the default values.
    let mut expected_state = X {
        x: false,
        a_1: A {
            a: 16u32,
            b: Some(TestState::One(4)),
            c: "some_string".to_owned(),
        },
        y: Box::from(2),
        z: vec![16, 4],
    };
    assert_eq!(restored_state, expected_state);

    snapshot_blob = v2_hardcoded_snapshot;

    let mut vm = VersionMap::new();
    vm.set_crate_version(env!("CARGO_PKG_NAME"), "0.2.0")
        .unwrap();
    restored_state = <X as Versionize>::deserialize(&mut snapshot_blob, &vm).unwrap();

    // We expect `b` and `z` fields to have the default values.
    expected_state = X {
        x: false,
        a_1: A {
            a: 16u32,
            b: None,
            c: "random".to_owned(),
        },
        y: Box::from(2),
        z: vec![16, 4],
    };
    assert_eq!(restored_state, expected_state);

    snapshot_blob = v3_hardcoded_snapshot;

    let mut vm = VersionMap::new();
    vm.set_crate_version(env!("CARGO_PKG_NAME"), "0.3.0")
        .unwrap();
    restored_state = <X as Versionize>::deserialize(&mut snapshot_blob, &vm).unwrap();

    // We expect `b` and `y` fields to have the default values.
    expected_state = X {
        x: false,
        a_1: A {
            a: 16u32,
            b: None,
            c: "random".to_owned(),
        },
        y: Box::from(4),
        z: vec![24; 4],
    };
    assert_eq!(restored_state, expected_state);
}

pub const SIZE: usize = 10;

pub mod dummy_mod {
    pub const SIZE: usize = 20;
}

#[test]
fn test_versionize_struct_with_array() {
    #[derive(Clone, Debug, PartialEq, Versionize)]
    struct TestStruct {
        a: [u32; SIZE],
        b: [u8; dummy_mod::SIZE],
        c: Option<[i16; SIZE]>,
    }

    let test_struct = TestStruct {
        a: [1; SIZE],
        b: [2; dummy_mod::SIZE],
        c: Some([3; SIZE]),
    };

    let mut mem = vec![0; 4096];
    let mut version_map = VersionMap::new();

    test_struct
        .serialize(&mut mem.as_mut_slice(), &mut version_map)
        .unwrap();
    let restored_test_struct = TestStruct::deserialize(&mut mem.as_slice(), &version_map).unwrap();

    assert_eq!(restored_test_struct, test_struct);
}

#[derive(Clone, Debug, PartialEq, Eq, Versionize)]
pub enum DeviceStatus {
    Inactive,
    Active,
    #[version(start = "0.0.1", end = "0.0.999", default_fn = "default_is_activating")]
    IsActivating(u32),
}

impl Default for DeviceStatus {
    fn default() -> Self {
        Self::Inactive
    }
}

impl DeviceStatus {
    fn default_is_activating(&self, srouce_version: &Version) -> VersionizeResult<DeviceStatus> {
        match srouce_version.patch {
            1..=9 => Ok(DeviceStatus::Inactive),
            i => Err(VersionizeError::Serialize(format!(
                "Unknown target version: {}",
                i
            ))),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Versionize)]
pub enum OperationSupported {
    Add,
    Remove,
    RemoveAndAdd(bool),
    #[version(start = "0.1.3", end = "0.1.5", default_fn = "default_update")]
    Update(String),
}

impl Default for OperationSupported {
    fn default() -> Self {
        Self::Add
    }
}

impl OperationSupported {
    fn default_update(&self, srouce_version: &Version) -> VersionizeResult<OperationSupported> {
        match srouce_version.patch {
            1..=9 => Ok(OperationSupported::RemoveAndAdd(true)),
            i => Err(VersionizeError::Serialize(format!(
                "Unknown target version: {}",
                i
            ))),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Versionize)]
pub struct DeviceV1 {
    #[version(ser_fn = "ser_v1")]
    name: String,
    id: Wrapping<u32>,
    some_params: Vec<String>,
    status: DeviceStatus,
    queues: Vec<u8>,
    features: u32,
}

impl DeviceV1 {
    fn ser_v1(&mut self, _ver: &Version) -> VersionizeResult<()> {
        self.features |= 1;
        self.status = DeviceStatus::Active;
        self.some_params.push("active".to_owned());
        self.some_params.retain(|x| x.clone() != *"inactive");

        self.some_params.push("extra_features".to_owned());

        self.features |= 1u32 << 31;
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Versionize)]
pub struct DeviceV2 {
    name: String,
    id: Wrapping<u32>,
    #[version(start = "0.1.2", ser_fn = "ser_is_activated")]
    is_activated: bool,
    some_params: Vec<String>,
    #[version(
        start = "0.1.2",
        default_fn = "default_ops",
        ser_fn = "ser_ops",
        de_fn = "de_ops"
    )]
    operations: Vec<OperationSupported>,
    status: DeviceStatus,
    #[version(
        start = "0.1.2",
        default_fn = "default_queues_limit",
        ser_fn = "ser_queues_limit"
    )]
    no_queues_limit: usize,
    queues: Vec<u8>,
    features: u32,
}

impl DeviceV2 {
    fn default_ops(_srouce_version: &Version) -> Vec<OperationSupported> {
        vec![OperationSupported::Add, OperationSupported::Remove]
    }

    fn default_queues_limit(_srouce_version: &Version) -> usize {
        2
    }

    fn ser_ops(&mut self, _current_version: &Version) -> VersionizeResult<()> {
        Ok(())
    }

    fn de_ops(&mut self, _source_version: &Version) -> VersionizeResult<()> {
        if self.some_params.contains(&"active".to_owned()) {
            self.status = DeviceStatus::Active;
        }
        Ok(())
    }

    fn ser_queues_limit(&mut self, _current_version: &Version) -> VersionizeResult<()> {
        self.some_params.push("extra_features".to_owned());
        Ok(())
    }

    fn ser_is_activated(&mut self, _current_version: &Version) -> VersionizeResult<()> {
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Versionize)]
pub struct Device {
    name: String,
    id: Wrapping<u32>,
    #[version(start = "0.1.2", ser_fn = "ser_is_activated")]
    is_activated: bool,
    some_params: Vec<String>,
    #[version(
        start = "0.1.2",
        default_fn = "default_ops",
        ser_fn = "ser_ops",
        de_fn = "de_ops"
    )]
    operations: Vec<OperationSupported>,
    status: DeviceStatus,
    #[version(
        start = "0.1.2",
        default_fn = "default_queues_limit",
        ser_fn = "ser_queues_limit"
    )]
    no_queues_limit: usize,
    queues: Vec<u8>,
    features: u32,
    #[version(start = "0.1.3", ser_fn = "ser_extra", de_fn = "de_extra")]
    extra_features: u64,
}

impl Device {
    fn default_ops(_srouce_version: &Version) -> Vec<OperationSupported> {
        vec![OperationSupported::Add, OperationSupported::Remove]
    }

    fn default_queues_limit(_srouce_version: &Version) -> usize {
        2
    }

    fn ser_ops(&mut self, current_version: &Version) -> VersionizeResult<()> {
        if current_version < &Version::new(0, 1, 3) {
            self.features |= 1;
        }
        Ok(())
    }

    fn de_ops(&mut self, _source_version: &Version) -> VersionizeResult<()> {
        Ok(())
    }

    fn ser_queues_limit(&mut self, current_version: &Version) -> VersionizeResult<()> {
        if current_version <= &Version::new(0, 1, 3) && self.queues.len() > 2 {
            return Err(VersionizeError::Semantic("Too many queues.".to_owned()));
        }
        Ok(())
    }

    fn ser_is_activated(&mut self, _current_version: &Version) -> VersionizeResult<()> {
        Ok(())
    }

    fn ser_extra(&mut self, _current_version: &Version) -> VersionizeResult<()> {
        if self.queues.len() > self.no_queues_limit {
            return Err(VersionizeError::Semantic("Too many queues.".to_owned()));
        }
        Ok(())
    }

    fn de_extra(&mut self, source_version: &Version) -> VersionizeResult<()> {
        if source_version < &Version::new(0, 1, 3) {
            self.features |= 1u32 << 31;
        }
        if source_version == &Version::new(0, 1, 3) && self.queues.len() >= self.no_queues_limit {
            return Err(VersionizeError::Semantic("Too many queues.".to_owned()));
        }
        Ok(())
    }
}

#[test]
fn test_versionize_struct_with_enums() {
    let state = DeviceV1 {
        name: "block".to_owned(),
        id: Wrapping(1u32),
        some_params: vec!["inactive".to_owned()],
        status: DeviceStatus::Inactive,
        queues: vec![1u8, 2u8],
        features: 6u32,
    };

    let mut snapshot_mem = vec![0u8; 1024];

    let mut vm = VersionMap::new();
    // Serialize as v1.
    state
        .serialize(&mut snapshot_mem.as_mut_slice(), &mut vm)
        .unwrap();

    let mut vm = VersionMap::new();
    vm.set_crate_version(env!("CARGO_PKG_NAME"), "0.1.1")
        .unwrap();
    let mut restored_state =
        <Device as Versionize>::deserialize(&mut snapshot_mem.as_slice(), &vm).unwrap();

    // At v1, all of the semantic functions should be called.
    // `operations` and `no_queues_limit` will take the default values (set by `default_fn`s),
    // `features` will be modified by `ser_ops` and `de_extra`, `status` will be changed to
    // `Active` by `de_ops`, `is_activated` will take the default bool value, `some_params`
    // will be also modified and the other fields will take the original values.
    let mut expected_state = Device {
        name: "block".to_owned(),
        id: Wrapping(1u32),
        is_activated: false,
        some_params: vec!["active".to_owned(), "extra_features".to_owned()],
        operations: vec![OperationSupported::Add, OperationSupported::Remove],
        status: DeviceStatus::Active,
        no_queues_limit: 2,
        queues: vec![1u8, 2u8],
        features: 0x8000_0007u32,
        extra_features: 0u64,
    };
    assert_eq!(expected_state, restored_state);

    let state2 = DeviceV2 {
        name: "block".to_owned(),
        id: Wrapping(1u32),
        is_activated: true,
        some_params: vec!["inactive".to_owned()],
        operations: vec![
            OperationSupported::Add,
            OperationSupported::Update("random".to_owned()),
        ],
        status: DeviceStatus::Inactive,
        no_queues_limit: 3,
        queues: vec![1u8, 2u8],
        features: 6u32,
    };
    let mut vm = VersionMap::new();
    state2
        .serialize(&mut snapshot_mem.as_mut_slice(), &mut vm)
        .unwrap();

    let mut vm = VersionMap::new();
    vm.set_crate_version(env!("CARGO_PKG_NAME"), "0.1.2")
        .unwrap();
    restored_state =
        <Device as Versionize>::deserialize(&mut snapshot_mem.as_slice(), &vm).unwrap();

    // At v2, we expect that only the semantic functions from `extra_features` to be called,
    // this means that `features` and `some_params` will take different values than the ones
    // at v1. `status` won't be modified anymore, `is_activated` and `no_queues_limit` will
    // take this time the original values. `operations` field will contain only the first
    // original element, the second one will be modified by `default_update` because at v2,
    // `Update` is not available.
    expected_state = Device {
        name: "block".to_owned(),
        id: Wrapping(1u32),
        is_activated: true,
        some_params: vec!["inactive".to_owned(), "extra_features".to_owned()],
        operations: vec![
            OperationSupported::Add,
            OperationSupported::RemoveAndAdd(true),
        ],
        status: DeviceStatus::Inactive,
        no_queues_limit: 3,
        queues: vec![1u8, 2u8],
        features: 0x8000_0006u32,
        extra_features: 0u64,
    };
    assert_eq!(expected_state, restored_state);

    let mut state3 = Device {
        name: "block".to_owned(),
        id: Wrapping(1u32),
        is_activated: true,
        some_params: vec!["inactive".to_owned()],
        operations: vec![
            OperationSupported::Add,
            OperationSupported::Update("random".to_owned()),
        ],
        status: DeviceStatus::Inactive,
        no_queues_limit: 3,
        queues: vec![1u8, 2u8],
        features: 6u32,
        extra_features: 0u64,
    };
    let mut vm = VersionMap::new();
    state3
        .serialize(&mut snapshot_mem.as_mut_slice(), &mut vm)
        .unwrap();

    let mut vm = VersionMap::new();
    vm.set_crate_version(env!("CARGO_PKG_NAME"), "0.1.3")
        .unwrap();

    restored_state =
        <Device as Versionize>::deserialize(&mut snapshot_mem.as_slice(), &vm).unwrap();

    // At v3, `Update` variant is available, so it will be deserialized to its original value.
    // We expect no semantic function to be called, so `features` and `some_params` will also
    // take the original values.
    expected_state = Device {
        name: "block".to_owned(),
        id: Wrapping(1u32),
        is_activated: true,
        some_params: vec!["inactive".to_owned()],
        operations: vec![
            OperationSupported::Add,
            OperationSupported::RemoveAndAdd(true),
        ],
        status: DeviceStatus::Inactive,
        no_queues_limit: 3,
        queues: vec![1u8, 2u8],
        features: 6u32,
        extra_features: 0u64,
    };
    assert_eq!(expected_state, restored_state);

    // Test semantic errors.
    state3.queues = vec![1u8, 2u8, 3u8, 4u8];
    let mut vm = VersionMap::new();
    assert_eq!(
        state3
            .serialize(&mut snapshot_mem.as_mut_slice(), &mut vm)
            .unwrap_err(),
        VersionizeError::Semantic("Too many queues.".to_owned())
    );

    state3.queues = vec![1u8, 2u8, 3u8];
    state3
        .serialize(&mut snapshot_mem.as_mut_slice(), &mut vm)
        .unwrap();
    let mut vm = VersionMap::new();
    vm.set_crate_version(env!("CARGO_PKG_NAME"), "0.1.3")
        .unwrap();
    assert_eq!(
        <Device as Versionize>::deserialize(&mut snapshot_mem.as_slice(), &vm).unwrap_err(),
        VersionizeError::Semantic("Too many queues.".to_owned())
    );
}

#[derive(Clone, Debug, PartialEq, Eq, Versionize)]
pub enum State {
    Zero(u32),
    One(bool),
    #[version(start = "0.0.2", end = "0.1.0", default_fn = "default_state_two")]
    Two(Vec<u8>),
    #[version(start = "0.0.2", end = "0.1.0", default_fn = "default_state_three")]
    Three(String),
    #[version(start = "0.0.3", end = "0.1.0", default_fn = "default_state_four")]
    Four(Option<u64>),
}

impl Default for State {
    fn default() -> Self {
        Self::One(false)
    }
}

impl State {
    fn default_state_two(&self, srouce_version: &Version) -> VersionizeResult<State> {
        match srouce_version.patch {
            1 => Ok(State::One(true)),
            2..=9 => Ok(State::Zero(2)),
            i => Err(VersionizeError::Serialize(format!(
                "Unknown target version: {}",
                i
            ))),
        }
    }

    fn default_state_three(&self, srouce_version: &Version) -> VersionizeResult<State> {
        match srouce_version.patch {
            1 => Ok(State::One(false)),
            2..=9 => Ok(State::Zero(3)),
            i => Err(VersionizeError::Serialize(format!(
                "Unknown target version: {}",
                i
            ))),
        }
    }

    fn default_state_four(&self, current_version: &Version) -> VersionizeResult<State> {
        match current_version.patch {
            1..=9 => Ok(State::Zero(4)),
            i => Err(VersionizeError::Serialize(format!(
                "Unknown target version: {}",
                i
            ))),
        }
    }
}

#[test]
fn test_versionize_enum() {
    let mut snapshot_mem = vec![0u8; 1024];

    // First we test that serializing and deserializing an enum variant available at the
    // target version results in the same variant.
    let mut state = State::One(true);
    let mut vm = VersionMap::new();
    state
        .serialize(&mut snapshot_mem.as_mut_slice(), &mut vm)
        .unwrap();

    let mut vm = VersionMap::new();
    vm.set_crate_version(env!("CARGO_PKG_NAME"), "0.0.1")
        .unwrap();
    let mut restored_state =
        <State as Versionize>::deserialize(&mut snapshot_mem.as_slice(), &vm).unwrap();
    assert_eq!(state, restored_state);

    // Now we test `default_fn`s for serialization of enum variants that don't exist in
    // previous versions.
    let mut vm = VersionMap::new();
    state = State::Four(Some(0x1234_5678_8765_4321u64));
    state
        .serialize(&mut snapshot_mem.as_mut_slice(), &mut vm)
        .unwrap();

    let mut vm = VersionMap::new();
    vm.set_crate_version(env!("CARGO_PKG_NAME"), "0.0.1")
        .unwrap();
    restored_state = <State as Versionize>::deserialize(&mut snapshot_mem.as_slice(), &vm).unwrap();
    assert_eq!(restored_state, State::Zero(4));

    let mut vm = VersionMap::new();
    state = State::Two(vec![0, 1]);
    state
        .serialize(&mut snapshot_mem.as_mut_slice(), &mut vm)
        .unwrap();

    restored_state = <State as Versionize>::deserialize(&mut snapshot_mem.as_slice(), &vm).unwrap();
    assert_eq!(restored_state, State::Zero(2));

    state = State::Three("some_string".to_owned());
    let mut vm = VersionMap::new();
    state
        .serialize(&mut snapshot_mem.as_mut_slice(), &mut vm)
        .unwrap();

    let mut vm = VersionMap::new();
    vm.set_crate_version(env!("CARGO_PKG_NAME"), "0.0.1")
        .unwrap();
    restored_state = <State as Versionize>::deserialize(&mut snapshot_mem.as_slice(), &vm).unwrap();
    assert_eq!(restored_state, State::Zero(3));
}

#[derive(Clone, Debug, PartialEq, Versionize)]
pub struct S {
    a: f64,
    b: i64,
}

#[derive(Clone, Debug, PartialEq, Versionize)]
pub struct TestV1 {
    usize_1: usize,
    vec_1: Vec<u16>,
    u64_1: u64,
    enum_1: State,
    i8_1: i8,
    i16_1: i16,
    char_1: char,
}

#[derive(Clone, Debug, PartialEq, Versionize)]
pub struct TestV2 {
    usize_1: usize,
    #[version(start = "0.0.2")]
    isize_1: isize,
    #[version(start = "0.0.2")]
    u8_1: u8,
    vec_1: Vec<u16>,
    u64_1: u64,
    #[version(start = "0.0.2", ser_fn = "ser_bool")]
    bool_1: bool,
    enum_1: State,
    i8_1: i8,
    i16_1: i16,
    #[version(start = "0.0.2", default_fn = "default_box", de_fn = "de_box")]
    box_1: Box<S>,
    #[version(start = "0.0.2", default_fn = "default_f32")]
    f32_1: f32,
    char_1: char,
    option_1: Option<String>,
}

impl TestV2 {
    fn default_box(_srouce_version: &Version) -> Box<S> {
        Box::new(S { a: 1.5, b: 2 })
    }
    fn default_f32(_srouce_version: &Version) -> f32 {
        0.5
    }
    fn ser_bool(&mut self, _current_version: &Version) -> VersionizeResult<()> {
        Ok(())
    }
    fn de_box(&mut self, _source_version: &Version) -> VersionizeResult<()> {
        self.option_1 = Some("box_change".to_owned());
        if self.vec_1.len() == 3 {
            return Err(VersionizeError::Semantic(
                "Vec len is too small.".to_owned(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Versionize)]
pub struct TestV3 {
    usize_1: usize,
    #[version(start = "0.0.2", end = "0.0.3", ser_fn = "ser_isize")]
    isize_1: isize,
    #[version(start = "0.0.2")]
    u8_1: u8,
    vec_1: Vec<u16>,
    #[version(start = "0.0.3")]
    wrapping_1: Wrapping<u32>,
    #[version(end = "0.0.3")]
    u64_1: u64,
    #[version(start = "0.0.2")]
    bool_1: bool,
    enum_1: State,
    i8_1: i8,
    i16_1: i16,
    #[version(start = "0.0.3")]
    i32_1: i32,
    #[version(start = "0.0.2", default_fn = "default_box")]
    box_1: Box<S>,
    #[version(start = "0.0.2", end = "0.0.3")]
    f32_1: f32,
    char_1: char,
    #[version(end = "0.0.3", ser_fn = "ser_option")]
    option_1: Option<String>,
}

impl TestV3 {
    fn default_box(_srouce_version: &Version) -> Box<S> {
        Box::new(S { a: 1.5, b: 2 })
    }
    fn ser_option(&mut self, _current_version: &Version) -> VersionizeResult<()> {
        self.u8_1 += 2;
        if self.vec_1.len() == 10 {
            return Err(VersionizeError::Semantic("Vec is full.".to_owned()));
        }
        Ok(())
    }
    fn ser_isize(&mut self, _current_version: &Version) -> VersionizeResult<()> {
        if self.i8_1 == -1 {
            return Err(VersionizeError::Semantic(
                "Unexpected value for `i8` field.".to_owned(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Versionize)]
pub struct Test {
    usize_1: usize,
    #[version(
        start = "0.0.2",
        end = "0.0.3",
        ser_fn = "ser_isize",
        de_fn = "de_isize"
    )]
    isize_1: isize,
    #[version(start = "0.0.2")]
    u8_1: u8,
    #[version(end = "0.0.4", default_fn = "default_vec")]
    vec_1: Vec<u16>,
    #[version(start = "0.0.3")]
    wrapping_1: Wrapping<u32>,
    #[version(
        end = "0.0.3",
        default_fn = "default_u64",
        ser_fn = "ser_u64",
        de_fn = "de_u64"
    )]
    u64_1: u64,
    #[version(start = "0.0.2", ser_fn = "ser_bool")]
    bool_1: bool,
    enum_1: State,
    i8_1: i8,
    i16_1: i16,
    #[version(start = "0.0.3", end = "0.0.4")]
    i32_1: i32,
    #[version(start = "0.0.2", default_fn = "default_box", de_fn = "de_box")]
    box_1: Box<S>,
    #[version(start = "0.0.2", end = "0.0.3", default_fn = "default_f32")]
    f32_1: f32,
    char_1: char,
    #[version(
        end = "0.0.3",
        default_fn = "default_option",
        ser_fn = "ser_option",
        de_fn = "de_option"
    )]
    option_1: Option<String>,
}

impl Test {
    fn default_vec(_srouce_version: &Version) -> Vec<u16> {
        vec![0x0102u16; 4]
    }

    fn default_u64(_srouce_version: &Version) -> u64 {
        0x0102_0102_0102_0102u64
    }

    fn default_f32(_srouce_version: &Version) -> f32 {
        0.5
    }

    fn default_box(_srouce_version: &Version) -> Box<S> {
        Box::new(S { a: 1.5, b: 2 })
    }

    fn default_option(_srouce_version: &Version) -> Option<String> {
        Some("something".to_owned())
    }

    fn ser_isize(&mut self, current_version: &Version) -> VersionizeResult<()> {
        assert_ne!(current_version.patch, 2);
        self.vec_1.push(0x0304u16);
        if self.i8_1 == -1 {
            return Err(VersionizeError::Semantic(
                "Unexpected value for `i8` field.".to_owned(),
            ));
        }
        Ok(())
    }
    fn ser_u64(&mut self, current_version: &Version) -> VersionizeResult<()> {
        if current_version.patch >= 3 {
            self.vec_1.pop();
            if self.u8_1 == 4 {
                self.bool_1 = false;
            }
        }
        Ok(())
    }

    fn ser_bool(&mut self, current_version: &Version) -> VersionizeResult<()> {
        if current_version.patch < 2 {
            self.vec_1.push(0x0506u16);
            self.vec_1.push(0x0708u16);
        }
        Ok(())
    }

    fn ser_option(&mut self, current_version: &Version) -> VersionizeResult<()> {
        if current_version.patch == 9 {
            self.u8_1 += 2;
            if self.vec_1.len() == 10 {
                return Err(VersionizeError::Semantic("Vec is full.".to_owned()));
            }
        }
        Ok(())
    }

    fn de_isize(&mut self, source_version: &Version) -> VersionizeResult<()> {
        if source_version.patch != 2 {
            self.u8_1 += 3;
        }
        Ok(())
    }

    fn de_u64(&mut self, source_version: &Version) -> VersionizeResult<()> {
        if source_version.patch >= 3 {
            self.vec_1.push(0x0101u16);
        }
        Ok(())
    }

    fn de_box(&mut self, source_version: &Version) -> VersionizeResult<()> {
        if source_version.patch < 2 {
            self.option_1 = Some("box_change".to_owned());
            if self.vec_1.is_empty() {
                return Err(VersionizeError::Semantic(
                    "Vec len is too small.".to_owned(),
                ));
            }
        }
        Ok(())
    }

    fn de_option(&mut self, source_version: &Version) -> VersionizeResult<()> {
        if source_version.patch >= 3 {
            self.enum_1 = State::Two(vec![1; 4]);
        }
        Ok(())
    }
}

#[test]
fn test_versionize_struct() {
    let mut vm = VersionMap::new();

    let mut state1 = TestV1 {
        usize_1: 0x0102_0304_0506_0708usize,
        vec_1: vec![0x1122u16; 5],
        u64_1: 0x0102_0304_0506_0708u64,
        enum_1: State::Four(Some(0x0102_0304_0506_0708u64)),
        i8_1: 8,
        i16_1: -12,
        char_1: 'c',
    };
    let mut snapshot_mem = vec![0u8; 1024];

    // Serialize as v1.
    state1
        .serialize(&mut snapshot_mem.as_mut_slice(), &mut vm)
        .unwrap();

    let mut vm = VersionMap::new();
    vm.set_crate_version(env!("CARGO_PKG_NAME"), "0.0.1")
        .unwrap();
    let mut restored_state =
        <Test as Versionize>::deserialize(&mut snapshot_mem.as_slice(), &vm).unwrap();

    let mut expected_state = Test {
        // usize field exists at all versions, will take the original value.
        usize_1: 0x0102_0304_0506_0708usize,
        // isize field will take the default value as it is not available at v1.
        isize_1: 0isize,
        // u8 field doesn't exist at v1, it wll take the default value and then it will be
        // modified by `de_isize`: 0 + 3 = 3.
        u8_1: 3,
        vec_1: vec![0x1122u16, 0x1122u16, 0x1122u16, 0x1122u16, 0x1122u16],
        // We expect here to have the default value.
        wrapping_1: Wrapping(0u32),
        // We expect here to have the original value.
        u64_1: 0x0102_0304_0506_0708u64,
        // We expect here to have the default value.
        bool_1: false,
        // This will take the default value for state `Four` and v1.
        enum_1: State::Zero(4),
        // i8, i16 fields take the original values.
        i8_1: 8,
        i16_1: -12,
        // i32 field takes the default value.
        i32_1: 0,
        // Box and f32 fields will take the default values set by `default_fn`s.
        box_1: Box::new(S { a: 1.5, b: 2 }),
        f32_1: 0.5,
        // We expect this field to take the original value.
        char_1: 'c',
        // This field will be modified by `de_box`.
        option_1: Some("box_change".to_owned()),
    };
    assert_eq!(expected_state, restored_state);

    // Serialize as v2.
    let state2 = TestV2 {
        usize_1: 0x0102_0304_0506_0708usize,
        isize_1: -0x1122_3344_5566_7788isize,
        u8_1: 4,
        vec_1: vec![0x1122u16; 5],
        u64_1: 0x0102_0304_0506_0708u64,
        bool_1: false,
        enum_1: State::Four(Some(0x0102_0304_0506_0708u64)),
        i8_1: 8,
        i16_1: -12,
        box_1: Box::new(S { a: 4.5, b: 4 }),
        f32_1: 1.25,
        char_1: 'c',
        option_1: None,
    };
    let mut vm = VersionMap::new();
    state2
        .serialize(&mut snapshot_mem.as_mut_slice(), &mut vm)
        .unwrap();

    let mut vm = VersionMap::new();
    vm.set_crate_version(env!("CARGO_PKG_NAME"), "0.0.2")
        .unwrap();
    restored_state = <Test as Versionize>::deserialize(&mut snapshot_mem.as_slice(), &vm).unwrap();

    // At v2 isize, u8, bool, box and f32 fields will be available, their semantic fns won't
    // be called.
    expected_state = Test {
        usize_1: 0x0102_0304_0506_0708usize,
        isize_1: -0x1122_3344_5566_7788isize,
        u8_1: 4,
        // This should take the original value this time.
        vec_1: vec![0x1122u16, 0x1122u16, 0x1122u16, 0x1122u16, 0x1122u16],
        wrapping_1: Wrapping(0u32),
        u64_1: 0x0102_0304_0506_0708u64,
        bool_1: false,
        // This will take the default value for state `Four` and v2.
        enum_1: State::Zero(4),
        i8_1: 8,
        i16_1: -12,
        i32_1: 0,
        box_1: Box::new(S { a: 4.5, b: 4 }),
        f32_1: 1.25,
        char_1: 'c',
        option_1: None,
    };
    assert_eq!(expected_state, restored_state);

    // Serialize as v3.
    let mut state3 = TestV3 {
        usize_1: 0x0102_0304_0506_0708usize,
        isize_1: -0x1122_3344_5566_7788isize,
        u8_1: 4,
        vec_1: vec![0x1122u16; 5],
        wrapping_1: Wrapping(4u32),
        u64_1: 0x0102_0304_0506_0708u64,
        bool_1: false,
        enum_1: State::Four(Some(0x0102_0304_0506_0708u64)),
        i8_1: 8,
        i16_1: -12,
        i32_1: -0x1234_5678,
        box_1: Box::new(S { a: 4.5, b: 4 }),
        f32_1: 1.25,
        char_1: 'c',
        option_1: None,
    };
    let mut vm = VersionMap::new();
    state3
        .serialize(&mut snapshot_mem.as_mut_slice(), &mut vm)
        .unwrap();

    let mut vm = VersionMap::new();
    vm.set_crate_version(env!("CARGO_PKG_NAME"), "0.0.3")
        .unwrap();
    restored_state = <Test as Versionize>::deserialize(&mut snapshot_mem.as_slice(), &vm).unwrap();

    expected_state = Test {
        usize_1: 0x0102_0304_0506_0708usize,
        isize_1: 0isize,
        // This field will be modified by `de_isize` and `ser_option`: 4 + 2 + 3 = 9.
        u8_1: 9,
        // Vec field will be modified by `ser_isize` (add one elem), `ser_u64` (remove one elem)
        // and `de_64` (add one elem).
        vec_1: vec![
            0x1122u16, 0x1122u16, 0x1122u16, 0x1122u16, 0x1122u16, 0x0101u16,
        ],
        wrapping_1: Wrapping(4u32),
        u64_1: 0x0102_0102_0102_0102u64,
        bool_1: false,
        enum_1: State::Two(vec![1; 4]),
        i8_1: 8,
        i16_1: -12,
        i32_1: -0x1234_5678,
        box_1: Box::new(S { a: 4.5, b: 4 }),
        f32_1: 0.5,
        char_1: 'c',
        // We expect this field to take the default value set by its `default_fn`.
        option_1: Some("something".to_owned()),
    };
    assert_eq!(expected_state, restored_state);

    // Test semantic errors.
    let mut snapshot_mem = vec![0u8; 1024];
    state1.vec_1 = Vec::new();
    let mut vm = VersionMap::new();
    state1
        .serialize(&mut snapshot_mem.as_mut_slice(), &mut vm)
        .unwrap();

    let mut vm = VersionMap::new();
    vm.set_crate_version(env!("CARGO_PKG_NAME"), "0.0.1")
        .unwrap();
    assert_eq!(
        <Test as Versionize>::deserialize(&mut snapshot_mem.as_slice(), &vm).unwrap_err(),
        VersionizeError::Semantic("Vec len is too small.".to_owned())
    );

    state3.vec_1 = vec![0x1122u16; 10];
    let mut vm = VersionMap::new();
    assert_eq!(
        state3
            .serialize(&mut snapshot_mem.as_mut_slice(), &mut vm)
            .unwrap_err(),
        VersionizeError::Semantic("Vec is full.".to_owned())
    );

    state3.i8_1 = -1;
    assert_eq!(
        state3
            .serialize(&mut snapshot_mem.as_mut_slice(), &mut vm)
            .unwrap_err(),
        VersionizeError::Semantic("Unexpected value for `i8` field.".to_owned())
    );

    // Test serialize and deserialize errors.
    snapshot_mem = vec![0u8; 8];
    // Serializing `state` will fail due to the small size of `snapshot_mem`.
    assert_eq!(
        state2
            .serialize(&mut snapshot_mem.as_mut_slice(), &mut vm)
            .unwrap_err(),
        VersionizeError::Serialize(
            "Io(Error { kind: WriteZero, message: \"failed to write whole buffer\" })".to_owned()
        )
    );
    snapshot_mem = vec![0u8; 256];

    state2
        .serialize(&mut snapshot_mem.as_mut_slice(), &mut vm)
        .unwrap();
    snapshot_mem.truncate(10);
    // Deserialization will fail if we don't use the whole `snapshot_mem` resulted from
    // serialization.
    assert_eq!(
        <Test as Versionize>::deserialize(&mut snapshot_mem.as_slice(), &vm).unwrap_err(),
        VersionizeError::Deserialize(
            "Io(Error { kind: UnexpectedEof, message: \"failed to fill whole buffer\" })"
                .to_owned()
        )
    );
}

#[repr(C)]
#[derive(Clone, Debug, Default, Versionize)]
struct MessageV1 {
    pub len: u32,
    pub padding: u32,
    pub value: u32,
    pub entries: __IncompleteArrayField<u32>,
}
type MessageV1FamStructWrapper = FamStructWrapper<MessageV1>;
generate_fam_struct_impl!(MessageV1, u32, entries, u32, len, 100);

#[repr(C)]
#[derive(Clone, Debug, Default, Versionize)]
struct MessageV2 {
    pub len: u32,
    pub padding: u32,
    pub value: u32,
    #[version(start = "0.1.2", default_fn = "default_extra_value")]
    pub extra_value: u16,
    pub entries: __IncompleteArrayField<u32>,
}
type MessageV2FamStructWrapper = FamStructWrapper<MessageV2>;
generate_fam_struct_impl!(MessageV2, u32, entries, u32, len, 100);

impl MessageV2 {
    fn default_extra_value(_source_version: &Version) -> u16 {
        4
    }
}

#[repr(C)]
#[derive(Clone, Debug, Default, Versionize)]
struct MessageV3 {
    pub len: u32,
    pub padding: u32,
    pub value: u32,
    #[version(start = "0.1.2", default_fn = "default_extra_value")]
    pub extra_value: u16,
    #[version(start = "0.1.3", default_fn = "default_status")]
    pub status: Wrapping<bool>,
    pub entries: __IncompleteArrayField<u32>,
}
type MessageV3FamStructWrapper = FamStructWrapper<MessageV3>;
generate_fam_struct_impl!(MessageV3, u32, entries, u32, len, 100);

impl MessageV3 {
    fn default_extra_value(_source_version: &Version) -> u16 {
        4
    }
    fn default_status(_source_version: &Version) -> Wrapping<bool> {
        Wrapping(false)
    }
}

#[repr(C)]
#[derive(Clone, Debug, Default, Versionize)]
struct Message {
    pub len: u32,
    #[version(end = "0.1.4")]
    pub padding: u32,
    pub value: u32,
    #[version(start = "0.1.2", default_fn = "default_extra_value")]
    pub extra_value: u16,
    #[version(start = "0.1.3", end = "0.1.4", default_fn = "default_status")]
    pub status: Wrapping<bool>,
    pub entries: __IncompleteArrayField<u32>,
}

impl Message {
    fn default_extra_value(_source_version: &Version) -> u16 {
        4
    }

    fn default_status(_source_version: &Version) -> Wrapping<bool> {
        Wrapping(false)
    }
}

#[repr(C)]
#[derive(Clone, Debug, Default, Versionize)]
struct Message2 {
    pub len: u32,
    #[version(end = "0.1.4")]
    pub padding: u32,
    pub value: u32,
    #[version(start = "0.1.2", default_fn = "default_extra_value")]
    pub extra_value: u16,
    #[version(start = "0.1.3", end = "0.1.4", default_fn = "default_status")]
    pub status: Wrapping<bool>,
    pub entries: __IncompleteArrayField<u32>,
}

impl Message2 {
    fn default_extra_value(_source_version: &Version) -> u16 {
        4
    }

    fn default_status(_source_version: &Version) -> Wrapping<bool> {
        Wrapping(false)
    }
}

generate_fam_struct_impl!(Message, u32, entries, u32, len, 100);
// Duplicated structure used but with max_len 1 - for negative testing.
generate_fam_struct_impl!(Message2, u32, entries, u32, len, 1);

#[repr(C)]
#[derive(Default)]
pub struct __IncompleteArrayField<T>(::std::marker::PhantomData<T>, [T; 0]);

impl<T> __IncompleteArrayField<T> {
    #[inline]
    pub fn new() -> Self {
        __IncompleteArrayField(::std::marker::PhantomData, [])
    }
    #[inline]
    pub unsafe fn as_ptr(&self) -> *const T {
        self as *const __IncompleteArrayField<T> as *const T
    }
    #[inline]
    pub unsafe fn as_mut_ptr(&mut self) -> *mut T {
        self as *mut __IncompleteArrayField<T> as *mut T
    }
    #[inline]
    pub unsafe fn as_slice(&self, len: usize) -> &[T] {
        ::std::slice::from_raw_parts(self.as_ptr(), len)
    }
    #[inline]
    pub unsafe fn as_mut_slice(&mut self, len: usize) -> &mut [T] {
        ::std::slice::from_raw_parts_mut(self.as_mut_ptr(), len)
    }
}

impl<T> Debug for __IncompleteArrayField<T> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result {
        fmt.write_str("__IncompleteArrayField")
    }
}

impl<T> ::std::clone::Clone for __IncompleteArrayField<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl<T> Versionize for __IncompleteArrayField<T> {
    #[inline]
    fn serialize<W: std::io::Write>(
        &self,
        mut _writer: W,
        _version_map: &mut VersionMap,
    ) -> VersionizeResult<()> {
        Ok(())
    }

    #[inline]
    fn deserialize<R: std::io::Read>(
        _reader: R,
        _version_map: &VersionMap,
    ) -> VersionizeResult<Self> {
        Ok(Self::new())
    }
}

type MessageFamStructWrapper = FamStructWrapper<Message>;
type Message2FamStructWrapper = FamStructWrapper<Message2>;

#[test]
fn test_deserialize_famstructwrapper_invalid_len() {
    let mut vm = VersionMap::new();

    // Create FamStructWrapper with len 2
    let state = MessageFamStructWrapper::new(0).unwrap();
    let mut buffer = [0; 256];

    state
        .serialize(&mut buffer.as_mut_slice(), &mut vm)
        .unwrap();

    // the `len` field of the header is the first serialized field.
    // Let's corrupt it by making it bigger than the actual number of serialized elements
    buffer[0] = 255;

    assert_eq!(
        MessageFamStructWrapper::deserialize(&mut buffer.as_slice(), &vm).unwrap_err(),
        VersionizeError::Deserialize("Mismatch between length of FAM specified in FamStruct header (255) and actual size of FAM (0)".to_string())
    );
}

#[test]
fn test_versionize_famstructwrapper() {
    let mut vm = VersionMap::new();

    let mut state1 = MessageV1FamStructWrapper::new(0).unwrap();
    state1.as_mut_fam_struct().padding = 8;

    state1.push(1).unwrap();
    state1.push(2).unwrap();

    let mut snapshot_mem = vec![0u8; 256];

    // Serialize as v1.
    state1
        .serialize(&mut snapshot_mem.as_mut_slice(), &mut vm)
        .unwrap();

    let mut vm = VersionMap::new();
    vm.set_crate_version(env!("CARGO_PKG_NAME"), "0.1.0")
        .unwrap();
    let mut restored_state =
        <MessageFamStructWrapper as Versionize>::deserialize(&mut snapshot_mem.as_slice(), &vm)
            .unwrap();

    let mut original_values = state1.as_slice();
    let mut restored_values = restored_state.as_slice();
    assert_eq!(original_values, restored_values);
    assert_eq!(
        restored_values.len(),
        state1.as_fam_struct_ref().len as usize
    );

    assert_eq!(
        state1.as_fam_struct_ref().padding,
        restored_state.as_fam_struct_ref().padding
    );
    assert_eq!(4, restored_state.as_fam_struct_ref().extra_value);
    assert_eq!(Wrapping(false), restored_state.as_fam_struct_ref().status);

    // Serialize as v2.
    let mut state2 = MessageV2FamStructWrapper::new(0).unwrap();
    state2.as_mut_fam_struct().padding = 8;
    state2.as_mut_fam_struct().extra_value = 16;

    state2.push(1).unwrap();
    state2.push(2).unwrap();

    let mut vm = VersionMap::new();
    state2
        .serialize(&mut snapshot_mem.as_mut_slice(), &mut vm)
        .unwrap();

    let mut vm = VersionMap::new();
    vm.set_crate_version(env!("CARGO_PKG_NAME"), "0.1.2")
        .unwrap();
    restored_state =
        <MessageFamStructWrapper as Versionize>::deserialize(&mut snapshot_mem.as_slice(), &vm)
            .unwrap();

    original_values = state2.as_slice();
    restored_values = restored_state.as_slice();
    assert_eq!(original_values, restored_values);

    assert_eq!(
        state2.as_fam_struct_ref().padding,
        restored_state.as_fam_struct_ref().padding
    );
    // `extra_value` is available at v2, so it will take its original value.
    assert_eq!(
        state2.as_fam_struct_ref().extra_value,
        restored_state.as_fam_struct_ref().extra_value
    );
    assert_eq!(Wrapping(false), restored_state.as_fam_struct_ref().status);

    // Serialize as v3.
    let mut state3 = MessageV3FamStructWrapper::new(0).unwrap();
    state3.as_mut_fam_struct().padding = 8;
    state3.as_mut_fam_struct().extra_value = 16;
    state3.as_mut_fam_struct().status = Wrapping(true);

    state3.push(1).unwrap();
    state3.push(2).unwrap();

    let mut vm = VersionMap::new();
    state3
        .serialize(&mut snapshot_mem.as_mut_slice(), &mut vm)
        .unwrap();

    let mut vm = VersionMap::new();
    vm.set_crate_version(env!("CARGO_PKG_NAME"), "0.1.3")
        .unwrap();
    restored_state =
        <MessageFamStructWrapper as Versionize>::deserialize(&mut snapshot_mem.as_slice(), &vm)
            .unwrap();

    assert_eq!(
        state3.as_fam_struct_ref().padding,
        restored_state.as_fam_struct_ref().padding
    );
    assert_eq!(
        state3.as_fam_struct_ref().extra_value,
        restored_state.as_fam_struct_ref().extra_value
    );
    // At v3, `status` field exists, so it will take its original value.
    assert_eq!(Wrapping(true), restored_state.as_fam_struct_ref().status);

    // Serialize as v4.
    let mut state4 = MessageFamStructWrapper::new(0).unwrap();
    state4.as_mut_fam_struct().padding = 8;
    state4.as_mut_fam_struct().extra_value = 16;
    state4.as_mut_fam_struct().status = Wrapping(true);

    state4.push(1).unwrap();
    state4.push(2).unwrap();
    let mut vm = VersionMap::new();
    state4
        .serialize(&mut snapshot_mem.as_mut_slice(), &mut vm)
        .unwrap();

    let mut vm = VersionMap::new();
    vm.set_crate_version(env!("CARGO_PKG_NAME"), "0.1.4")
        .unwrap();
    restored_state =
        <MessageFamStructWrapper as Versionize>::deserialize(&mut snapshot_mem.as_slice(), &vm)
            .unwrap();

    // At v4, `padding` field will take the default u32 value.
    assert_eq!(0, restored_state.as_fam_struct_ref().padding);
    assert_eq!(
        state4.as_fam_struct_ref().extra_value,
        restored_state.as_fam_struct_ref().extra_value
    );
    // `status` is not available anymore, so it will take the default value.
    assert_eq!(Wrapping(false), restored_state.as_fam_struct_ref().status);

    snapshot_mem = vec![0u8; 16];

    let mut vm = VersionMap::new();
    assert_eq!(
        state1
            .serialize(&mut snapshot_mem.as_mut_slice(), &mut vm)
            .unwrap_err(),
        VersionizeError::Serialize(
            "Io(Error { kind: WriteZero, message: \"failed to write whole buffer\" })".to_owned()
        )
    );
}

#[derive(Clone, Versionize)]
pub struct FamStructTestV1 {
    some_u8: u8,
    message_box: Box<MessageV1FamStructWrapper>,
    messages: Vec<MessageV1FamStructWrapper>,
}

#[derive(Clone, Versionize)]
pub struct FamStructTestV2 {
    some_u8: u8,
    message_box: Box<MessageV2FamStructWrapper>,
    #[version(start = "0.1.2")]
    some_option: Option<S>,
    messages: Vec<MessageV2FamStructWrapper>,
}

#[derive(Clone, Versionize)]
pub struct FamStructTest {
    some_u8: u8,
    message_box: Box<MessageFamStructWrapper>,
    #[version(start = "0.1.2", default_fn = "default_option", de_fn = "de_option")]
    some_option: Option<S>,
    #[version(start = "0.1.3")]
    some_string: String,
    #[version(end = "0.1.3", default_fn = "default_message", de_fn = "de_message")]
    messages: Vec<MessageFamStructWrapper>,
}

impl FamStructTest {
    fn default_message(_source_version: &Version) -> Vec<MessageFamStructWrapper> {
        let mut f = MessageFamStructWrapper::new(0).unwrap();
        f.as_mut_fam_struct().padding = 1;
        f.as_mut_fam_struct().extra_value = 2;

        f.push(10).unwrap();
        f.push(20).unwrap();

        vec![f]
    }

    fn default_option(_source_version: &Version) -> Option<S> {
        Some(S { a: 0.5, b: 0 })
    }

    fn de_message(&mut self, source_version: &Version) -> VersionizeResult<()> {
        // Fail if semantic deserialization is called for v2.
        if source_version.patch > 2 {
            self.some_option = None;
            self.some_string = "some_new_string".to_owned();
        }
        Ok(())
    }

    fn de_option(&mut self, source_version: &Version) -> VersionizeResult<()> {
        // Fail if semantic deserialization is called for a version >= 2.
        if source_version.patch < 2 {
            let mut f = MessageFamStructWrapper::new(0).unwrap();
            f.as_mut_fam_struct().padding = 3;
            f.as_mut_fam_struct().extra_value = 4;

            f.push(10).unwrap();
            f.push(20).unwrap();

            self.messages.push(f);
        }
        Ok(())
    }
}

#[test]
fn test_versionize_struct_with_famstructs() {
    let mut vm = VersionMap::new();

    let mut snapshot_mem = vec![0u8; 1024];

    let mut f = MessageV1FamStructWrapper::new(0).unwrap();
    f.as_mut_fam_struct().padding = 5;
    //f.as_mut_fam_struct().extra_value = 6;
    f.push(10).unwrap();

    let mut f2 = MessageV1FamStructWrapper::new(0).unwrap();
    f2.as_mut_fam_struct().padding = 7;
    //f2.as_mut_fam_struct().extra_value = 8;
    f2.push(20).unwrap();

    let state = FamStructTestV1 {
        some_u8: 1,
        messages: vec![f],
        message_box: Box::new(f2),
    };

    state
        .serialize(&mut snapshot_mem.as_mut_slice(), &mut vm)
        .unwrap();

    let mut vm = VersionMap::new();
    vm.set_crate_version(env!("CARGO_PKG_NAME"), "0.1.1")
        .unwrap();
    let mut restored_state =
        <FamStructTest as Versionize>::deserialize(&mut snapshot_mem.as_slice(), &vm).unwrap();

    // At version 1, we expect `de_option` and `de_message` to be called.
    // `some_string` and `some_option` will take the default values.
    assert_eq!(restored_state.some_string, String::default());
    assert_eq!(restored_state.some_option, Some(S { a: 0.5, b: 0 }));
    let messages = restored_state.messages;

    // We expect to have 2 elements in the messages Vec (the one with which it was initialized and
    // the one inserted by `de_option`).
    assert_eq!(messages.len(), 2);
    for message in messages.iter() {
        assert_eq!(message.as_fam_struct_ref().extra_value, 4);
        assert_eq!(message.as_fam_struct_ref().status, Wrapping(false));
    }
    assert_eq!(messages[0].as_fam_struct_ref().padding, 5);
    assert_eq!(messages[1].as_fam_struct_ref().padding, 3);

    // Serialize as v2.
    let mut f = MessageV2FamStructWrapper::new(0).unwrap();
    f.as_mut_fam_struct().padding = 5;
    f.as_mut_fam_struct().extra_value = 6;
    f.push(10).unwrap();

    let mut f2 = MessageV2FamStructWrapper::new(0).unwrap();
    f2.as_mut_fam_struct().padding = 7;
    f2.as_mut_fam_struct().extra_value = 8;
    f2.push(20).unwrap();

    let state2 = FamStructTestV2 {
        some_u8: 1,
        messages: vec![f],
        some_option: None,
        message_box: Box::new(f2),
    };
    let mut vm = VersionMap::new();
    state2
        .serialize(&mut snapshot_mem.as_mut_slice(), &mut vm)
        .unwrap();

    let mut vm = VersionMap::new();
    vm.set_crate_version(env!("CARGO_PKG_NAME"), "0.1.2")
        .unwrap();
    restored_state =
        <FamStructTest as Versionize>::deserialize(&mut snapshot_mem.as_slice(), &vm).unwrap();

    assert_eq!(restored_state.some_string, String::default());
    // `some_option` is available at v2, so it will take the original value.
    assert_eq!(restored_state.some_option, None);
    let messages = restored_state.messages;
    // We expect to have only one element in `messages` as `de_option` shouldn't be called
    // this time.
    assert_eq!(messages.len(), 1);

    // Serialize as v3.
    let mut f = MessageFamStructWrapper::new(0).unwrap();
    f.as_mut_fam_struct().padding = 5;
    f.as_mut_fam_struct().extra_value = 6;
    f.push(10).unwrap();

    let mut f2 = MessageFamStructWrapper::new(0).unwrap();
    f2.as_mut_fam_struct().padding = 7;
    f2.as_mut_fam_struct().extra_value = 8;
    f2.push(20).unwrap();

    let state = FamStructTest {
        some_u8: 1,
        messages: vec![f],
        some_string: "some_string".to_owned(),
        message_box: Box::new(f2),
        some_option: None,
    };
    let mut vm = VersionMap::new();
    state
        .serialize(&mut snapshot_mem.as_mut_slice(), &mut vm)
        .unwrap();

    let mut vm = VersionMap::new();
    vm.set_crate_version(env!("CARGO_PKG_NAME"), "0.1.4")
        .unwrap();
    restored_state =
        <FamStructTest as Versionize>::deserialize(&mut snapshot_mem.as_slice(), &vm).unwrap();

    // `some_string` is also available at v3.
    assert_eq!(restored_state.some_string, "some_new_string".to_owned());
    assert_eq!(restored_state.some_option, None);
    let messages = restored_state.messages;
    // `messages` field is not available anymore at v3, it will take the default value,
    // set by the corresponding `default_fn`.
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].as_fam_struct_ref().padding, 1);
}

#[derive(Clone, Versionize)]
pub struct SomeStructV1 {
    #[version(ser_fn = "ser_u16")]
    message: MessageV1FamStructWrapper,
}
impl SomeStructV1 {
    fn ser_u16(&mut self, _current_version: &Version) -> VersionizeResult<()> {
        self.message.as_mut_fam_struct().padding += 2;

        Ok(())
    }
}

#[derive(Clone, Versionize)]
pub struct SomeStruct {
    message: MessageFamStructWrapper,
    #[version(start = "0.2.0", ser_fn = "ser_u16")]
    some_u16: u16,
}

impl SomeStruct {
    fn ser_u16(&mut self, _current_version: &Version) -> VersionizeResult<()> {
        self.message.as_mut_fam_struct().padding += 2;

        Ok(())
    }
}

#[derive(Clone, Versionize)]
pub struct SomeStruct2 {
    message: Message2FamStructWrapper,
    #[version(start = "0.2.0", ser_fn = "ser_u16")]
    some_u16: u16,
}

impl SomeStruct2 {
    fn ser_u16(&mut self, _current_version: &Version) -> VersionizeResult<()> {
        self.message.as_mut_fam_struct().padding += 2;

        Ok(())
    }
}

// `Clone` issue fixed: https://github.com/rust-vmm/vmm-sys-util/issues/85.
// We are keeping this as regression test.
#[test]
fn test_famstructwrapper_clone() {
    // Test that having a `FamStructWrapper<T>` in a structure that implements
    // Clone will result in keeping with their original values, only the number
    // of entries and the entries array when serializing.
    let mut vm = VersionMap::new();

    let mut f = MessageV1FamStructWrapper::new(0).unwrap();
    f.as_mut_fam_struct().padding = 8;

    f.push(1).unwrap();
    f.push(2).unwrap();

    let state = SomeStructV1 { message: f };

    let mut snapshot_mem = vec![0u8; 128];

    // Serialize as v1.
    state
        .serialize(&mut snapshot_mem.as_mut_slice(), &mut vm)
        .unwrap();

    let mut vm = VersionMap::new();
    vm.set_crate_version(env!("CARGO_PKG_NAME"), "0.1.0")
        .unwrap();
    let mut restored_state =
        <SomeStruct as Versionize>::deserialize(&mut snapshot_mem.as_slice(), &vm).unwrap();

    // Negative scenario - FamStruct versionize impl fails due to SizeLimitExceeded.
    assert!(<SomeStruct2 as Versionize>::deserialize(&mut snapshot_mem.as_slice(), &vm).is_err());

    let original_values = state.message.as_slice();
    let restored_values = restored_state.message.as_slice();

    assert_ne!(
        state.message.as_fam_struct_ref().padding,
        restored_state.message.as_fam_struct_ref().padding
    );
    assert_eq!(original_values, restored_values);
    // `padding` field will have its value serialized (8), and then it will be incremented with 2
    // by `ser_u16`.
    assert_eq!(10, restored_state.message.as_fam_struct_ref().padding);

    // Serialize as v2.
    let mut f = MessageFamStructWrapper::new(0).unwrap();
    f.as_mut_fam_struct().padding = 8;

    f.push(1).unwrap();
    f.push(2).unwrap();

    let state2 = SomeStruct {
        message: f,
        some_u16: 2,
    };
    let mut vm = VersionMap::new();
    state2
        .serialize(&mut snapshot_mem.as_mut_slice(), &mut vm)
        .unwrap();

    let mut vm = VersionMap::new();
    vm.set_crate_version(env!("CARGO_PKG_NAME"), "0.2.0")
        .unwrap();
    restored_state =
        <SomeStruct as Versionize>::deserialize(&mut snapshot_mem.as_slice(), &vm).unwrap();

    assert_eq!(0, restored_state.message.as_fam_struct_ref().padding);
}