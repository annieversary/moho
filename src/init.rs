use std::path::Path;

use super::*;

pub fn init() -> Result<()> {
    std::fs::create_dir_all(".moho")?;

    let filters = Path::new(".moho/filters.sh");
    if !filters.is_file() {
        std::fs::write(
            filters,
            "#!/bin/sh\nset -e\n\n# write your custom filters here\n",
        )?;
    }

    let readme = Path::new(".moho/readme.md");
    if !readme.is_file() {
        std::fs::write(
            readme,
            "# moho\n\nthis folder contains this project's moho templates\n",
        )?;
    }

    Ok(())
}
