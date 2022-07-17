use clap::Parser;
use color_eyre::eyre::Result;
use std::{
    fs::{self, File},
    io::{self, Write},
    os::unix::prelude::PermissionsExt,
    path::PathBuf,
};

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

    let mut parsed = parse_template(&template);

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
            v.default = Some(s.trim().to_string());
        }
        s.clear();

        print!(
            "description value for {} (leave empty for no description): ",
            v.variable
        );
        io::stdout().flush()?;
        io::stdin().read_line(&mut s)?;
        if !s.is_empty() {
            v.description = Some(s.trim().to_string());
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

fn parse_template<'a>(template: &'a str) -> Template<'a> {
    let mut generated = String::with_capacity(template.len());
    let mut variables: Vec<Variable<'a>> = Vec::new();
    let mut filtered: Vec<FilteredVariable> = Vec::new();

    let chars = template.chars();
    let mut last_char = None;

    let mut var: Option<usize> = None;

    for (i, c) in chars.enumerate() {
        // starting variable
        if var.is_none() && last_char == Some('{') && c == '{' {
            last_char = None;
            var = Some(i + 1);
            continue;
        }

        // ending variable
        if last_char == Some('}') && c == '}' {
            if let Some(start) = var.take() {
                let var_str = &template[start..i - 1].trim();

                generated.push_str("${");
                if var_str.contains('|') {
                    let f = parse_filtered_variable(&var_str);
                    generated.push_str(&f.name);

                    variables.push(Variable::new(f.variable));
                    filtered.push(f);
                } else {
                    generated.push_str(var_str);
                    variables.push(Variable::new(var_str));
                }
                generated.push('}');

                last_char = None;
                continue;
            }
        }

        if var.is_none() {
            if let Some(l) = last_char {
                generated.push(l);
            }
        }

        last_char = Some(c);
    }

    let mut is_name_used = true;
    // insert `name` variable if not exists
    if variables.iter().find(|v| v.variable == "name").is_none() {
        is_name_used = false;
        variables.push(Variable {
            variable: "name",
            default: None,
            description: None,
        });
    }

    Template {
        _original: template,
        generated,
        variables,
        filtered,
        is_name_used,
    }
}

fn parse_filtered_variable<'a>(v: &'a str) -> FilteredVariable<'a> {
    let mut vals = v.split('|').map(|s| s.trim());

    let name = vals.clone().collect::<Vec<_>>().join("_");
    let variable = vals.next().expect("variable to have at least one value");
    let filters = vals.collect();

    FilteredVariable {
        variable,
        filters,
        name,
    }
}

fn generate_bash_script(template_name: &str, t: Template, default_path: Option<PathBuf>) -> String {
    let mut script = String::from("#!/bin/sh\nset -e\n\n");

    macro_rules! append {
        ( $($s:expr),* $(,)? ) => {
            $(script.push_str($s);)*
        };
    }

    // generate variables
    append!("# variable declarations\n");
    for v in &t.variables {
        append!(v.variable, "=''\n");
    }

    // generate argument parsing
    append!(
        r#"
# parse arguments
while test $# -gt 0; do
  case "$1" in
"#,
    );
    for v in &t.variables {
        append!(
            "    --",
            v.variable,
            ")\n      shift\n      ",
            v.variable,
            r#"="$1"
      shift
      ;;
"#,
        );
    }

    // count how many spaces we need
    let help_len = "-h, --help".len();
    let name_len = "--name NAME".len();
    let max = t
        .variables
        .iter()
        // +3 because of the two dashes, and the one space
        .map(|v| v.variable.len() * 2 + 3)
        .max()
        .unwrap_or_default()
        .max(help_len)
        .max(name_len)
        + 5;

    let spaces = (0..max).map(|_| ' ').collect::<String>();

    let path = if let Some(p) = default_path.clone() {
        if let Some(ext) = p.extension() {
            p.with_file_name("NAME.meow").with_extension(ext)
        } else {
            p.with_file_name("NAME")
        }
        .to_string_lossy()
        .to_string()
    } else {
        "./NAME".into()
    };

    append!(
        r#"    *)
      echo ""#,
        template_name,
        r#":"
      echo "generates file at "#,
        &path,
        r#""
      echo ""
      echo "options:"
      echo "-h, --help"#,
        &spaces[help_len..],
        r#"show brief help"
      echo "--name NAME"#,
        &spaces[name_len..],
        r#"filename (without extension)"
"#
    );

    for v in &t.variables {
        if v.variable == "name" {
            continue;
        }

        append!(
            r#"      echo "--"#,
            v.variable,
            " ",
            &v.variable.to_uppercase(),
        );
        if let Some(desc) = &v.description {
            let l = v.variable.len() * 2 + 3;
            append!(&spaces[l..], desc);
        }
        append!("\"\n");
    }

    append!(
        r#"      exit 0
      ;;
  esac
done
"#,
    );

    // defaults if there are any
    if t.variables.iter().any(|v| v.default.is_some()) {
        append!("\n# set variable defaults\n");
    }
    for v in &t.variables {
        if let Some(default) = &v.default {
            append!(v.variable, "=${", v.variable, ":-\"", default, "\"}\n");
        }
    }

    // check that all variables have values
    append!("\n# check that all variables have values\n");
    for v in &t.variables {
        if v.variable == "name" {
            continue;
        }
        append!(
            "if [[ -z \"$",
            v.variable,
            "\" ]]; then\n  echo \"No value provided for ",
            v.variable,
            "\"\n  exit 1\nfi\n"
        );
    }

    // TODO change this to only do the check if the variable is used,
    // or if it's not being piped to a file

    let name_check = if t.is_name_used { "" } else { " && [[ -t 1 ]]" };
    append!(
        r#"if [[ -z "$name" ]]"#,
        name_check,
        r#"; then
  echo "No value provided for name"
  exit 1
fi
"#
    );

    if !t.filtered.is_empty() {
        // get all the used filters
        let filters = get_filters(
            &t.filtered
                .iter()
                .flat_map(|a| a.filters.clone())
                .collect::<Vec<_>>(),
        );
        append!("\n# filters\n", &filters);

        append!("\n# filtered variables\n");
        for v in &t.filtered {
            append!(&v.name, "=");
            for filter in &v.filters {
                append!("$(", filter, " ");
            }
            append!("\"$", v.variable, "\"");
            for _ in &v.filters {
                append!(")");
            }
            append!("\n");
        }
    }
    append!("\nout=\"", &t.generated, "\"\n");

    let mkdir = if let Some(p) = &default_path {
        if let Some(p) = p.parent() {
            format!(
                r#"  mkdir -p "{}"
"#,
                p.to_string_lossy().to_string()
            )
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    };
    let path = if let Some(p) = default_path {
        if let Some(ext) = p.extension() {
            p.with_file_name("${name}.meow").with_extension(ext)
        } else {
            p.with_file_name("${name}")
        }
        .to_string_lossy()
        .to_string()
    } else {
        "./${name}".into()
    };

    append!(
        r#"if [ -t 1 ] ; then
"#,
        &mkdir,
        r#"  echo "$out" > ""#,
        &path,
        r#""
  echo "created file at "#,
        &path,
        r#"";
else
  echo "$out"
fi
"#
    );

    script
}

fn get_filters(_filters: &[&str]) -> String {
    // TODO properly implement this
    // not sure if we wanna have like a stdlib or smth

    r#"upper() {
  echo $(echo "$1" | tr '[:lower:]' '[:upper:]')
}
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse() {
        let out = parse_template("hello {{ hi }} {{ hey | upper }} hii");

        assert_eq!(out.generated, "hello ${hi} ${hey_upper} hi");

        // check variables
        assert_eq!(
            out.variables
                .into_iter()
                .map(|v| v.variable)
                .collect::<Vec<_>>(),
            vec!["hi", "hey", "name"]
        );

        // check filtered
        assert_eq!(
            out.filtered,
            vec![FilteredVariable {
                variable: "hey",
                filters: vec!["upper"],
                name: "hey_upper".to_string()
            }]
        );
    }

    #[test]
    fn parse_and_generate() {
        let mut out = parse_template("hello {{ hi }} {{ hey | upper }} hii");

        out.variables.first_mut().unwrap().default = Some("meooow".to_string());
        out.variables.first_mut().unwrap().description = Some("this is a description".to_string());

        let out = generate_bash_script("test", out, Some("./folder/name.rs".into()));

        assert_eq!(
            out,
            r#"#!/bin/sh
set -e

# variable declarations
hi=''
hey=''
name=''

# parse arguments
while test $# -gt 0; do
  case "$1" in
    --hi)
      shift
      hi="$1"
      shift
      ;;
    --hey)
      shift
      hey="$1"
      shift
      ;;
    --name)
      shift
      name="$1"
      shift
      ;;
    *)
      echo "test:"
      echo "generates file at ./folder/NAME.rs"
      echo ""
      echo "options:"
      echo "-h, --help      show brief help"
      echo "--name NAME     filename (without extension)"
      echo "--hi HI         this is a description"
      echo "--hey HEY"
      exit 0
      ;;
  esac
done

# set variable defaults
hi=${hi:-"meooow"}

# check that all variables have values
if [[ -z "$hi" ]]; then
  echo "No value provided for hi"
  exit 1
fi
if [[ -z "$hey" ]]; then
  echo "No value provided for hey"
  exit 1
fi
if [[ -z "$name" ]] && [[ -t 1 ]]; then
  echo "No value provided for name"
  exit 1
fi

# filters
upper() {
  echo $(echo "$1" | tr '[:lower:]' '[:upper:]')
}

# filtered variables
hey_upper=$(upper "$hey")

out="hello ${hi} ${hey_upper} hi"
if [ -t 1 ] ; then
  mkdir -p "./folder"
  echo "$out" > "./folder/${name}.rs"
  echo "created file at ./folder/${name}.rs";
else
  echo "$out"
fi
"#
        )
    }
}
