/*
 * Created on Sun Jul 16 2023
 *
 * Copyright (c) storycraft. Licensed under the Apache Licence 2.0.
 */

use phf_shared::{get_index, hash, HashKey, PhfHash};

#[derive(Debug)]
pub struct LookupMap {
    #[doc(hidden)]
    pub key: HashKey,
    #[doc(hidden)]
    pub disps: &'static [(u32, u32)],
    #[doc(hidden)]
    pub values: &'static [(u32, u64)],
}

impl LookupMap {
    /// Create new empty [`LookupMap`]
    pub const fn new() -> Self {
        Self {
            key: 0,
            disps: &[],
            values: &[],
        }
    }

    #[inline(always)]
    pub fn get_raw<T: ?Sized + PhfHash>(&self, x: &T) -> (u32, u64) {
        self.values[get_index(&hash(x, &self.key), self.disps, self.values.len()) as usize]
    }
}

impl Default for LookupMap {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SizeKey<'a>(pub &'a str);
impl PhfHash for SizeKey<'_> {
    fn phf_hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.phf_hash(state);
        ":size".phf_hash(state);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OffsetKey<'a>(pub &'a str);
impl PhfHash for OffsetKey<'_> {
    fn phf_hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.phf_hash(state);
        ":offset".phf_hash(state);
    }
}
