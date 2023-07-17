/*
 * Created on Sat Jul 15 2023
 *
 * Copyright (c) storycraft. Licensed under the Apache Licence 2.0.
 */

use std::{
    io::{self, Read, Seek, SeekFrom, Write},
    pin::Pin,
    task::{ready, Context, Poll},
};

use chacha20::{
    cipher::{KeyIvInit, StreamCipher, StreamCipherSeek},
    ChaCha20,
};
use futures::{AsyncRead, AsyncSeek};

pub fn create_cipher(key: &[u8; 32], hash: u64) -> ChaCha20 {
    ChaCha20::new(
        key.into(),
        &{
            let mut arr = [0_u8; 12];
            arr[4..].copy_from_slice(&hash.to_le_bytes());
            arr
        }
        .into(),
    )
}

#[pin_project::pin_project]
pub struct SpxCipherReader<R> {
    cipher: ChaCha20,
    buffer: [u8; 8192],
    #[pin]
    reader: R,
}

impl<R> SpxCipherReader<R> {
    pub const fn new(cipher: ChaCha20, reader: R) -> Self {
        Self {
            cipher,
            buffer: [0_u8; 8192],
            reader,
        }
    }

    pub const fn inner(&self) -> &R {
        &self.reader
    }

    pub fn inner_mut(&mut self) -> &mut R {
        &mut self.reader
    }

    pub fn into_inner(self) -> R {
        self.reader
    }
}

impl<R: Read> Read for SpxCipherReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let read = {
            let len = self.buffer.len().min(buf.len());

            self.reader.read(&mut self.buffer[..len])
        }?;

        self.cipher
            .apply_keystream_b2b(&self.buffer[..read], &mut buf[..read])
            .unwrap();

        Ok(read)
    }
}

impl<R: Seek> Seek for SpxCipherReader<R> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let pos = self.reader.seek(pos)?;
        self.cipher.seek(pos);

        Ok(pos)
    }
}

impl<R: AsyncRead> AsyncRead for SpxCipherReader<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.project();
        let read = ready!({
            let len = this.buffer.len().min(buf.len());

            this.reader.poll_read(cx, &mut this.buffer[..len])
        }?);

        this.cipher
            .apply_keystream_b2b(&this.buffer[..read], &mut buf[..read])
            .unwrap();

        Poll::Ready(Ok(read))
    }
}

impl<R: AsyncSeek> AsyncSeek for SpxCipherReader<R> {
    fn poll_seek(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        pos: SeekFrom,
    ) -> Poll<io::Result<u64>> {
        let this = self.project();

        let pos = ready!(this.reader.poll_seek(cx, pos)?);
        this.cipher.seek(pos);

        Poll::Ready(Ok(pos))
    }
}

pub struct SpxCipherWriter<W> {
    cipher: ChaCha20,
    buffer: [u8; 8192],
    writer: W,
}

impl<W> SpxCipherWriter<W> {
    pub const fn new(cipher: ChaCha20, writer: W) -> Self {
        Self {
            cipher,
            buffer: [0_u8; 8192],
            writer,
        }
    }

    pub const fn inner(&self) -> &W {
        &self.writer
    }

    pub fn inner_mut(&mut self) -> &mut W {
        &mut self.writer
    }

    pub fn into_inner(self) -> W {
        self.writer
    }
}

impl<W: Write> Write for SpxCipherWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let size = self.buffer.len().min(buf.len());

        self.cipher
            .apply_keystream_b2b(&buf[..size], &mut self.buffer[..size])
            .unwrap();

        self.writer.write(&self.buffer[..size])
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}
