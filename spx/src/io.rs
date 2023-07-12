/*
 * Created on Tue Jul 11 2023
 *
 * Copyright (c) storycraft. Licensed under the Apache Licence 2.0.
 */

use std::io::{self, Read, Seek, SeekFrom, Take};

use crate::{FileInfo, FileMap};

#[derive(Debug)]
pub struct ArchiveStream<'a, R> {
    file_map: FileMap<'a>,
    stream: R,
}

impl<'a, R> ArchiveStream<'a, R> {
    pub const fn new(file_map: FileMap<'a>, stream: R) -> Self {
        Self { file_map, stream }
    }
}

impl<R: Read + Seek> ArchiveStream<'_, R> {
    pub fn open(
        &mut self,
        path: &(impl AsRef<str> + ?Sized),
    ) -> io::Result<Option<FileStream<&mut R>>> {
        let file = match self.file_map.get(path) {
            Some(file) => *file,
            None => return Ok(None),
        };

        self.stream.seek(SeekFrom::Start(file.offset))?;
        Ok(Some(FileStream {
            file,
            stream: (&mut self.stream).take(file.size),
        }))
    }
}

#[derive(Debug)]
pub struct FileStream<R> {
    file: FileInfo,
    stream: Take<R>,
}

impl<R: Read> Read for FileStream<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        Ok(self.stream.read(buf)?)
    }
}

impl<R: Seek> Seek for FileStream<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let absolute_pos = match pos {
            SeekFrom::Start(offset) => {
                self.stream.get_mut().seek(SeekFrom::Start(
                    self.file.offset + offset.min(self.file.size),
                ))? - self.file.offset
            }
            SeekFrom::End(offset) => self.stream.get_mut().seek(SeekFrom::Start(
                self.file.offset + (self.file.size as i64 - offset).max(0) as u64,
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

        return Ok(file_pos);
    }
}
