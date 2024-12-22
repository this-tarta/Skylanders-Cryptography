use std::{fs, io::{self, Read, Seek, Write}, path};

mod skyutils;

fn main() {
    new_skylander_file(path::Path::new("../Skylanders/test3.sky")).expect("new sky failure");
    encryption_skylander_file(path::Path::new("../Skylanders/test.sky"),
                           path::Path::new("../Skylanders/Ignitor.sky"), false)
        .expect("file failure");
    calculate_checksums_file(path::Path::new("../Skylanders/test.sky")).expect("file failure");
    encryption_skylander_file(path::Path::new("../Skylanders/test2.sky"),
                           path::Path::new("../Skylanders/test.sky"), true)
        .expect("file failure");
}

fn new_skylander_file(file_name: &path::Path) -> io::Result<()> {
    let mut fd= fs::File::create(file_name)?;
    let data = skyutils::new_skylander(0x13, 0x2805, Some([0x0F, 0xF0, 0x3C, 0x11]));
    fd.write_all(&data)?;

    Ok(())
}

fn encryption_skylander_file(dst: &path::Path, src: &path::Path, encrypt: bool) -> io::Result<()> {
    let mut fd_src = fs::File::open(src)?;
    let mut fd_dst = fs::File::create(dst)?;
    let mut data = [0u8; skyutils::NUM_BYTES];
    fd_src.read_exact(&mut data)?;
    skyutils::encryption_skylander(&mut data, encrypt);
    fd_dst.write_all(&data)?;

    Ok(())
}

fn calculate_checksums_file(path: &path::Path) -> io::Result<()> {
    let mut fd = fs::File::options().read(true).write(true).open(path)?;
    let mut data = [0u8; skyutils::NUM_BYTES];
    fd.read_exact(&mut data)?;
    skyutils::calculate_checksums(&mut data);
    fd.seek(io::SeekFrom::Start(0))?;
    fd.write_all(&data)?;

    Ok(())
}