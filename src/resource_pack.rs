use std::{fs::{File, self}, path::Path, io::Write};

use log::info;
use zip::{ZipWriter, write::FileOptions, result::ZipResult};

pub fn create(audio: &[Vec<u8>]) -> ZipResult<Vec<u8>> {
    let path = Path::new("audio.zip");
    let file = File::create(path).unwrap();
    let mut zip = ZipWriter::new(file);

    let options = FileOptions::default();

    zip.start_file("pack.mcmeta", options)?;
    zip.write_all(include_bytes!("assets/pack.mcmeta"))?;

    zip.add_directory("assets/audio/sounds", options)?;

    let mut sounds_json = String::from("{");
    for (i, file) in audio.iter().enumerate() {
        zip.start_file(format!("assets/audio/sounds/{}.ogg", i), options)?;
        zip.write_all(file)?;

        sounds_json.push_str(format!("\"{}\":{{\"sounds\":[\"audio:{}\"]}},", i, i).as_str())
    }

    sounds_json.pop();
    sounds_json.push('}');
    zip.start_file("assets/audio/sounds.json", options)?;
    zip.write_all(sounds_json.as_bytes())?;

    zip.finish()?;

    info!("Created resource pack");

    let bytes = fs::read(path).unwrap();
    fs::remove_file(path).unwrap();

    Ok(bytes)
}