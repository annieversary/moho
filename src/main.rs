use clap::Parser;
use color_eyre::eyre::{eyre, Result};
use std::{
    fs::{self, File},
    io::{self, Write},
    os::unix::prelude::PermissionsExt,
    path::PathBuf,
};

mod filters;
mod generate;
mod parse;
#[cfg(test)]
mod tests;

use generate::*;
use parse::*;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    action: Action,
}

#[derive(clap::Subcommand, Debug)]
enum Action {
    Create {
        /// name for the template
        ///
        /// will create a template script at `.moho/NAME.mh`, which you can run by calling
        /// the file directly, or by `bash .moho/NAME.mh`
        #[clap(value_parser)]
        name: String,
        /// output path for the file
        ///
        /// should end in `name.ext`
        ///
        /// eg: if it's set to `/path/to/name.rs`, and the template is called with
        /// `--name hi`, the file will be created at `/path/to/hi.rs`
        ///
        /// if none is provided, the file will be created at `name` in the current directory
        #[clap(value_parser)]
        default_path: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    match args.action {
        Action::Create { name, default_path } => create_template(name, default_path),
    }
}

fn create_template(name: String, default_path: Option<PathBuf>) -> Result<()> {
    let template = edit::edit("basic template demo {{ meow }}")?;

    let mut parsed = parse_template(&template)?;

    ask_defaults_and_descriptions(&mut parsed)?;

    let out = generate_bash_script(&name, parsed, default_path);

    // save to file
    let out_path = format!(".moho/{name}.mh");
    std::fs::create_dir_all(".moho")?;
    let mut file = File::create(&out_path)?;
    file.write_all(out.as_bytes())?;
    make_executable(&out_path)?;

    Ok(())
}

fn make_executable(path: &str) -> Result<()> {
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms)?;
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
        if !s.is_empty() {
            v.default = Some(
                s.trim()
                    // escape
                    .replace('"', "\\\"")
                    .replace('$', "\\$")
                    .replace('`', "\\`")
                    .replace('\\', "\\\\"),
            );
        }
        s.clear();

        print!(
            "description value for {} (leave empty for no description): ",
            v.variable
        );
        io::stdout().flush()?;
        io::stdin().read_line(&mut s)?;
        if !s.is_empty() {
            v.description = Some(
                s.trim()
                    .replace('"', "\\\"")
                    .replace('$', "\\$")
                    .replace('`', "\\`")
                    .replace('\\', "\\\\"),
            );
        }
        s.clear();
    }

    Ok(())
}

#[derive(Debug)]
pub struct Template<'a> {
    _original: &'a str,
    generated: String,
    variables: Vec<Variable<'a>>,
    is_name_used: bool,
    filtered: Vec<FilteredVariable<'a>>,
}

#[derive(Debug, PartialEq)]
pub struct Variable<'a> {
    variable: &'a str,
    default: Option<String>,
    description: Option<String>,
}

impl<'a> Variable<'a> {
    pub fn new(variable: &'a str) -> Self {
        Self {
            variable,
            default: None,
            description: None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct FilteredVariable<'a> {
    variable: &'a str,
    filters: Vec<&'a str>,
    /// name for the filtered variable
    name: String,
}
