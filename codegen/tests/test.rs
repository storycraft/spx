use std::{error::Error, io::Cursor};

use spx_codegen::{ext::FileExt, SpxBuilder};

#[test]
fn test() -> Result<(), Box<dyn Error>> {
    let mut data = Vec::new();
    let mut builder = SpxBuilder::new(Cursor::new(&mut data));

    builder.from_dir("./")?;

    println!("{}", builder.build());

    Ok(())
}
