/*
 * Created on Tue Jul 11 2023
 *
 * Copyright (c) storycraft. Licensed under the Apache Licence 2.0.
 */

use std::{
    io::{self, Read, Seek, SeekFrom},
    pin::Pin,
    task::{ready, Context, Poll},
};

use chacha20::ChaCha20;
use futures::{AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt, Future};
use sha2::{Digest, Sha256};

use crate::{
    crypto::{create_cipher, SpxCipherReader},
    FileInfo, FileMap,
};

#[derive(Debug, Clone, Copy)]
pub struct SpxArchive<'a, R> {
    file_map: &'a FileMap,
    stream: R,
}

impl<'a, R> SpxArchive<'a, R> {
    pub const fn new(file_map: &'a FileMap, stream: R) -> Self {
        Self { file_map, stream }
    }

    #[inline(always)]
    fn open_raw(&self, path: &(impl AsRef<str> + ?Sized)) -> Option<(FileInfo, ChaCha20)> {
        let (hash, file) = self.file_map.get_entry(path)?;

        let key: [u8; 32] = Sha256::new()
            .chain_update(path.as_ref().as_bytes())
            .finalize()
            .into();

        Some((file, create_cipher(&key, hash)))
    }
}

impl<R: Read + Seek> SpxArchive<'_, R> {
    #[inline(always)]
    pub fn open(mut self, path: &(impl AsRef<str> + ?Sized)) -> io::Result<SpxSyncFileStream<R>> {
        let (file, cipher) = self.open_raw(path).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("File `{}` is not found", path.as_ref()),
            )
        })?;

        self.stream.seek(SeekFrom::Start(file.offset))?;

        Ok(SpxCipherReader::new(
            cipher,
            SpxRawFileStream {
                file,
                stream: self.stream.take(file.size),
            },
        ))
    }
}

impl<'a, R: AsyncRead + AsyncSeek + Unpin> SpxArchive<'a, R> {
    #[inline(always)]
    pub fn open_async(
        self,
        path: &(impl AsRef<str> + ?Sized),
    ) -> impl Future<Output = io::Result<SpxAsyncFileStream<R>>> {
        let res = self.open_raw(path).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("File `{}` is not found", path.as_ref()),
            )
        });

        let mut stream = self.stream;
        async move {
            let (file, cipher) = res?;

            stream.seek(SeekFrom::Start(file.offset)).await?;

            Ok(SpxCipherReader::new(
                cipher,
                SpxRawFileStream {
                    file,
                    stream: stream.take(file.size),
                },
            ))
        }
    }
}

pub type SpxSyncFileStream<R> = SpxFileStream<std::io::Take<R>>;
pub type SpxAsyncFileStream<R> = SpxFileStream<futures::io::Take<R>>;

type SpxFileStream<R> = SpxCipherReader<SpxRawFileStream<R>>;

#[derive(Debug)]
#[pin_project::pin_project]
pub struct SpxRawFileStream<R> {
    file: FileInfo,
    #[pin]
    stream: R,
}

impl<R> SpxRawFileStream<R> {
    fn to_absolute_seek_pos(&self, pos: SeekFrom, limit: u64) -> SeekFrom {
        match pos {
            SeekFrom::Start(offset) => {
                SeekFrom::Start(self.file.offset + offset.min(self.file.size))
            }

            SeekFrom::End(offset) => {
                SeekFrom::Start(self.file.offset + (self.file.size as i64 + offset).max(0) as u64)
            }

            SeekFrom::Current(offset) => {
                let limit = limit as i64;

                SeekFrom::Current(offset.clamp(limit, limit - self.file.size as i64))
            }
        }
    }
}

impl<R: Read> Read for SpxRawFileStream<std::io::Take<R>> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }
}

impl<R: Seek> Seek for SpxRawFileStream<std::io::Take<R>> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        // it's not really expensive so it would be ok... just marking for future
        let pos = self.to_absolute_seek_pos(pos, self.stream.limit());

        let absolute_pos = self.stream.get_mut().seek(pos)?;

        let file_pos = absolute_pos - self.file.offset;
        self.stream.set_limit(self.file.size - file_pos);

        Ok(file_pos)
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for SpxRawFileStream<futures::io::Take<R>> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        self.project().stream.poll_read(cx, buf)
    }
}

impl<R: AsyncRead + AsyncSeek + Unpin> AsyncSeek for SpxRawFileStream<futures::io::Take<R>> {
    fn poll_seek(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        pos: SeekFrom,
    ) -> Poll<io::Result<u64>> {
        let pos = self.to_absolute_seek_pos(pos, self.stream.limit());

        let mut this = self.project();

        let absolute_pos = ready!(this.stream.as_mut().get_pin_mut().poll_seek(cx, pos)?);

        let file_pos = absolute_pos - this.file.offset;

        let limit = this.file.size - file_pos;
        this.stream.set_limit(limit);

        Poll::Ready(Ok(file_pos))
    }
}
