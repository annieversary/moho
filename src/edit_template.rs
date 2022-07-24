use super::*;
use serde_derive::Deserialize;
use std::{collections::HashMap, process::Command};

pub fn edit_template(name: String) -> Result<()> {
    let path = format!(".moho/{name}.mh");
    let old_template = String::from_utf8(
        Command::new("/bin/sh")
            .args(["-c", &format!("{path} get-template")])
            .output()?
            .stdout,
    )?;

    let mut variables: Vars = toml::from_slice(
        &Command::new("/bin/sh")
            .args(["-c", &format!("{path} get-vars")])
            .output()?
            .stdout,
    )?;

    // Command::output adds an extra newline, so we remove it
    let template = edit::edit(&old_template[0..old_template.len() - 1])?;
    let mut parsed = parse_template(&template)?;

    ask_defaults_and_descriptions(&mut parsed, &mut variables)?;

    let out = generate_bash_script(&name, parsed, variables.default_path);

    // save to file
    let out_path = format!(".moho/{name}.mh");
    std::fs::create_dir_all(".moho")?;
    let mut file = File::create(&out_path)?;
    file.write_all(out.as_bytes())?;
    crate::helpers::make_executable(&out_path)?;

    Ok(())
}

fn ask_defaults_and_descriptions(t: &mut Template, vars: &mut Vars) -> Result<()> {
    let mut rl = rustyline::Editor::<()>::new()?;

    // default path
    let prompt = "default path (leave empty for no default path): ";
    let readline = if let Some(d) = &vars.default_path {
        rl.readline_with_initial(prompt, (&d.to_string_lossy(), ""))
    } else {
        rl.readline(prompt)
    }
    .unwrap_or_default();
    let readline = readline.trim();

    if !readline.is_empty() {
        vars.default_path = Some(readline.into());
    }

    // vars
    for v in &mut t.variables {
        if v.variable == "name" {
            continue;
        }

        let prompt = format!(
            "default value for {} (leave empty for no default): ",
            v.variable
        );

        let readline = if let Some(d) = vars.defaults.get(v.variable) {
            rl.readline_with_initial(&prompt, (d, ""))
        } else {
            rl.readline(&prompt)
        }
        .unwrap_or_default();

        let default = readline
            .trim()
            // escape
            .replace('"', "\\\"")
            .replace('$', "\\$")
            .replace('`', "\\`")
            .replace('\\', "\\\\");
        if !default.is_empty() {
            v.default = Some(default);
        }

        let prompt = format!(
            "description value for {} (leave empty for no description): ",
            v.variable
        );

        let readline = if let Some(d) = vars.descriptions.get(v.variable) {
            rl.readline_with_initial(&prompt, (d, ""))
        } else {
            rl.readline(&prompt)
        }
        .unwrap_or_default();
        let desc = readline
            .trim()
            .replace('"', "\\\"")
            .replace('$', "\\$")
            .replace('`', "\\`")
            .replace('\\', "\\\\");
        if !desc.is_empty() {
            v.description = Some(desc);
        }
    }

    Ok(())
}

#[derive(Deserialize)]
struct Vars {
    default_path: Option<PathBuf>,
    defaults: HashMap<String, String>,
    descriptions: HashMap<String, String>,
}
