use clap::Parser;
use color_eyre::eyre::{eyre, Result};
use std::{
    fs::File,
    io::{self, Write},
    path::PathBuf,
};

mod filters;
mod generate;
mod helpers;
mod parse;
#[cfg(test)]
mod tests;

mod create_template;
mod edit_template;
mod init;
mod list_templates;

use generate::*;
use parse::*;

/// code generation templating toolkit
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    action: Action,
}

#[derive(clap::Subcommand, Debug)]
enum Action {
    /// Create a new template
    Create {
        /// name for the template
        ///
        /// will create a template script at `.moho/NAME.mh`, which you can run by calling
        /// the file directly, or by `bash .moho/NAME.mh`
        #[clap(value_parser)]
        name: String,
        /// default path the template will output to
        ///
        /// should end in `name.ext`
        ///
        /// eg: if it's set to `/path/to/name.rs`, and the template is called with
        /// `--name hi`, the file will be created at `/path/to/hi.rs`
        ///
        /// if none is provided, the file will be created at `name` in the current directory
        #[clap(name = "path", short, long, value_parser)]
        default_path: Option<PathBuf>,
        /// if set, the editor will be prefilled with this file's contents
        ///
        /// useful for creating templates out of existing files
        #[clap(short, long, value_parser)]
        source: Option<PathBuf>,
    },
    /// Edit an existing template
    Edit {
        /// name for the template to edit
        ///
        /// file at `.moho/NAME.mh` must exist
        #[clap(value_parser)]
        name: String,
    },
    /// List all existing templates in current directory
    List,
    /// Creates an empty .moho directory
    ///
    /// this is not strictly necessary, the `create` command will create the directory if it does not exist
    Init,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    match args.action {
        Action::Create {
            name,
            default_path,
            source,
        } => create_template::create_template(name, default_path, source),
        Action::Edit { name } => edit_template::edit_template(name),
        Action::List => list_templates::list_templates(),
        Action::Init => init::init(),
    }
}

#[derive(Debug)]
pub struct Template<'a> {
    original: &'a str,
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
