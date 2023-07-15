use std::{error::Error, io::{Cursor, Read}};

use spx::{io::SpxArchive, FileInfo};

#[test]
fn map_test() {
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

    assert_eq!(*map.get("Cargo.toml").unwrap(), FileInfo::new(0, 454));
}

#[test]
fn stream_test() -> Result<(), Box<dyn Error>> {
    let archive = [195_u8, 205, 125, 46, 222, 58, 38, 131, 10, 247, 71, 226];

    // generated code
    let map = ::spx::FileMap {
        key: 12913932095322966823_u64,
        disps: &[(0, 0)],
        values: &[(2454477377, ::spx::FileInfo::new(0, 12))],
    };

    let mut archive = SpxArchive::new(map, Cursor::new(&archive));

    let mut stream = archive.open("hello world.txt").unwrap()?;

    let mut output = String::new();
    stream.read_to_string(&mut output)?;

    assert_eq!(output, "Hello world!");

    Ok(())
}
