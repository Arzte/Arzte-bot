use serenity::framework::standard::CommandResult;
use std::fs::File;
use std::io::copy;
use tempdir::TempDir;

pub fn dn_file(url: &str, file: &str) -> CommandResult {
    let tmp_dir = TempDir::new("arzte.download")?;
    let mut response = reqwest::get(url)?;

    let mut dest = File::create(tmp_dir.path().join(file))?;

    copy(&mut response, &mut dest)?;

    std::fs::copy(
        tmp_dir.path().join(&file),
        std::path::Path::new(&format!("{}/{}", ".", &file)),
    )?;
    Ok(())
}