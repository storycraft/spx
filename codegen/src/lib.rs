/*
 * Created on Tue Jul 11 2023
 *
 * Copyright (c) storycraft. Licensed under the Apache Licence 2.0.
 */

pub mod ext;

use core::fmt;
use std::io::{self, Write};

use const_fnv1a_hash::fnv1a_hash_str_32;
use phf_generator::{generate_hash, HashState};
use spx::FileInfo;

#[derive(Debug)]
pub struct SpxBuilder<W> {
    writer: W,
    keys: Vec<String>,
    values: Vec<(u32, FileInfo)>,
}

impl<W: Write> SpxBuilder<W> {
    pub const fn new(writer: W) -> Self {
        Self {
            writer,
            keys: Vec::new(),
            values: Vec::new(),
        }
    }

    pub fn start_file(&mut self, name: String) -> SpxFileEntry<'_, W> {
        let hash = fnv1a_hash_str_32(&name);

        let pos = self
            .values
            .last()
            .and_then(|info| Some(info.1.offset + info.1.size))
            .unwrap_or(0);

        dbg!(&name);

        self.keys.push(name);
        self.values.push((hash, FileInfo::new(pos, 0)));

        SpxFileEntry {
            writer: &mut self.writer,
            info: &mut self.values.last_mut().unwrap().1,
        }
    }

    pub fn build(&self) -> Display {
        let state = generate_hash(&self.keys);

        Display {
            state,
            values: &self.values,
        }
    }
}

#[derive(Debug)]
pub struct SpxFileEntry<'a, W> {
    writer: &'a mut W,
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
        Ok(())
    }
}

pub struct Display<'a> {
    state: HashState,
    values: &'a [(u32, FileInfo)],
}

impl core::fmt::Display for Display<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "::spx::FileMap {{ key: {}_u64, disps: &[",
            self.state.key
        )?;

        for disp in &self.state.disps {
            write!(f, "({}, {}), ", disp.0, disp.1)?;
        }
        write!(f, "], values: &[")?;

        for index in &self.state.map {
            let value = &self.values[*index];
            write!(
                f,
                "({}, ::spx::FileInfo::new({}, {})), ",
                value.0, value.1.offset, value.1.size
            )?;
        }
        write!(f, "] }};")?;

        Ok(())
    }
}
