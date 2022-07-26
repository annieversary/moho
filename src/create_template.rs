use crate::helpers::escape;

use super::*;

pub fn create_template(
    name: String,
    default_path: Option<PathBuf>,
    source: Option<PathBuf>,
) -> Result<()> {
    let source = source
        .map(std::fs::read_to_string)
        .transpose()?
        .unwrap_or_default();

    let template = edit::edit(source)?;

    let mut parsed = parse_template(&template)?;

    ask_defaults_and_descriptions(&mut parsed)?;

    let out = generate_bash_script(&name, parsed, default_path);

    // save to file
    let out_path = format!(".moho/{name}.mh");
    std::fs::create_dir_all(".moho")?;
    let mut file = File::create(&out_path)?;
    file.write_all(out.as_bytes())?;
    crate::helpers::make_executable(&out_path)?;

    Ok(())
}

fn ask_defaults_and_descriptions(t: &mut Template) -> Result<()> {
    let mut s = String::new();

    for v in &mut t.variables {
        if v.variable == "name" {
            continue;
        }

        print!(
            "default value for {} (leave empty for no default): ",
            v.variable
        );
        io::stdout().flush()?;
        io::stdin().read_line(&mut s)?;
        let default = escape(s.trim());
        // escape
        if !default.is_empty() {
            v.default = Some(default);
        }
        s.clear();

        print!(
            "description value for {} (leave empty for no description): ",
            v.variable
        );
        io::stdout().flush()?;
        io::stdin().read_line(&mut s)?;
        let desc = escape(s.trim());
        if !desc.is_empty() {
            v.description = Some(desc);
        }
        s.clear();
    }

    Ok(())
}
