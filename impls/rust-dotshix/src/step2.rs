mod printer;
mod reader;

use env_logger;
use pest::error::Error;
use printer::mal_printer::print_node;
use reader::mal_parser::{format_pest_error, parse_input, MalValue, Rule};
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};
use std::result::Result as StdResult;
use std::collections::HashMap;

// Eval stuff
type Function = fn(i64, i64) -> i64;

fn add(a: i64, b: i64) -> i64 {
    a + b
}

fn sub(a: i64, b: i64) -> i64 {
    a - b
}

fn mult(a: i64, b: i64) -> i64 {
    a * b
}

fn divide(a: i64, b: i64) -> i64 {
    if b != 0 {
        a / b
    } else {
        panic!("Division by 0");
    }
}

fn create_repl_env() -> HashMap<String, Function> {
    let mut repl_env: HashMap<String, Function> = HashMap::new();
    repl_env.insert("+".to_string(), add);
    repl_env.insert("-".to_string(), sub);
    repl_env.insert("*".to_string(), mult);
    repl_env.insert("/".to_string(), divide);
    repl_env
}

fn read(input: String) -> StdResult<Vec<MalValue>, Error<Rule>> {
    parse_input(&input)
}

fn eval_ast(ast: &MalValue, env: &HashMap<String, Function>) -> MalValue {
    match ast {
        MalValue::Symbol(s) => {
            if let Some(_func) = env.get(s) {
                MalValue::Symbol(s.clone())
            } else {
                panic!("Symbol not found in environment: {}", s)
            }
        }
        MalValue::Round(list) | MalValue::Square(list) | MalValue::Curly(list) => {
            let eval_list: Vec<MalValue> = list.iter().map(|x| eval(x, env)).collect();
            match ast {
                MalValue::Round(_) => MalValue::Round(eval_list),
                MalValue::Square(_) => MalValue::Square(eval_list),
                MalValue::Curly(_) => MalValue::Curly(eval_list),
                _ => unreachable!(),
            }
        }
        MalValue::Mal(list) => {
            let eval_list: Vec<MalValue> = list.iter().map(|x| eval(x, env)).collect();
            MalValue::Mal(eval_list)
        }
        _ => ast.clone(),
    }
}

fn eval(ast: &MalValue, env: &HashMap<String, Function>) -> MalValue {
    match ast {
        MalValue::Round(list) => {
            if list.is_empty() {
                return MalValue::Round(list.clone());
            }
            let eval_list: Vec<MalValue> = list.iter().map(|x| eval(x, env)).collect();
            let name = &eval_list[0];
            let rest = &eval_list[1..];
            match name {
                MalValue::Symbol(s) => {
                    if let Some(func) = env.get(s) {
                        if rest.len() != 2 {
                            panic!("Expected exactly two arguments for binary function")
                        }
                        if let (MalValue::Number(a), MalValue::Number(b)) = (&rest[0], &rest[1]) {
                            MalValue::Number(func(*a, *b))
                        } else {
                            panic!("Expected number arguments")
                        }
                    } else {
                        panic!("Function not found: {}", s)
                    }
                }
                _ => panic!("First element is not a function symbol"),
            }
        }
        MalValue::Square(_) | MalValue::Curly(_) | MalValue::Mal(_) => eval_ast(ast, env),
        _ => eval_ast(ast, env),
    }
}

fn eval_all(input: Vec<MalValue>, env: &HashMap<String, Function>) -> Vec<MalValue> {
    input.into_iter().map(|x| eval(&x, env)).collect()
}

fn print(input: Vec<MalValue>) -> String {
    for node in input.iter() {
        print_node(node);
        print!(" "); // Add space after each top-level element
    }
    String::new()
}

fn rep(input: String, env: &HashMap<String, Function>) -> String {
    match read(input) {
        Ok(parsed) => {
            let evaluated = eval_all(parsed, env);
            print(evaluated)
        }
        Err(e) => format!("Error: {:?}", format_pest_error(e)),
    }
}

fn main() -> Result<()> {
    env_logger::init();

    let mut rl = DefaultEditor::new()?;
    rl.set_auto_add_history(true);
    let repl_env = create_repl_env();

    loop {
        let readline = rl.readline("user> ");
        match readline {
            Ok(line) => {
                let result = rep(line, &repl_env);
                println!("{}", result);
            }

            Err(ReadlineError::Interrupted) => {
                break;
            }

            Err(ReadlineError::Eof) => {
                break;
            }

            Err(err) => {
                eprintln!("Error {}", err);
                break;
            }
        }
    }

    Ok(())
}
