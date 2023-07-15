use std::{
    fs::File,
    io::{self, BufReader, Seek},
};

use spx::{io::SpxArchive, FileMap};

#[test]
fn map() {
    let map = FileMap {
        key: 12913932095322966823_u64,
        disps: &[(1, 0)],
        values: &[
            (2933750114, ::spx::FileInfo::new(460, 1156)),
            (4177863687, ::spx::FileInfo::new(0, 460)),
            (3011276538, ::spx::FileInfo::new(1616, 2631)),
            (1509948260, ::spx::FileInfo::new(4247, 834)),
        ],
    };

    let mut archive = SpxArchive::new(map, BufReader::new(File::open("../archive.spx").unwrap()));
    let mut stream = archive.open("src/lib.rs").unwrap().unwrap();

    stream.seek(io::SeekFrom::Current(50)).unwrap();

    io::copy(&mut stream, &mut io::stdout()).unwrap();
}
