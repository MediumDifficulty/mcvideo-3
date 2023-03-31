use std::{fs::{File, self}, path::Path, io::Write};

use zip::{ZipWriter, write::FileOptions, result::ZipResult};

pub fn create(audio: &[u8]) -> ZipResult<Vec<u8>> {
    let path = Path::new("audio.zip");
    let file = File::create(path).unwrap();
    let mut zip = ZipWriter::new(file);

    let options = FileOptions::default();

    zip.start_file("pack.mcmeta", options)?;
    zip.write_all(include_bytes!("assets/pack.mcmeta"))?;

    zip.add_directory("assets/audio/sounds", options)?;

    zip.start_file("assets/audio/sounds.json", options)?;
    zip.write_all(include_bytes!("assets/sounds.json"))?;

    zip.start_file("assets/audio/sounds/audio.ogg", options)?;
    zip.write_all(audio)?;

    zip.finish()?;

    let bytes = fs::read(path).unwrap();
    fs::remove_file(path).unwrap();

    Ok(bytes)
}