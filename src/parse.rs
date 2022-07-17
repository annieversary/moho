use super::*;

pub fn parse_template<'a>(template: &'a str) -> Template<'a> {
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
