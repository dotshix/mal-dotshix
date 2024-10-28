use crate::Function;
use crate::MalValue;

// Custom function to escape strings
fn escape_string(s: &str) -> String {
    let mut escaped = String::new();
    for c in s.chars() {
        match c {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(c),
        }
    }
    escaped
}

// Converts a MalValue to a String with optional readably formatting
pub fn pr_str(node: &MalValue, print_readably: bool) -> String {
    match node {
        MalValue::String(s) => {
            if print_readably {
                let escaped = escape_string(s);
                format!("\"{}\"", escaped)
            } else {
                s.clone()
            }
        }
        MalValue::Symbol(s) => s.clone(),
        MalValue::Number(n) => n.to_string(),
        MalValue::Bool(b) => b.to_string(),
        MalValue::Nil => "nil".to_string(),
        MalValue::Atom(a) => a.clone(),
        MalValue::Round(r) => {
            let contents = r
                .iter()
                .map(|v| pr_str(v, print_readably))
                .collect::<Vec<String>>()
                .join(" ");
            format!("({})", contents)
        }
        MalValue::Square(r) => {
            let contents = r
                .iter()
                .map(|v| pr_str(v, print_readably))
                .collect::<Vec<String>>()
                .join(" ");
            format!("[{}]", contents)
        }
        MalValue::Curly(r) => {
            let contents = r
                .iter()
                .map(|v| pr_str(v, print_readably))
                .collect::<Vec<String>>()
                .join(" ");
            format!("{{{}}}", contents)
        }
        MalValue::Comment(c) => c.clone(),
        MalValue::NonSpecialSeq(s) => s.clone(),
        MalValue::Mal(content) => content
            .iter()
            .map(|v| pr_str(v, print_readably))
            .collect::<Vec<String>>()
            .join(" "),
        MalValue::BuiltinFunction(func) => match func {
            Function::Builtin(_) => "<#builtin function>".to_string(),
            Function::SpecialForm(_) => "<#special form>".to_string(),
            Function::UserDefined { .. } => "<#function>".to_string(),
        },
        MalValue::EOI => "".to_string(),
    }
}

