use crate::env::Function;
use log::debug;
use pest::error::{Error, ErrorVariant};
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "mal.pest"]
pub struct MalParser;

#[derive(Debug, Clone)]
pub enum MalValue {
    String(String),        // Represents a LISP string, e.g., "hello"
    Symbol(String),        // Represents a LISP symbol, e.g., +, some-function
    Number(i64),           // Represents a LISP number, e.g., 123
    Bool(bool),            // Represents a LISP boolean, e.g., true or false
    Nil,                   // Represents LISP nil
    Round(Vec<MalValue>),  // Represents a LISP list, e.g., (1 2 3)
    Square(Vec<MalValue>), // Represents a LISP list, e.g., [1 2 3]
    Curly(Vec<MalValue>),  // Represents a LISP list, e.g., [1 2 3]
    Mal(Vec<MalValue>),    // Represents a LISP S-expression, e.g., (+ 1 2)
    Comment(String),       // Represents a LISP comment, e.g., ; this is a comment
    NonSpecialSeq(String), // Represents a sequence of characters that are not special symbols, e.g., abc123
    Atom(String), // Represents a LISP atom, e.g., a single, indivisible unit like a variable name or keyword
    BuiltinFunction(Function),
    // Other(String),         // Represents any other token not specifically categorized, e.g., +
    EOI, // Represents the end of input
}

impl PartialEq for MalValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (MalValue::String(s1), MalValue::String(s2)) => s1 == s2,
            (MalValue::Symbol(s1), MalValue::Symbol(s2)) => s1 == s2,
            (MalValue::Number(n1), MalValue::Number(n2)) => n1 == n2,
            (MalValue::Bool(b1), MalValue::Bool(b2)) => b1 == b2,
            (MalValue::Nil, MalValue::Nil) => true,
            // Consider Round and Square equal if their contents are equal
            (MalValue::Round(v1), MalValue::Round(v2)) => v1 == v2,
            (MalValue::Square(v1), MalValue::Square(v2)) => v1 == v2,
            (MalValue::Round(v1), MalValue::Square(v2)) => v1 == v2,
            (MalValue::Square(v1), MalValue::Round(v2)) => v1 == v2,
            (MalValue::Curly(v1), MalValue::Curly(v2)) => v1 == v2,
            //(MalValue::Mal(v1), MalValue::Mal(v2)) => v1 == v2,
            //(MalValue::Comment(c1), MalValue::Comment(c2)) => c1 == c2,
            //(MalValue::NonSpecialSeq(s1), MalValue::NonSpecialSeq(s2)) => s1 == s2,
            (MalValue::Atom(a1), MalValue::Atom(a2)) => a1 == a2,
            // Compare function pointers for equality
            (MalValue::BuiltinFunction(f1), MalValue::BuiltinFunction(f2)) => f1 == f2,
            (MalValue::EOI, MalValue::EOI) => true,
            _ => false, // Default case for non-matching variants
        }
    }
}

pub fn format_pest_error(error: Error<Rule>) -> String {
    match error.variant {
        ErrorVariant::ParsingError {
            positives,
            negatives: _,
        } => {
            let location = format!("{:?}", error.location);
            let positives_str = format!("{:?}", positives);
            let message = if positives_str.contains("EOF") || positives_str.contains("end of input")
            {
                "unbalanced or unexpected end of input"
            } else {
                "unbalanced input"
            };
            format!(
                "Error at {}:\nExpected one of: {}\nFound: []\n{}",
                location, positives_str, message
            )
        }
        ErrorVariant::CustomError { message } => {
            format!("Custom error at {:?}:\n{}", error.location, message)
        }
    }
}

pub fn parse_input(input: &str) -> Result<Vec<MalValue>, Box<Error<Rule>>> {
    let pairs = MalParser::parse(Rule::mal, input).map_err(Box::new)?;
    let mut ast = Vec::new();

    for pair in pairs {
        let node = build_ast(pair);
        ast.push(node);
    }

    Ok(ast)
}

fn build_ast(pair: Pair<Rule>) -> MalValue {
    debug!("Processing rule: {:?}", pair.as_rule());
    //debug!("Pair content: {:?}", pair.as_str());

    match pair.as_rule() {
        Rule::STRING => {
            let content_with_quotes = pair.as_str();
            // Remove the surrounding quotes
            let inner_content = &content_with_quotes[1..content_with_quotes.len() - 1];
            // Unescape the string content
            let unescaped = unescape_string(inner_content);
            MalValue::String(unescaped)
        }

        Rule::symbol => {
            let content = pair.as_str().to_string();
            debug!("SYMBOL content: {:?}", content);
            MalValue::Symbol(content)
        }

        Rule::number => {
            let content = pair.as_str().parse::<i64>().unwrap();
            debug!("NUMBER content: {:?}", content);
            MalValue::Number(content)
        }

        Rule::boolean => {
            let content = pair.as_str() == "true";
            debug!("BOOLEAN content: {:?}", content);
            MalValue::Bool(content)
        }

        Rule::round => {
            let content = pair.into_inner().map(build_ast).collect::<Vec<_>>();
            debug!("ROUND content: {:?}", content);
            MalValue::Round(content)
        }
        Rule::square => {
            let content = pair.into_inner().map(build_ast).collect::<Vec<_>>();
            debug!("SQUARE content: {:?}", content);
            MalValue::Square(content)
        }
        Rule::curly => {
            let content = pair.into_inner().map(build_ast).collect::<Vec<_>>();
            debug!("CURLY content: {:?}", content);
            MalValue::Curly(content)
        }

        Rule::COMMENT => {
            let content = pair.as_str().to_string();
            debug!("COMMENT content: {:?}", content);
            MalValue::Comment(content)
        }

        Rule::quote => {
            let inner_pair = pair.into_inner().next().unwrap();
            let quoted_value = build_ast(inner_pair);
            debug!("QUOTE content: {:?}", quoted_value);
            MalValue::Round(vec![MalValue::Symbol("quote".to_string()), quoted_value])
        }

        Rule::quasiquote => {
            let inner_pair = pair.into_inner().next().unwrap();
            let quoted_value = build_ast(inner_pair);
            debug!("QUASIQUOTE content: {:?}", quoted_value);
            MalValue::Round(vec![
                MalValue::Symbol("quasiquote".to_string()),
                quoted_value,
            ])
        }

        Rule::unquote => {
            let inner_pair = pair.into_inner().next().unwrap();
            let quoted_value = build_ast(inner_pair);
            debug!("UNQUOTE content: {:?}", quoted_value);
            MalValue::Round(vec![MalValue::Symbol("unquote".to_string()), quoted_value])
        }

        Rule::splicing_unquote => {
            let inner_pair = pair.into_inner().next().unwrap();
            let quoted_value = build_ast(inner_pair);
            debug!("SPLICING-UNQUOTE content: {:?}", quoted_value);
            MalValue::Round(vec![
                MalValue::Symbol("splice-unquote".to_string()),
                quoted_value,
            ])
        }

        Rule::deref => {
            let inner_pair = pair.into_inner().next().unwrap();
            let quoted_value = build_ast(inner_pair);
            debug!("DEREF content: {:?}", quoted_value);
            MalValue::Round(vec![MalValue::Symbol("deref".to_string()), quoted_value])
        }

        Rule::atom => {
            let content = pair.as_str().to_string();
            debug!("ATOM content: {:?}", content);
            MalValue::Atom(content)
        }

        Rule::metadata => {
            let mut inner_pairs = pair.into_inner();
            let meta_pair = inner_pairs.next().unwrap();
            debug!("META pair content: {:?}", meta_pair);
            let meta_value = build_ast(meta_pair);
            debug!("META value: {:?}", meta_value);
            let target_pair = inner_pairs.next().unwrap();
            debug!("META TARGET pair content: {:?}", target_pair);
            let target_value = build_ast(target_pair);
            debug!("META TARGET value: {:?}", target_value);
            MalValue::Round(vec![
                MalValue::Symbol("with-meta".to_string()),
                target_value,
                meta_value,
            ])
        }

        Rule::nil => {
            debug!("NIL content: nil");
            MalValue::Nil
        }

        Rule::NON_SPECIAL_SEQ => {
            let content = pair.as_str().to_string();
            debug!("NON_SPECIAL_SEQ content: {:?}", content);
            MalValue::NonSpecialSeq(content)
        }

        Rule::mal => {
            let content = pair.into_inner().map(build_ast).collect::<Vec<_>>();
            debug!("Mal content: {:?}", content);
            if content.len() == 1 {
                content.into_iter().next().unwrap()
            } else {
                MalValue::Mal(content)
            }
        }
        Rule::EOI => {
            debug!("EOI encountered");
            MalValue::EOI
        }
        _ => {
            // debug!("Unexpected rule encountered: {:?}", pair.as_rule());
            panic!("Unexpected rule: {:?}", pair.as_rule());
        }
    }
}

fn unescape_string(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some(other) => {
                    // Handle unknown escape sequences by including the backslash and character
                    result.push('\\');
                    result.push(other);
                },
                None => {
                    // Backslash at end of string
                    result.push('\\');
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}
