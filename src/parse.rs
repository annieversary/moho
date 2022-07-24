use super::*;

pub fn parse_template(template: &str) -> Result<Template> {
    let mut generated = String::with_capacity(template.len());
    let mut variables: Vec<Variable> = Vec::new();
    let mut filtered: Vec<FilteredVariable> = Vec::new();

    let chars = template.chars();
    let mut last_char = None;

    let mut var: Option<usize> = None;

    for (i, c) in chars.enumerate() {
        // starting variable
        if last_char == Some('{') && c == '{' {
            if var.is_none() {
                last_char = None;
                var = Some(i + 1);
                continue;
            } else {
                return Err(eyre!("nested variables are not allowed"));
            }
        }

        // ending variable
        if last_char == Some('}') && c == '}' {
            if let Some(start) = var.take() {
                let var_str = &template[start..i - 1].trim();

                generated.push_str("${");
                if var_str.contains('|') {
                    let f = parse_filtered_variable(var_str)?;
                    generated.push_str(&f.name);

                    if !variables.iter().any(|v| v.variable == f.variable) {
                        variables.push(Variable::new(f.variable));
                    }

                    if !filtered
                        .iter()
                        .any(|v| v.variable == f.variable && v.filters == f.filters)
                    {
                        filtered.push(f);
                    }
                } else {
                    validate_ident(var_str)?;

                    generated.push_str(var_str);
                    if !variables.iter().any(|v| &v.variable == var_str) {
                        variables.push(Variable::new(var_str));
                    }
                }
                generated.push('}');

                last_char = None;
                continue;
            }
        }

        if var.is_none() {
            if let Some(l) = last_char {
                if l == '"' || l == '$' || l == '`' || l == '\\' {
                    generated.push('\\');
                }

                generated.push(l);
            }
        }

        last_char = Some(c);
    }

    if var.is_some() {
        return Err(eyre!("variable was unfinished"));
    }

    if let Some(l) = last_char {
        if l == '"' || l == '$' || l == '`' || l == '\\' {
            generated.push('\\');
        }

        generated.push(l);
    }

    let mut is_name_used = true;
    // insert `name` variable if not exists
    if !variables.iter().any(|v| v.variable == "name") {
        is_name_used = false;
        variables.push(Variable {
            variable: "name",
            default: None,
            description: None,
        });
    }

    Ok(Template {
        original: template,
        generated,
        variables,
        filtered,
        is_name_used,
    })
}

fn parse_filtered_variable(v: &str) -> Result<FilteredVariable> {
    let mut vals = v.split('|').map(|s| s.trim());

    for v in vals.clone() {
        validate_ident(v)?;
    }

    let name = vals.clone().collect::<Vec<_>>().join("_");
    let variable = vals
        .next()
        .ok_or_else(|| eyre!("variable should have at least one ident"))?;
    let filters = vals.collect();

    Ok(FilteredVariable {
        variable,
        filters,
        name,
    })
}

fn validate_ident(s: &str) -> Result<()> {
    if s == "_" {
        return Err(eyre!("identifiers can't be a single underscore"));
    }

    if !s
        .chars()
        .next()
        .map(|c| c.is_ascii_alphabetic() || c == '_')
        .unwrap_or(false)
    {
        return Err(eyre!(
            "identifier {s} has to start with a letter or an underscore"
        ));
    }

    if !s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(eyre!("identifier {s} contains invalid characters"));
    }
    Ok(())
}
