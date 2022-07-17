use super::*;
use crate::filters::*;

pub fn generate_bash_script(
    template_name: &str,
    t: Template,
    default_path: Option<PathBuf>,
) -> String {
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

    // only do the name nullable check if the variable is used,
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