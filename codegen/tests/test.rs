use std::{error::Error, io::Cursor};

use spx_codegen::{ext::FileExt, SpxBuilder};

#[test]
fn test() -> Result<(), Box<dyn Error>> {
    let mut data = Vec::new();
    let mut builder = SpxBuilder::new(Cursor::new(&mut data));

    builder.from_dir("./")?;

    println!("{}", builder.build());

    std::fs::write("../archive.spx", data).unwrap();

    Ok(())
}

#[test]
fn tests2() {
    let map = ::spx::FileMap {
        key: 12913932095322966823_u64,
        disps: &[(1, 0)],
        values: &[
            (2933750114, ::spx::FileInfo::new(454, 1156)),
            (4177863687, ::spx::FileInfo::new(0, 454)),
            (3011276538, ::spx::FileInfo::new(1610, 2567)),
            (1509948260, ::spx::FileInfo::new(4177, 325)),
        ],
    };

    println!("{:?}", map.get("Cargo.toml"))
}
