/*
 * Created on Tue Jul 11 2023
 *
 * Copyright (c) storycraft. Licensed under the Apache Licence 2.0.
 */

pub mod io;

use const_fnv1a_hash::fnv1a_hash_str_32;
use phf_shared::HashKey;

#[derive(Debug)]
pub struct FileMap<'a> {
    #[doc(hidden)]
    pub key: HashKey,
    #[doc(hidden)]
    pub disps: &'a [(u32, u32)],
    #[doc(hidden)]
    pub values: &'a [(u32, FileInfo)],
}

impl<'a> FileMap<'a> {
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

    pub fn get(&self, path: &(impl AsRef<str> + ?Sized)) -> Option<&'a FileInfo> {
        if self.disps.is_empty() {
            return None;
        }

        let path = path.as_ref();
        let hashes = phf_shared::hash(path, &self.key);
        let index = phf_shared::get_index(&hashes, self.disps, self.values.len());

        let value = &self.values[index as usize];
        if value.0 == fnv1a_hash_str_32(path) {
            Some(&value.1)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct FileInfo {
    pub offset: u64,
    pub size: u64,
}

impl FileInfo {
    pub const fn new(offset: u64, size: u64) -> Self {
        Self { offset, size }
    }
}

#[macro_export]
macro_rules! spx_archive {
    ($path: literal) => {};
}
