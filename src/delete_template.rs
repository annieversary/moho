use super::*;

pub fn delete_template(name: String) -> Result<()> {
    let path = format!(".moho/{name}.mh");
    std::fs::remove_file(path)?;

    Ok(())
}
