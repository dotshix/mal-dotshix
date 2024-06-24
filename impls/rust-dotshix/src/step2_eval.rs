mod printer;
mod reader;

use env_logger;
use pest::error::Error;
use printer::mal_printer::print_node;
use reader::mal_parser::{format_pest_error, parse_input, MalValue, Rule};
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result as RustylineResult};
use std::collections::HashMap;
use std::result::Result as StdResult;

// Custom Result type for our application
type Result<T> = StdResult<T, String>;

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
                        return eval_function_call(func, rest);
                    }
                    Ok(MalValue::Round(eval_list.clone()))
                }
                _ => Ok(MalValue::Round(eval_list.clone())),
            }
        }
        _ => eval_ast(ast, env),
    }
}

fn eval_function_call(func: &Function, args: &[MalValue]) -> Result<MalValue> {
    if args.len() != 2 {
        return Err(format!("Expected exactly two arguments for binary function").into());
    }
    if let (MalValue::Number(a), MalValue::Number(b)) = (&args[0], &args[1]) {
        Ok(MalValue::Number(func(*a, *b)))
    } else {
        Err(format!("Expected number arguments").into())
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
