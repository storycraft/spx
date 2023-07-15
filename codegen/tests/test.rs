use std::{error::Error, io::Cursor};

use spx_codegen::{ext::StreamExt, SpxBuilder};

#[test]
fn codegen_test() -> Result<(), Box<dyn Error>> {
    let mut data = Vec::new();
    let mut builder = SpxBuilder::new(Cursor::new(&mut data));

    builder.write_stream("hello world.txt".into(), Cursor::new(&"Hello world!"))?;

    println!("{}", builder.build());

    println!("{:?}", data);

    Ok(())
}
