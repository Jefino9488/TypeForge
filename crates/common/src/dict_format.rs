use bytemuck::{Pod, Zeroable};
use std::mem::size_of;

pub const MAGIC_NUMBER: [u8; 8] = *b"TYPEDICT";
pub const CURRENT_VERSION: u32 = 1;
pub const LANG_EN: u16 = 0;

/// Header flags
pub const FLAG_COMPRESSED: u32 = 1 << 0;
pub const FLAG_STEMMING: u32 = 1 << 1;
pub const FLAG_POS_TAGS: u32 = 1 << 2;
pub const FLAG_UTF8_NORMALIZED: u32 = 1 << 3;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct DictionaryHeader {
    pub magic: [u8; 8],
    pub version: u32,
    pub flags: u32,
    pub language: u16,
    pub reserved: u16,
    pub word_count: u32,
    pub index_offset: u64,
    pub strings_offset: u64,
    pub checksum_offset: u64,
}

impl Default for DictionaryHeader {
    fn default() -> Self {
        Self {
            magic: MAGIC_NUMBER,
            version: CURRENT_VERSION,
            flags: 0,
            language: LANG_EN,
            reserved: 0,
            word_count: 0,
            index_offset: 0,
            strings_offset: 0,
            checksum_offset: 0,
        }
    }
}

/// The alpha index stores the offset into the `DictionaryEntry` array
/// for the first word starting with a specific letter (a-z).
/// Array size is 26 * sizeof(u32).
pub type AlphaIndex = [u32; 26];

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct DictionaryEntry {
    pub offset: u32,
    pub length: u16,
    pub first_char: u16,
    pub frequency: u32,
}

// Ensure the structs have the expected sizes so we don't get surprises across architectures.
const _: () = assert!(size_of::<DictionaryHeader>() == 48);
const _: () = assert!(size_of::<DictionaryEntry>() == 12);
const _: () = assert!(size_of::<AlphaIndex>() == 104);
