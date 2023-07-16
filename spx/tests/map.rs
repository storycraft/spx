use std::{
    error::Error,
    io::{Cursor, Read},
};

use spx::io::SpxArchive;

#[test]
fn stream_test() -> Result<(), Box<dyn Error>> {
    let archive = [
        98, 230, 205, 129, 245, 38, 142, 205, 232, 219, 234, 162, 51, 194, 242, 194,
    ];

    // generated code
    let map = ::spx::FileMap::from_maps(
        &::spx::map::LookupMap {
            key: 12913932095322966823_u64,
            disps: &[(1, 0)],
            values: &[(1124799619, 12), (1613200686, 0)],
        },
        &::spx::map::LookupMap {
            key: 12913932095322966823_u64,
            disps: &[(0, 0)],
            values: &[(3369492545, 4), (1370935553, 12)],
        },
    );

    let mut archive = SpxArchive::new(map, Cursor::new(&archive));

    {
        let mut stream = archive.open("hello world.txt").unwrap()?;

        let mut output = String::new();
        stream.read_to_string(&mut output)?;

        assert_eq!(output, "Hello world!");
    }

    {
        let mut stream = archive.open("example").unwrap()?;

        let mut output = String::new();
        stream.read_to_string(&mut output)?;

        assert_eq!(output, "asdf");
    }

    Ok(())
}
