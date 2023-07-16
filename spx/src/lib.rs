/*
 * Created on Tue Jul 11 2023
 *
 * Copyright (c) storycraft. Licensed under the Apache Licence 2.0.
 */

#![doc = include_str!("../README.md")]

pub mod crypto;
pub mod io;
pub mod map;

use const_fnv1a_hash::fnv1a_hash_str_64;
use map::{LookupMap, OffsetKey, SizeKey};

#[derive(Debug, Clone, Copy)]
/// File mapping of a spx archive file. It is built in compile-time with perfect hash table, with original key vanished.
///
/// Each entry contains tuple of  fnv1a hash of actual file name and [`FileInfo`] object.
pub struct FileMap {
    #[doc(hidden)]
    offsets: &'static LookupMap,
    #[doc(hidden)]
    sizes: &'static LookupMap,
}

impl FileMap {
    /// Create new empty [`FileMap`]
    pub const fn new() -> Self {
        const EMPTY: LookupMap = LookupMap::new();

        Self {
            offsets: &EMPTY,
            sizes: &EMPTY,
        }
    }

    pub const fn from_maps(offsets: &'static LookupMap, sizes: &'static LookupMap) -> Self {
        Self { offsets, sizes }
    }

    pub const fn len(&self) -> usize {
        self.offsets.disps.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline(always)]
    pub fn get(&self, path: &(impl AsRef<str> + ?Sized)) -> Option<FileInfo> {
        self.get_entry(path).map(|(_, info)| info)
    }

    #[inline(always)]
    pub fn get_entry(&self, path: &(impl AsRef<str> + ?Sized)) -> Option<(u64, FileInfo)> {
        if self.is_empty() {
            None
        } else {
            let path = path.as_ref();
            let hash = fnv1a_hash_str_64(path);

            let (high, offset) = self.offsets.get_raw(&OffsetKey(path));
            let (low, size) = self.sizes.get_raw(&SizeKey(path));

            if (high as u64) << 32 | low as u64 == hash {
                Some((hash, FileInfo::new(offset, size)))
            } else {
                None
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct FileInfo {
    /// Absolute offset from archive file
    pub offset: u64,

    /// Size of actual file
    pub size: u64,
}

impl FileInfo {
    pub const fn new(offset: u64, size: u64) -> Self {
        Self { offset, size }
    }
}
