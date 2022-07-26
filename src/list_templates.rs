use std::ffi::OsStr;

use super::*;

pub fn list_templates() -> Result<()> {
    let res = std::fs::read_dir(".moho")?
        .into_iter()
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_file() && p.file_stem().is_some() && p.extension() == Some(OsStr::new("mh"))
        })
        .collect::<Vec<_>>();

    if res.is_empty() {
        return Ok(());
    }

    for t in res {
        println!("{}", t.file_stem().unwrap().to_string_lossy());
        // TODO show template descriptions
    }

    Ok(())
}
