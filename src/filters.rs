pub fn get_filters(_filters: &[&str]) -> String {
    // TODO properly implement this
    // not sure if we wanna have like a stdlib or smth

    r#"upper() {
  echo $(echo "$1" | tr '[:lower:]' '[:upper:]')
}
"#
    .to_string()
}
