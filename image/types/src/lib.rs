/*++

Licensed under the Apache-2.0 license.

File Name:

   lib.rs

Abstract:

    File contains data strucutres for the firmware image bundle.

--*/

#![cfg_attr(not(feature = "std"), no_std)]

use core::ops::Range;

use memoffset::{offset_of, span_of};
use zerocopy::{AsBytes, FromBytes};

pub const MANIFEST_MARKER: u32 = 0x4E414D43;
pub const VENDOR_ECC_KEY_COUNT: u32 = 4;
pub const MAX_TOC_ENTRY_COUNT: u32 = 2;
pub const IMAGE_REVISION_BYTE_SIZE: usize = 20;
pub const ECC384_SCALAR_WORD_SIZE: usize = 12;
pub const ECC384_SCALAR_BYTE_SIZE: usize = 48;
pub const SHA384_DIGEST_WORD_SIZE: usize = 12;
pub const SHA384_DIGEST_BYTE_SIZE: usize = 48;
pub const IMAGE_BYTE_SIZE: usize = 128 * 1024;
pub const IMAGE_MANIFEST_BYTE_SIZE: usize = core::mem::size_of::<ImageManifest>();

pub type ImageScalar = [u32; ECC384_SCALAR_WORD_SIZE];
pub type ImageDigest = [u32; SHA384_DIGEST_WORD_SIZE];
pub type ImageRevision = [u8; IMAGE_REVISION_BYTE_SIZE];
pub type ImageEccPrivKey = ImageScalar;

#[repr(C)]
#[derive(AsBytes, FromBytes, Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct ImageEccPubKey {
    /// X Coordinate
    pub x: ImageScalar,

    /// Y Coordinate
    pub y: ImageScalar,
}

#[repr(C)]
#[derive(AsBytes, FromBytes, Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct ImageEccSignature {
    /// Random point
    pub r: ImageScalar,

    /// Proof
    pub s: ImageScalar,
}

/// Caliptra Image Bundle
#[cfg(feature = "std")]
#[derive(Debug, Default)]
pub struct ImageBundle {
    /// Manifest
    pub manifest: ImageManifest,

    /// FMC
    pub fmc: Vec<u8>,

    /// Runtime
    pub runtime: Vec<u8>,
}

#[cfg(feature = "std")]
impl ImageBundle {
    pub fn to_bytes(&self) -> std::io::Result<Vec<u8>> {
        use std::io::ErrorKind;
        let mut result = vec![];
        result.extend_from_slice(self.manifest.as_bytes());
        if self.manifest.fmc.offset as usize != result.len() {
            return Err(std::io::Error::new(
                ErrorKind::Other,
                "actual fmc offset does not match manifest",
            ));
        }
        if self.manifest.fmc.size as usize != self.fmc.len() {
            return Err(std::io::Error::new(
                ErrorKind::Other,
                "actual fmc size does not match manifest",
            ));
        }
        result.extend_from_slice(&self.fmc);
        if self.manifest.runtime.offset as usize != result.len() {
            return Err(std::io::Error::new(
                ErrorKind::Other,
                "actual runtime offset does not match manifest",
            ));
        }
        if self.manifest.runtime.size as usize != self.runtime.len() {
            return Err(std::io::Error::new(
                ErrorKind::Other,
                "actual runtime size does not match manifest",
            ));
        }
        result.extend_from_slice(&self.runtime);
        Ok(result)
    }
}

/// Calipatra Image Manifest
#[repr(C)]
#[derive(AsBytes, FromBytes, Default, Debug)]
pub struct ImageManifest {
    /// Marker
    pub marker: u32,

    /// Size of `Manifest` strucuture
    pub size: u32,

    /// Preamle
    pub preamble: ImagePreamble,

    /// Header
    pub header: ImageHeader,

    /// First Mutable Code TOC Entry
    pub fmc: ImageTocEntry,

    /// Runtime TOC Entry
    pub runtime: ImageTocEntry,
}

impl ImageManifest {
    /// Returns the `Range<u32>` containing the vendor public keys
    pub fn vendor_pub_keys_range() -> Range<u32> {
        let offset = offset_of!(ImageManifest, preamble) as u32;
        let span = span_of!(ImagePreamble, vendor_pub_keys);
        span.start as u32 + offset..span.end as u32 + offset
    }

    /// Returns the `Range<u32>` containing the specified vendor public key,
    /// or an empty range if the index is invalid.
    pub fn vendor_pub_key_range(vendor_ecc_pub_key_idx: u32) -> Range<u32> {
        if vendor_ecc_pub_key_idx > VENDOR_ECC_KEY_COUNT {
            return 0..0;
        }

        let pub_key_size = (2 * 4 * ECC384_SCALAR_WORD_SIZE) as u32;
        let range = Self::vendor_pub_keys_range();

        // Sanity check
        // TODO: can remove this when LMS keys are added
        if range.len() as u32 != VENDOR_ECC_KEY_COUNT * pub_key_size {
            return 0..0;
        }

        let offset = (offset_of!(ImageVendorPubKeys, ecc_pub_keys) as u32)
            + vendor_ecc_pub_key_idx * pub_key_size;

        range.start + offset..range.len() as u32 + offset
    }

    /// Returns `Range<u32>` containing the owner public key
    pub fn owner_pub_key_range() -> Range<u32> {
        let offset = offset_of!(ImageManifest, preamble) as u32;
        let span = span_of!(ImagePreamble, owner_pub_keys);
        span.start as u32 + offset..span.end as u32 + offset
    }

    /// Returns `Range<u32>` containing the header
    pub fn header_range() -> Range<u32> {
        let span = span_of!(ImageManifest, header);
        span.start as u32..span.end as u32
    }

    /// Returns `Range<u32>` containing the table of contents
    pub fn toc_range() -> Range<u32> {
        let span = span_of!(ImageManifest, fmc..=runtime);
        span.start as u32..span.end as u32
    }
}

#[repr(C)]
#[derive(AsBytes, FromBytes, Default, Debug, Clone, Copy)]
pub struct ImageVendorPubKeys {
    pub ecc_pub_keys: [ImageEccPubKey; VENDOR_ECC_KEY_COUNT as usize],
    // TODO: Add LMS Public Keys here
    // TODO: Update vendor_pub_key_range to pick up the new LMS keys
}

#[repr(C)]
#[derive(AsBytes, FromBytes, Default, Debug, Clone, Copy)]
pub struct ImageVendorPrivKeys {
    pub ecc_priv_keys: [ImageEccPrivKey; VENDOR_ECC_KEY_COUNT as usize],
    // TODO: Add LMS Private Keys here
}

#[repr(C)]
#[derive(AsBytes, FromBytes, Default, Debug, Clone, Copy)]
pub struct ImageOwnerPubKeys {
    pub ecc_pub_key: ImageEccPubKey,
    // TODO: Add LMS Public Keys here
}

#[repr(C)]
#[derive(AsBytes, FromBytes, Default, Debug)]
pub struct ImageOwnerPrivKeys {
    pub ecc_priv_key: ImageEccPrivKey,
    // TODO: Add LMS Private Keys here
}

#[repr(C)]
#[derive(AsBytes, FromBytes, Default, Debug)]
pub struct ImageSignatures {
    pub ecc_sig: ImageEccSignature,
    // TODO: Add LMS Signature here
}

/// Calipatra Image Bundle Preamble
#[repr(C)]
#[derive(AsBytes, FromBytes, Default, Debug)]
pub struct ImagePreamble {
    /// Vendor  Public Keys
    pub vendor_pub_keys: ImageVendorPubKeys,

    /// Vendor ECC Public Key Index
    pub vendor_ecc_pub_key_idx: u32,

    /// Vendor Signatures
    pub vendor_sigs: ImageSignatures,

    /// Owner Public Key
    pub owner_pub_keys: ImageOwnerPubKeys,

    /// Owner Signatures
    pub owner_sigs: ImageSignatures,

    pub _rsvd: [u32; 2],
}

#[repr(C)]
#[derive(AsBytes, FromBytes, Default, Debug)]
pub struct OwnerSignedData {
    /// Owner Start Date [ASN1 Time Format] For LDEV-Id certificate: Takes Preference over vendor start date
    pub owner_not_before: [u8; 15],

    /// Owner End Date [ASN1 Time Format] For LDEV-Id certificate: Takes Preference over vendor end date
    pub owner_not_after: [u8; 15],
}

/// Caliptra Image header
#[repr(C)]
#[derive(AsBytes, FromBytes, Default, Debug)]
pub struct ImageHeader {
    /// Revision
    pub revision: [u32; 2],

    /// Vendor ECC Public Key Index
    pub vendor_ecc_pub_key_idx: u32,

    /// Flags
    pub flags: u32,

    /// TOC Entry Count
    pub toc_len: u32,

    /// TOC Digest
    pub toc_digest: ImageDigest,

    /// Vendor Start Date [ASN1 Time Format] For LDEV-Id certificate
    pub vendor_not_before: [u8; 15],

    /// Vendor End Date [ASN1 Time Format] For LDEV-Id certificate
    pub vendor_not_after: [u8; 15],

    /// The Signed owner data
    pub owner_data: OwnerSignedData,
}

/// Caliptra table contents entry id
pub enum ImageTocEntryType {
    /// First mutable code
    Executable = 1,
}

impl From<ImageTocEntryType> for u32 {
    /// Converts to this type from the input type.
    fn from(value: ImageTocEntryType) -> Self {
        value as u32
    }
}

/// Caliptra table contents entry id
pub enum ImageTocEntryId {
    /// First mutable code
    Fmc = 1,

    /// Runtime
    Runtime = 2,
}

impl From<ImageTocEntryId> for u32 {
    /// Converts to this type from the input type.
    fn from(value: ImageTocEntryId) -> Self {
        value as u32
    }
}

/// Caliptra Table of contents entry
#[repr(C)]
#[derive(AsBytes, FromBytes, Default, Debug)]
pub struct ImageTocEntry {
    /// ID
    pub id: u32,

    /// Type
    pub r#type: u32,

    /// Commit revision
    pub revision: ImageRevision,

    /// Security Version Number
    pub svn: u32,

    /// Minimum Security Version Number
    pub min_svn: u32,

    /// Entry Point
    pub load_addr: u32,

    /// Entry Point
    pub entry_point: u32,

    /// Offset
    pub offset: u32,

    /// Size
    pub size: u32,

    /// Digest
    pub digest: ImageDigest,
}

impl ImageTocEntry {
    pub fn image_range(&self) -> Range<u32> {
        self.offset..self.offset + self.size
    }
}
