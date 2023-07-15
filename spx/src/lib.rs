/*
 * Created on Tue Jul 11 2023
 *
 * Copyright (c) storycraft. Licensed under the Apache Licence 2.0.
 */

#![doc = include_str!("../README.md")]

pub mod io;
pub mod crypto;

use const_fnv1a_hash::fnv1a_hash_str_32;
use phf_shared::{HashKey, Hashes};

#[derive(Debug)]
/// File mapping of a spx archive file. It is built in compile-time with perfect hash table, with original key vanished.
/// 
/// Each entry contains tuple of  fnv1a hash of actual file name and [`FileInfo`] object.
pub struct FileMap<'a> {
    #[doc(hidden)]
    pub key: HashKey,
    #[doc(hidden)]
    pub disps: &'a [(u32, u32)],
    #[doc(hidden)]
    pub values: &'a [(u32, FileInfo)],
}

impl<'a> FileMap<'a> {
    /// Create new empty [`FileMap`]
    pub const fn new() -> Self {
        Self {
            key: 0,
            disps: &[],
            values: &[],
        }
    }

    pub const fn len(&self) -> usize {
        self.values.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline(always)]
    pub fn get(&self, path: &(impl AsRef<str> + ?Sized)) -> Option<&'a FileInfo> {
        self.get_entry(path).map(|(_, info)| info)
    }

    #[inline(always)]
    pub fn get_entry(&self, path: &(impl AsRef<str> + ?Sized)) -> Option<&'a (u32, FileInfo)> {
        if self.disps.is_empty() {
            None
        } else {
            let path = path.as_ref();
            self.get_internal(phf_shared::hash(path, &self.key), fnv1a_hash_str_32(path))
        }
    }

    fn get_internal(&self, map_hash: Hashes, fnv1a_hash: u32) -> Option<&'a (u32, FileInfo)> {
        let index = phf_shared::get_index(&map_hash, self.disps, self.values.len());

        let value = &self.values[index as usize];
        if value.0 == fnv1a_hash {
            Some(&value)
        } else {
            None
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
