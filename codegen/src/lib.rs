/*
 * Created on Tue Jul 11 2023
 *
 * Copyright (c) storycraft. Licensed under the Apache Licence 2.0.
 */

#![doc = include_str!("../README.md")]

pub mod ext;

use core::fmt;
use std::{
    collections::HashSet,
    io::{self, Seek, Write},
};

use const_fnv1a_hash::fnv1a_hash_str_64;
use phf_generator::{generate_hash, HashState};
use sha2::{Digest, Sha256};
use spx::{
    crypto::SpxCipherStream,
    map::{OffsetKey, SizeKey},
    FileInfo,
};

#[derive(Debug)]
/// Compile-time [`FileMap`] builder
pub struct SpxBuilder<W> {
    writer: W,
    key_set: HashSet<String>,
    keys: Vec<String>,
    values: Vec<(u64, FileInfo)>,
}

impl<W: Write + Seek> SpxBuilder<W> {
    /// Create new builder
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            key_set: HashSet::new(),
            keys: Vec::new(),
            values: Vec::new(),
        }
    }

    /// Start new file entry
    pub fn start_file(&mut self, name: String) -> io::Result<SpxFileEntry<'_, W>> {
        let hash = fnv1a_hash_str_64(&name);

        let pos = self.writer.stream_position()?;

        let key: [u8; 32] = Sha256::new()
            .chain_update(name.as_bytes())
            .finalize()
            .into();

        if self.key_set.insert(name.clone()) {
            panic!("duplicate key `{}`", &name);
        }

        self.keys.push(name);

        self.values.push((hash, FileInfo::new(pos, 0)));

        Ok(SpxFileEntry {
            writer: SpxCipherStream::new(&key, hash, &mut self.writer),
            info: &mut self.values.last_mut().unwrap().1,
        })
    }

    /// Generate and return [`FileMap`] code
    pub fn build(&self) -> Display {
        let offset_state = generate_hash(
            &self
                .keys
                .iter()
                .map(|key| OffsetKey(key))
                .collect::<Vec<_>>(),
        );

        let size_state =
            generate_hash(&self.keys.iter().map(|key| SizeKey(key)).collect::<Vec<_>>());

        Display {
            offset_state,
            size_state,
            values: &self.values,
        }
    }
}

pub struct SpxFileEntry<'a, W> {
    writer: SpxCipherStream<&'a mut W>,
    info: &'a mut FileInfo,
}

impl<W> SpxFileEntry<'_, W> {
    pub fn finish(self) -> FileInfo {
        *self.info
    }
}

impl<W: Write> Write for SpxFileEntry<'_, W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let written = self.writer.write(buf)?;

        self.info.size += written as u64;

        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

pub struct Display<'a> {
    offset_state: HashState,
    size_state: HashState,

    values: &'a [(u64, FileInfo)],
}

impl Display<'_> {
    fn fmt_lookup_map(
        state: &HashState,
        value_fn: impl Fn(usize) -> (u32, u64),
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        write!(
            f,
            "::spx::map::LookupMap {{ key: {}_u64, disps: &[",
            state.key
        )?;

        for disp in &state.disps {
            write!(f, "({}, {}), ", disp.0, disp.1)?;
        }
        write!(f, "], values: &[")?;

        for &index in &state.map {
            let value = value_fn(index);
            write!(f, "({}, {}), ", value.0, value.1)?;
        }
        write!(f, "] }}")?;

        Ok(())
    }
}

impl core::fmt::Display for Display<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "::spx::FileMap::from_maps(&")?;
        Self::fmt_lookup_map(
            &self.offset_state,
            |index| {
                let value = self.values[index];
                ((value.0 >> 32) as u32, value.1.offset)
            },
            f,
        )?;

        write!(f, ", &")?;
        Self::fmt_lookup_map(
            &self.size_state,
            |index| {
                let value = self.values[index];
                (value.0 as u32, value.1.size)
            },
            f,
        )?;
        write!(f, ")")?;

        Ok(())
    }
}
