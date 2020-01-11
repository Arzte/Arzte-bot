use log::trace;
use serenity::framework::standard::CommandResult;
use std::{
    fs,
    io,
};
use tempdir::TempDir;

pub fn dn_file(url: &str, download_file: &str, final_file: &str) -> CommandResult {
    let tmp_dir = TempDir::new("arzte.download")?;
    let mut response = reqwest::get(url)?;

    let mut dest = fs::File::create(tmp_dir.path().join(download_file))?;

    io::copy(&mut response, &mut dest)?;

    trace!("Opening the file.");
    let tar_gz = std::fs::File::open(tmp_dir.path().join(&download_file))?;
    let tar = flate2::read::GzDecoder::new(tar_gz);
    let mut ar = tar::Archive::new(tar);
    ar.unpack(".")?;

    trace!("Copying bot bin to replace old bot bin");
    fs::copy(
        tmp_dir.path().join(final_file),
        std::path::Path::new(&format!("{}/{}", ".", final_file)),
    )?;
    Ok(())
}
