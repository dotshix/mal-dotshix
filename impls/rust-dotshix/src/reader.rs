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
    // Other(String),         // Represents any other token not specifically categorized, e.g., +
    EOI, // Represents the end of input
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

pub fn parse_input(input: &str) -> Result<Vec<MalValue>, Error<Rule>> {
    let pairs = MalParser::parse(Rule::mal, input)?;
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
            let content = pair.as_str().to_string();
            debug!("STRING content: {:?}", content);
            MalValue::String(content)
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
