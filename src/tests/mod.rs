use super::*;
use pretty_assertions::assert_eq;

#[test]
fn parse() -> Result<()> {
    let out = parse_template("hello {{ hi }} {{ hey | upper }} hii")?;

    assert_eq!(out.generated, "hello ${hi} ${hey_upper} hii");

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

    Ok(())
}

#[test]
fn parse_and_generate() -> Result<()> {
    let mut out = parse_template("hello {{ hi }} {{ hey | upper }} hii")?;

    out.variables.first_mut().unwrap().default = Some("meooow".to_string());
    out.variables.first_mut().unwrap().description = Some("this is a description".to_string());

    let out = generate_bash_script("test", out, Some("./folder/name.rs".into()));

    assert_eq!(
        out,
        r#"#!/bin/sh
set -e

if [ ! "$1" = "get-template" ] && [ ! "$1" = "get-vars" ]; then

# normal template-outputing block

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
if [ -z "$hi" ]; then
  echo "Error: No value provided for hi"
  exit 1
fi
if [ -z "$hey" ]; then
  echo "Error: No value provided for hey"
  exit 1
fi
if [ -z "$name" ] && [ -t 1 ]; then
  echo "Error: No value provided for name"
  exit 1
fi

# filters
upper() {
  echo $(echo "$1" | tr '[:lower:]' '[:upper:]')
}

# filtered variables
hey_upper=$(upper "$hey")

out="hello ${hi} ${hey_upper} hii"
if [ -t 1 ] ; then
  mkdir -p "./folder"

  # check if file exists
  if [ -f "./folder/${name}.rs" ] ; then
     read -r -p "File already exists, overwrite? [y/N] " response
     case "$response" in
       [yY][eE][sS]|[yY])
         ;;
       *)
         echo "Stopping"
         exit 1
         ;;
     esac
  fi

  echo "$out" > "./folder/${name}.rs"
  echo "created file at ./folder/${name}.rs";
else
  echo "$out"
fi

# end normal block
fi

# template editing section

if [ "$1" = "get-template" ]; then
echo "hello {{ hi }} {{ hey | upper }} hii"
fi

if [ "$1" = "get-vars" ]; then
echo "
default_path=\"./folder/name.rs\"

[defaults]
hi=\"meooow\"

[descriptions]
hi=\"this is a description\"

"
fi
"#
    );

    Ok(())
}

#[test]
fn invalid_variables() {
    let out = parse_template("this is a {{ demo that breaks }} because the variables are invalid");
    assert!(out.is_err());
}

#[test]
fn escapes() -> Result<()> {
    let out = parse_template(r#" this "string" should be $escaped "#)?;
    assert_eq!(out.generated, r#" this \"string\" should be \$escaped "#);

    let out = parse_template(r#" \$ double escape "#)?;
    assert_eq!(out.generated, r#" \\\$ double escape "#);

    Ok(())
}

#[test]
fn unfinished_variable() -> Result<()> {
    let out = parse_template(r#" this variable is {{ unfinished "#);
    assert!(out.is_err());

    Ok(())
}

#[test]
fn nested_variable() -> Result<()> {
    let out = parse_template(r#" this variable has {{ nesting {{ inside }} }} "#);
    assert!(out.is_err());

    Ok(())
}
