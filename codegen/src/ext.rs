/*
 * Created on Wed Jul 12 2023
 *
 * Copyright (c) storycraft. Licensed under the Apache Licence 2.0.
 */

use std::{
    fs::File,
    io::{self, BufReader, Read, Seek, Write},
    path::Path,
};

use easy_ext::ext;
use path_slash::PathExt;
use walkdir::WalkDir;

use crate::SpxBuilder;

#[ext(StreamExt)]
pub impl<W: Write + Seek> SpxBuilder<W> {
    /// Write a file entry from [`Read`] stream
    fn write_stream(&mut self, name: String, mut stream: impl Read) -> io::Result<u64> {
        io::copy(&mut stream, &mut self.start_file(name)?)
    }
}

#[ext(FileExt)]
pub impl<W: Write + Seek> SpxBuilder<W> {

    /// Write every disk files from base directory
    fn from_dir(&mut self, base: impl AsRef<Path>) -> io::Result<u64> {
        let mut count = 0;

        for entry in WalkDir::new(&base) {
            let entry = entry?;
            if entry.file_type().is_dir() {
                continue;
            }

            let path = entry.path();
            let striped_path = entry.path().strip_prefix(&base).unwrap();

            self.write_stream(
                striped_path.to_slash_lossy().to_string(),
                BufReader::new(File::open(path)?),
            )?;
            count += 1;
        }

        Ok(count)
    }
}
