use color_eyre::eyre::Result;
use std::{fs, os::unix::prelude::PermissionsExt};

pub fn make_executable(path: &str) -> Result<()> {
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms)?;
    Ok(())
}
