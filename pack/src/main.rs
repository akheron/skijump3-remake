use std::fs::{read_dir, File};
use std::io::{Read, Write};

const ASSET_DIR: &str = "assets";

/// Pack file format:
///
/// u8: number of files
/// header -- for each file:
///   u8: length of file name
///   [u8]: file name
///   u32: file size
///   u32: file data offset from the start of data section
/// data -- for each file:
///   [u8]: file data
///
fn main() {
    let mut files = Vec::new();
    read_dir(ASSET_DIR).unwrap().for_each(|entry| {
        let entry = entry.unwrap();
        let file_name = entry.file_name().into_string().unwrap();
        let file_size = entry.metadata().unwrap().len() as u32;
        files.push((file_name, file_size));
    });

    let mut f = File::create("assets.pack").unwrap();
    f.write_all(&(files.len() as u8).to_le_bytes()).unwrap();
    let mut offset: u32 = 0;
    for (file_name, file_size) in &files {
        let file_name_bytes = file_name.as_bytes();
        f.write_all(&(file_name_bytes.len() as u8).to_le_bytes())
            .unwrap();
        f.write_all(file_name_bytes).unwrap();
        f.write_all(&file_size.to_le_bytes()).unwrap();
        f.write_all(&offset.to_le_bytes()).unwrap();
        offset += file_size;
    }
    for (file_name, _) in &files {
        let mut file = File::open(format!("{}/{}", ASSET_DIR, file_name)).unwrap();
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();
        f.write_all(&data).unwrap();
    }
}
