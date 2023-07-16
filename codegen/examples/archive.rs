/*
 * Created on Sun Jul 16 2023
 *
 * Copyright (c) storycraft. Licensed under the Apache Licence 2.0.
 */

use std::{error::Error, io::Cursor};

use spx_codegen::{ext::StreamExt, SpxBuilder};

fn main() -> Result<(), Box<dyn Error>> {
    let mut data = Vec::new();
    let mut builder = SpxBuilder::new(Cursor::new(&mut data));

    builder.write_stream("hello world.txt".into(), Cursor::new(&"Hello world!"))?;
    builder.write_stream("example".into(), Cursor::new(&"asdf"))?;

    println!("map: {}", builder.build());
    println!("archive: {:?}", data);

    Ok(())
}
