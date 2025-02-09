/*++

Licensed under the Apache-2.0 license.

File Name:

    hmac384_tests.rs

Abstract:

    File contains test cases for HMAC-384 API

--*/

#![no_std]
#![no_main]

use caliptra_drivers::{
    Array4x12, Array4xN, Ecc384, Ecc384PrivKeyOut, Ecc384Scalar, Ecc384Seed, Hmac384, KeyId,
    KeyReadArgs, KeyUsage, KeyWriteArgs,
};
use caliptra_kat::Hmac384Kat;

use caliptra_test_harness::test_suite;

fn test_hmac0() {
    let key: [u8; 48] = [
        0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b,
        0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b,
        0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b, 0x0b,
        0x0b, 0x0b, 0x0b,
    ];

    let data: [u8; 8] = [0x48, 0x69, 0x20, 0x54, 0x68, 0x65, 0x72, 0x65];

    let result: [u8; 48] = [
        0xb6, 0xa8, 0xd5, 0x63, 0x6f, 0x5c, 0x6a, 0x72, 0x24, 0xf9, 0x97, 0x7d, 0xcf, 0x7e, 0xe6,
        0xc7, 0xfb, 0x6d, 0x0c, 0x48, 0xcb, 0xde, 0xe9, 0x73, 0x7a, 0x95, 0x97, 0x96, 0x48, 0x9b,
        0xdd, 0xbc, 0x4c, 0x5d, 0xf6, 0x1d, 0x5b, 0x32, 0x97, 0xb4, 0xfb, 0x68, 0xda, 0xb9, 0xf1,
        0xb5, 0x82, 0xc2,
    ];

    let mut out_tag = Array4x12::default();
    let actual = Hmac384::default().hmac(
        (&Array4x12::from(key)).into(),
        (&data).into(),
        (&mut out_tag).into(),
    );

    assert!(actual.is_ok());
    assert_eq!(out_tag, Array4x12::from(result));
}

fn test_hmac1() {
    let key: [u8; 48] = [
        0x4a, 0x65, 0x66, 0x65, 0x4a, 0x65, 0x66, 0x65, 0x4a, 0x65, 0x66, 0x65, 0x4a, 0x65, 0x66,
        0x65, 0x4a, 0x65, 0x66, 0x65, 0x4a, 0x65, 0x66, 0x65, 0x4a, 0x65, 0x66, 0x65, 0x4a, 0x65,
        0x66, 0x65, 0x4a, 0x65, 0x66, 0x65, 0x4a, 0x65, 0x66, 0x65, 0x4a, 0x65, 0x66, 0x65, 0x4a,
        0x65, 0x66, 0x65,
    ];

    let data: [u8; 28] = [
        0x77, 0x68, 0x61, 0x74, 0x20, 0x64, 0x6f, 0x20, 0x79, 0x61, 0x20, 0x77, 0x61, 0x6e, 0x74,
        0x20, 0x66, 0x6f, 0x72, 0x20, 0x6e, 0x6f, 0x74, 0x68, 0x69, 0x6e, 0x67, 0x3f,
    ];

    let result: [u8; 48] = [
        0x2c, 0x73, 0x53, 0x97, 0x4f, 0x18, 0x42, 0xfd, 0x66, 0xd5, 0x3c, 0x45, 0x2c, 0xa4, 0x21,
        0x22, 0xb2, 0x8c, 0x0b, 0x59, 0x4c, 0xfb, 0x18, 0x4d, 0xa8, 0x6a, 0x36, 0x8e, 0x9b, 0x8e,
        0x16, 0xf5, 0x34, 0x95, 0x24, 0xca, 0x4e, 0x82, 0x40, 0x0c, 0xbd, 0xe0, 0x68, 0x6d, 0x40,
        0x33, 0x71, 0xc9,
    ];

    let mut out_tag = Array4x12::default();
    let actual = Hmac384::default().hmac(
        (&Array4x12::from(key)).into(),
        (&data).into(),
        (&mut out_tag).into(),
    );

    assert!(actual.is_ok());
    assert_eq!(out_tag, Array4x12::from(result));
}

fn test_hmac3() {
    //
    // Step 1: Place a key in the key-vault.
    //
    let seed = [0u8; 48];
    let nonce = Array4xN::default();
    let mut key_usage = KeyUsage::default();
    key_usage.set_hmac_key(true);
    let key_out_1 = KeyWriteArgs {
        id: KeyId::KeyId0,
        usage: key_usage, // hmac_key
    };
    let result = Ecc384::default().key_pair(
        Ecc384Seed::from(&Ecc384Scalar::from(seed)),
        &nonce,
        Ecc384PrivKeyOut::from(key_out_1),
    );
    assert!(result.is_ok());

    //
    // Step 2: Hash the data with the key from key-vault.
    // Key is [ 0xfe, 0xee, 0xf5, 0x54, 0x4a, 0x76, 0x56, 0x49, 0x90, 0x12, 0x8a, 0xd1, 0x89, 0xe8, 0x73, 0xf2,
    //          0x1f, 0xd, 0xfd, 0x5a, 0xd7, 0xe2, 0xfa, 0x86, 0x11, 0x27, 0xee, 0x6e, 0x39, 0x4c, 0xa7, 0x84,
    //          0x87, 0x1c, 0x1a, 0xec, 0x3, 0x2c, 0x7a, 0x8b, 0x10, 0xb9, 0x3e, 0xe, 0xab, 0x89, 0x46, 0xd6,,];
    //
    let data: [u8; 8] = [0x48, 0x69, 0x20, 0x54, 0x68, 0x65, 0x72, 0x65];

    let result: [u8; 48] = [
        0x41, 0x6a, 0x79, 0x62, 0xa, 0x9a, 0x1d, 0xa2, 0xe6, 0x27, 0x26, 0xfc, 0xc0, 0x1a, 0xaf,
        0xaa, 0xd7, 0x7b, 0xd5, 0x66, 0xc9, 0x10, 0xec, 0xd3, 0x76, 0xbf, 0xda, 0x6d, 0x87, 0x77,
        0x3a, 0x38, 0x12, 0x97, 0x5d, 0xee, 0x8a, 0xab, 0xef, 0xa1, 0x79, 0x7f, 0xc0, 0x2b, 0x2e,
        0x38, 0x5d, 0xb0,
    ];

    let mut out_tag = Array4x12::default();
    let key = KeyReadArgs::new(KeyId::KeyId0);
    let actual = Hmac384::default().hmac(key.into(), (&data).into(), (&mut out_tag).into());

    assert!(actual.is_ok());
    assert_eq!(out_tag, Array4x12::from(result));
}

fn test_hmac4() {
    //
    // Step 1: Place a key in the key-vault.
    //
    let seed = [0u8; 48];
    let nonce = Array4xN::default();
    let mut key_usage = KeyUsage::default();
    key_usage.set_hmac_key(true);
    let key_out_1 = KeyWriteArgs {
        id: KeyId::KeyId0,
        usage: key_usage, // hmac_key
    };
    let result = Ecc384::default().key_pair(
        Ecc384Seed::from(&Ecc384Scalar::from(seed)),
        &nonce,
        Ecc384PrivKeyOut::from(key_out_1),
    );
    assert!(result.is_ok());

    //
    // Step 2: Hash the data with the key from key-vault.
    // Key is [ 0xfe, 0xee, 0xf5, 0x54, 0x4a, 0x76, 0x56, 0x49, 0x90, 0x12, 0x8a, 0xd1, 0x89, 0xe8, 0x73, 0xf2,
    //          0x1f, 0xd, 0xfd, 0x5a, 0xd7, 0xe2, 0xfa, 0x86, 0x11, 0x27, 0xee, 0x6e, 0x39, 0x4c, 0xa7, 0x84,
    //          0x87, 0x1c, 0x1a, 0xec, 0x3, 0x2c, 0x7a, 0x8b, 0x10, 0xb9, 0x3e, 0xe, 0xab, 0x89, 0x46, 0xd6,];
    //

    let data: [u8; 28] = [
        0x77, 0x68, 0x61, 0x74, 0x20, 0x64, 0x6f, 0x20, 0x79, 0x61, 0x20, 0x77, 0x61, 0x6e, 0x74,
        0x20, 0x66, 0x6f, 0x72, 0x20, 0x6e, 0x6f, 0x74, 0x68, 0x69, 0x6e, 0x67, 0x3f,
    ];

    let result: [u8; 48] = [
        0x7e, 0x97, 0xa1, 0xb8, 0x23, 0x87, 0x43, 0x50, 0x9d, 0x5, 0xb9, 0x38, 0x91, 0x27, 0xc3,
        0x8d, 0x57, 0xf2, 0xf2, 0x18, 0x48, 0x35, 0x16, 0x49, 0xf5, 0xbe, 0xbc, 0x7e, 0x3c, 0x59,
        0x13, 0xaa, 0xdf, 0x5, 0xce, 0x52, 0x7b, 0x89, 0xb7, 0xd6, 0xb, 0xcf, 0x8a, 0x14, 0xe4,
        0xbc, 0x31, 0x23,
    ];

    let mut out_tag = Array4x12::default();
    let key = KeyReadArgs::new(KeyId::KeyId0);

    let actual = Hmac384::default().hmac(key.into(), (&data).into(), (&mut out_tag).into());

    assert!(actual.is_ok());
    assert_eq!(out_tag, Array4x12::from(result));
}

fn test_hmac5() {
    //
    // Step 1: Place a key in the key-vault.
    //
    let seed = [0u8; 48];
    let nonce = Array4xN::default();
    let mut key_usage = KeyUsage::default();
    key_usage.set_hmac_key(true);
    let key_out_1 = KeyWriteArgs {
        id: KeyId::KeyId0,
        usage: key_usage, // hmac_key
    };
    let result = Ecc384::default().key_pair(
        Ecc384Seed::from(&Ecc384Scalar::from(seed)),
        &nonce,
        Ecc384PrivKeyOut::from(key_out_1),
    );
    assert!(result.is_ok());

    //
    // Step 2: Hash the data with the key from key-vault.
    // Key is [ 0xfe, 0xee, 0xf5, 0x54, 0x4a, 0x76, 0x56, 0x49, 0x90, 0x12, 0x8a, 0xd1, 0x89, 0xe8, 0x73, 0xf2,
    //          0x1f, 0xd, 0xfd, 0x5a, 0xd7, 0xe2, 0xfa, 0x86, 0x11, 0x27, 0xee, 0x6e, 0x39, 0x4c, 0xa7, 0x84,
    //          0x87, 0x1c, 0x1a, 0xec, 0x3, 0x2c, 0x7a, 0x8b, 0x10, 0xb9, 0x3e, 0xe, 0xab, 0x89, 0x46, 0xd6,];
    //
    let data: [u8; 48] = [
        0xe9, 0x54, 0x51, 0x9b, 0xd1, 0x02, 0xfc, 0xe1, 0x94, 0xf3, 0xf9, 0x12, 0x60, 0xcc, 0x3d,
        0xf4, 0x54, 0x73, 0x35, 0xb4, 0x5d, 0x82, 0x4f, 0xfb, 0x5a, 0xc4, 0x94, 0xef, 0x8f, 0x69,
        0x97, 0xd8, 0x76, 0xd3, 0x70, 0xf4, 0x31, 0x47, 0x7c, 0xd2, 0x7b, 0x5d, 0xb6, 0xc2, 0x6f,
        0x15, 0xc7, 0x9e,
    ];

    let result: [u8; 48] = [
        0xc7, 0x43, 0xf2, 0xbb, 0x41, 0xf1, 0x81, 0x7d, 0x6b, 0xaa, 0x10, 0xad, 0x34, 0xca, 0x7f,
        0x87, 0xb2, 0xcd, 0x51, 0x4a, 0x0, 0xde, 0xf9, 0x38, 0x6, 0x1b, 0x4a, 0xf, 0x65, 0x47,
        0xac, 0x6b, 0xb0, 0x4e, 0xec, 0xb7, 0x55, 0x15, 0x2c, 0xdd, 0xb4, 0xd5, 0x95, 0xee, 0x27,
        0x97, 0xf7, 0xb2,
    ];

    let mut out_tag = Array4x12::default();
    let key = KeyReadArgs::new(KeyId::KeyId0);
    let actual = Hmac384::default().hmac(key.into(), (&data).into(), (&mut out_tag).into());

    assert!(actual.is_ok());
    assert_eq!(out_tag, Array4x12::from(result));
}

///
/// Step 1:
/// Key From Key Vault
/// Generate the output tag in the buffer.
/// Generate the HMAC of the output tag in the buffer - step_1 Tag
///
///
/// Step 2:
/// Key From Key Vault
/// Generate the output tag that goes in the KV
/// Generate the HMAC of the tag in KV and the tag goes in specified buffer
///
///

fn test_hmac6() {
    //
    // Step 1: Place a key in the key-vault.
    //
    // Key is [ 0xfe, 0xee, 0xf5, 0x54, 0x4a, 0x76, 0x56, 0x49, 0x90, 0x12, 0x8a, 0xd1, 0x89, 0xe8, 0x73, 0xf2,
    //          0x1f, 0xd, 0xfd, 0x5a, 0xd7, 0xe2, 0xfa, 0x86, 0x11, 0x27, 0xee, 0x6e, 0x39, 0x4c, 0xa7, 0x84,
    //          0x87, 0x1c, 0x1a, 0xec, 0x3, 0x2c, 0x7a, 0x8b, 0x10, 0xb9, 0x3e, 0xe, 0xab, 0x89, 0x46, 0xd6,];
    //
    let seed = [0u8; 48];
    let nonce = Array4xN::default();
    let mut key_usage = KeyUsage::default();
    key_usage.set_hmac_key(true);
    let key_out_1 = KeyWriteArgs {
        id: KeyId::KeyId0,
        usage: key_usage, // hmac_key
    };
    let result = Ecc384::default().key_pair(
        Ecc384Seed::from(&Ecc384Scalar::from(seed)),
        &nonce,
        Ecc384PrivKeyOut::from(key_out_1),
    );
    assert!(result.is_ok());

    // Key vault key to be used for all the operations. This is a constant
    let key = KeyReadArgs::new(KeyId::KeyId0);

    let data: [u8; 28] = [
        0x77, 0x68, 0x61, 0x74, 0x20, 0x64, 0x6f, 0x20, 0x79, 0x61, 0x20, 0x77, 0x61, 0x6e, 0x74,
        0x20, 0x66, 0x6f, 0x72, 0x20, 0x6e, 0x6f, 0x74, 0x68, 0x69, 0x6e, 0x67, 0x3f,
    ];

    let result: [u8; 48] = [
        0x7e, 0x97, 0xa1, 0xb8, 0x23, 0x87, 0x43, 0x50, 0x9d, 0x5, 0xb9, 0x38, 0x91, 0x27, 0xc3,
        0x8d, 0x57, 0xf2, 0xf2, 0x18, 0x48, 0x35, 0x16, 0x49, 0xf5, 0xbe, 0xbc, 0x7e, 0x3c, 0x59,
        0x13, 0xaa, 0xdf, 0x5, 0xce, 0x52, 0x7b, 0x89, 0xb7, 0xd6, 0xb, 0xcf, 0x8a, 0x14, 0xe4,
        0xbc, 0x31, 0x23,
    ];

    // Take the Data Generate the Tag in buffer
    let mut out_tag = Array4x12::default();
    let actual = Hmac384::default().hmac(key.into(), (&data).into(), (&mut out_tag).into());
    assert!(actual.is_ok());
    assert_eq!(out_tag, Array4x12::from(result));

    let step_1_result_expected: [u8; 48] = [
        0x88, 0xe8, 0x47, 0xe, 0x8, 0x10, 0xc2, 0xf6, 0x7d, 0x56, 0x66, 0x37, 0x43, 0x14, 0x9c,
        0x65, 0xe, 0x4c, 0x0, 0x3e, 0xd9, 0x97, 0x98, 0x49, 0xbb, 0x53, 0x2, 0xd7, 0xce, 0x5e,
        0xf3, 0x62, 0xc2, 0xa3, 0xb8, 0x26, 0xcb, 0xee, 0xbf, 0x97, 0x32, 0xca, 0x62, 0xa, 0x9d,
        0xe4, 0x6b, 0xe3,
    ];

    // Generate the HMAC of the Tag in to a hmac_step_1
    let mut hmac_step_1 = Array4x12::default();
    let actual = Hmac384::default().hmac(key.into(), (&result).into(), (&mut hmac_step_1).into());
    assert!(actual.is_ok());
    assert_eq!(hmac_step_1, Array4x12::from(step_1_result_expected));

    // Generate the Tag Of Original Data and put the tag In KV @5.  KV @5 will be used as data in the next step
    let mut key_usage = KeyUsage::default();
    key_usage.set_hmac_data(true);
    let out_tag = KeyWriteArgs::new(KeyId::KeyId5, key_usage);
    let actual = Hmac384::default().hmac(key.into(), (&data).into(), out_tag.into());
    assert!(actual.is_ok());

    // Data From Key Vault generate HMAC in to output buffer
    let mut hmac_step_2 = Array4x12::default();
    let data_input: KeyReadArgs = KeyReadArgs::new(KeyId::KeyId5);

    let actual = Hmac384::default().hmac(key.into(), data_input.into(), (&mut hmac_step_2).into());

    assert!(actual.is_ok());
    assert_eq!(hmac_step_1, hmac_step_2);
}

fn test_hmac_multi_block() {
    let key: [u8; 48] = [
        0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61,
        0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61,
        0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61,
        0x61, 0x61, 0x61,
    ];

    let data: [u8; 130] = [
        0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F,
        0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x61, 0x62, 0x63, 0x64,
        0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F, 0x70, 0x71, 0x72, 0x73,
        0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68,
        0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77,
        0x78, 0x79, 0x7A, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C,
        0x6D, 0x6E, 0x6F, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x61,
        0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F, 0x70,
        0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A,
    ];

    let result: [u8; 48] = [
        0x70, 0xF1, 0xF6, 0x3C, 0x8C, 0x0A, 0x0D, 0xFE, 0x09, 0x65, 0xE7, 0x3D, 0x79, 0x62, 0x93,
        0xFD, 0x6E, 0xCD, 0x56, 0x43, 0xB4, 0x20, 0x15, 0x46, 0x58, 0x7E, 0xBD, 0x46, 0xCD, 0x07,
        0xE3, 0xEA, 0xE2, 0x51, 0x4A, 0x61, 0xC1, 0x61, 0x44, 0x24, 0xE7, 0x71, 0xCC, 0x4B, 0x7C,
        0xCA, 0xC8, 0xC3,
    ];

    let mut out_tag = Array4x12::default();
    let actual = Hmac384::default().hmac(
        (&Array4x12::from(key)).into(),
        (&data).into(),
        (&mut out_tag).into(),
    );

    assert!(actual.is_ok());
    assert_eq!(out_tag, Array4x12::from(result));
}

fn test_hmac_exact_single_block() {
    let key: [u8; 48] = [
        0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61,
        0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61,
        0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61,
        0x61, 0x61, 0x61,
    ];

    let data: [u8; 130] = [
        0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F,
        0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x61, 0x62, 0x63, 0x64,
        0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F, 0x70, 0x71, 0x72, 0x73,
        0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68,
        0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77,
        0x78, 0x79, 0x7A, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C,
        0x6D, 0x6E, 0x6F, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x61,
        0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F, 0x70,
        0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A,
    ];

    let result: [u8; 48] = [
        0x70, 0xF1, 0xF6, 0x3C, 0x8C, 0x0A, 0x0D, 0xFE, 0x09, 0x65, 0xE7, 0x3D, 0x79, 0x62, 0x93,
        0xFD, 0x6E, 0xCD, 0x56, 0x43, 0xB4, 0x20, 0x15, 0x46, 0x58, 0x7E, 0xBD, 0x46, 0xCD, 0x07,
        0xE3, 0xEA, 0xE2, 0x51, 0x4A, 0x61, 0xC1, 0x61, 0x44, 0x24, 0xE7, 0x71, 0xCC, 0x4B, 0x7C,
        0xCA, 0xC8, 0xC3,
    ];

    let mut out_tag = Array4x12::default();
    let actual = Hmac384::default().hmac(
        (&Array4x12::from(key)).into(),
        (&data).into(),
        (&mut out_tag).into(),
    );

    assert!(actual.is_ok());
    assert_eq!(out_tag, Array4x12::from(result));
}

fn test_hmac_multi_block_two_step() {
    let key: [u8; 48] = [
        0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61,
        0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61,
        0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61, 0x61,
        0x61, 0x61, 0x61,
    ];

    let data: [u8; 130] = [
        0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F,
        0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x61, 0x62, 0x63, 0x64,
        0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F, 0x70, 0x71, 0x72, 0x73,
        0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68,
        0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77,
        0x78, 0x79, 0x7A, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C,
        0x6D, 0x6E, 0x6F, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x61,
        0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6A, 0x6B, 0x6C, 0x6D, 0x6E, 0x6F, 0x70,
        0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A,
    ];

    let result: [u8; 48] = [
        0x70, 0xF1, 0xF6, 0x3C, 0x8C, 0x0A, 0x0D, 0xFE, 0x09, 0x65, 0xE7, 0x3D, 0x79, 0x62, 0x93,
        0xFD, 0x6E, 0xCD, 0x56, 0x43, 0xB4, 0x20, 0x15, 0x46, 0x58, 0x7E, 0xBD, 0x46, 0xCD, 0x07,
        0xE3, 0xEA, 0xE2, 0x51, 0x4A, 0x61, 0xC1, 0x61, 0x44, 0x24, 0xE7, 0x71, 0xCC, 0x4B, 0x7C,
        0xCA, 0xC8, 0xC3,
    ];

    let mut out_tag = Array4x12::default();
    let hmac384 = Hmac384::default();
    let mut hmac_op = hmac384
        .hmac_init((&Array4x12::from(key)).into(), (&mut out_tag).into())
        .unwrap();
    assert!(hmac_op.update(&data).is_ok());
    let actual = hmac_op.finalize();
    assert!(actual.is_ok());
    assert_eq!(out_tag, Array4x12::from(result));
}

fn test_kat() {
    assert_eq!(
        Hmac384Kat::default().execute(&Hmac384::default()).is_ok(),
        true
    );
}

test_suite! {
    test_kat,
    test_hmac0,
    test_hmac1,
    test_hmac3,
    test_hmac4,
    test_hmac5,
    test_hmac6,
    test_hmac_multi_block,
    test_hmac_exact_single_block,
    test_hmac_multi_block_two_step,
}
