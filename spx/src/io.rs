/*
 * Created on Tue Jul 11 2023
 *
 * Copyright (c) storycraft. Licensed under the Apache Licence 2.0.
 */

use std::io::{self, Read, Seek, SeekFrom, Take};

use sha2::{Digest, Sha256};

use crate::{crypto::SpxCipherStream, FileInfo, FileMap};

#[derive(Debug)]
pub struct SpxArchive<'a, R> {
    file_map: &'a FileMap,
    stream: R,
}

impl<'a, R> SpxArchive<'a, R> {
    pub const fn new(file_map: &'a FileMap, stream: R) -> Self {
        Self { file_map, stream }
    }
}

impl<R: Read + Seek> SpxArchive<'_, R> {
    #[inline(always)]
    pub fn open(&mut self, path: &(impl AsRef<str> + ?Sized)) -> io::Result<SpxFileStream<&mut R>> {
        let (hash, file) = self.file_map.get_entry(path).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("File `{}` is not found", path.as_ref()),
            )
        })?;

        let key: [u8; 32] = Sha256::new()
            .chain_update(path.as_ref().as_bytes())
            .finalize()
            .into();

        self.stream.seek(SeekFrom::Start(file.offset))?;

        Ok(SpxCipherStream::new(
            &key,
            hash,
            SpxRawFileStream {
                file,
                stream: (&mut self.stream).take(file.size),
            },
        ))
    }
}

pub type SpxFileStream<R> = SpxCipherStream<SpxRawFileStream<R>>;

pub struct SpxRawFileStream<R> {
    file: FileInfo,
    stream: Take<R>,
}

impl<R: Read> Read for SpxRawFileStream<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }
}

impl<R: Seek> Seek for SpxRawFileStream<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let absolute_pos = match pos {
            SeekFrom::Start(offset) => self.stream.get_mut().seek(SeekFrom::Start(
                self.file.offset + offset.min(self.file.size),
            ))?,

            SeekFrom::End(offset) => self.stream.get_mut().seek(SeekFrom::Start(
                self.file.offset + (self.file.size as i64 + offset).max(0) as u64,
            ))?,

            SeekFrom::Current(offset) => {
                let limit = self.stream.limit() as i64;

                self.stream.get_mut().seek(SeekFrom::Current(
                    offset.min(limit).max(limit - self.file.size as i64),
                ))?
            }
        };

        let file_pos = absolute_pos - self.file.offset;
        self.stream.set_limit(self.file.size - file_pos);

        Ok(file_pos)
    }
}
