mod printer;
mod reader;
mod env;

use env_logger;
use pest::error::Error;
use printer::mal_printer::print_node;
use reader::{format_pest_error, parse_input, MalValue, Rule};
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result as RustylineResult};
use std::collections::HashMap;
use std::result::Result as StdResult;
use env::{create_repl_env, Function};

// Custom Result type for our application
type Result<T> = StdResult<T, String>;

fn read(input: String) -> StdResult<Vec<MalValue>, Error<Rule>> {
    parse_input(&input)
}

fn eval_ast(ast: &MalValue, env: &HashMap<String, Function>) -> Result<MalValue> {
    match ast {
        MalValue::Symbol(s) => {
            if let Some(_func) = env.get(s) {
                Ok(MalValue::Symbol(s.clone()))
            } else {
                // TODO Might need to change this
                //Err(format!("Symbol not found in environment: {}", s))
                Ok(MalValue::Symbol(s.clone()))
            }
        }
        MalValue::Round(list) | MalValue::Square(list) | MalValue::Curly(list) => {
            let eval_list: Result<Vec<MalValue>> = list.iter().map(|x| eval(x, env)).collect();
            match eval_list {
                Ok(eval_list) => match ast {
                    MalValue::Round(_) => Ok(MalValue::Round(eval_list)),
                    MalValue::Square(_) => Ok(MalValue::Square(eval_list)),
                    MalValue::Curly(_) => Ok(MalValue::Curly(eval_list)),
                    _ => unreachable!(),
                },
                Err(e) => Err(e),
            }
        }
        MalValue::Mal(list) => {
            let eval_list: Result<Vec<MalValue>> = list.iter().map(|x| eval(x, env)).collect();
            match eval_list {
                Ok(eval_list) => Ok(MalValue::Mal(eval_list)),
                Err(e) => Err(e),
            }
        }
        _ => Ok(ast.clone()),
    }
}

fn eval(ast: &MalValue, env: &HashMap<String, Function>) -> Result<MalValue> {
    match ast {
        MalValue::Round(list) => {
            if list.is_empty() {
                return Ok(MalValue::Round(list.clone()));
            }
            let eval_list: Vec<MalValue> = list
                .iter()
                .map(|x| eval(x, env))
                .collect::<Result<Vec<MalValue>>>()?;
            let name = &eval_list[0];
            let rest = &eval_list[1..];

            match name {
                MalValue::Symbol(s) => {
                    if let Some(func) = env.get(s) {
                        return func(rest);
                    }
                    Ok(MalValue::Round(eval_list.clone()))
                }
                _ => Ok(MalValue::Round(eval_list.clone())),
            }
        }
        _ => eval_ast(ast, env),
    }
}

fn eval_all(input: Vec<MalValue>, env: &HashMap<String, Function>) -> Result<Vec<MalValue>> {
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
        Ok(parsed) => match eval_all(parsed, env) {
            Ok(evaluated) => print(evaluated),
            Err(e) => format!("Error: {}", e),
        },
        Err(e) => format!("Error: {:?}", format_pest_error(e)),
    }
}

fn main() -> RustylineResult<()> {
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
