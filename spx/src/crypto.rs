/*
 * Created on Sat Jul 15 2023
 *
 * Copyright (c) storycraft. Licensed under the Apache Licence 2.0.
 */

use std::io::{self, Read, Seek, SeekFrom, Write};

use chacha20::{
    cipher::{KeyIvInit, StreamCipher, StreamCipherSeek},
    ChaCha20,
};

pub struct SpxCipherStream<S> {
    cipher: ChaCha20,
    buffer: [u8; 8192],
    stream: S,
}

impl<S> SpxCipherStream<S> {
    pub fn new(key: &[u8; 32], hash: u64, stream: S) -> Self {
        Self {
            cipher: ChaCha20::new(
                key.into(),
                &{
                    let mut arr = [0_u8; 12];
                    arr[4..].copy_from_slice(&hash.to_le_bytes());
                    arr
                }
                .into(),
            ),
            buffer: [0_u8; 8192],
            stream,
        }
    }

    pub const fn inner(&self) -> &S {
        &self.stream
    }

    pub fn inner_mut(&mut self) -> &mut S {
        &mut self.stream
    }

    pub fn into_inner(self) -> S {
        self.stream
    }
}

impl<S: Read> Read for SpxCipherStream<S> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let read = {
            let len = self.buffer.len().min(buf.len());

            self.stream.read(&mut self.buffer[..len])
        }?;

        self.cipher
            .apply_keystream_b2b(&self.buffer[..read], &mut buf[..read])
            .unwrap();

        Ok(read)
    }
}

impl<S: Seek> Seek for SpxCipherStream<S> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let pos = self.stream.seek(pos)?;
        self.cipher.seek(pos);

        Ok(pos)
    }
}

impl<S: Write> Write for SpxCipherStream<S> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let size = self.buffer.len().min(buf.len());

        self.cipher
            .apply_keystream_b2b(&buf[..size], &mut self.buffer[..size])
            .unwrap();

        self.stream.write(&mut self.buffer[..size])
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}
