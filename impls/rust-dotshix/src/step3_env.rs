mod printer;
mod reader;
mod env;

use env_logger;
use pest::error::Error;
use printer::print_node;
use reader::{format_pest_error, parse_input, MalValue, Rule};
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result as RustylineResult};
use std::result::Result as StdResult;
use env::{create_repl_env, Env, Function};
use std::rc::Rc;
use std::cell::RefCell;


// Custom Result type for our application
type Result<T> = StdResult<T, String>;

fn read(input: String) -> StdResult<Vec<MalValue>, Error<Rule>> {
    parse_input(&input)
}

fn eval_ast(ast: &MalValue, env: Rc<RefCell<Env>>) -> Result<MalValue> {
    match ast {
        MalValue::Symbol(s) => {
            if env.borrow().get(s).is_some() {
                Ok(MalValue::Symbol(s.clone()))
            } else {
                // Return the symbol as is, assuming it might be defined later
                Ok(MalValue::Symbol(s.clone()))
            }
        }
        MalValue::Round(list) | MalValue::Square(list) | MalValue::Curly(list) | MalValue::Mal(list) => {
            let eval_list: Result<Vec<MalValue>> = list.iter().map(|x| eval(x, env.clone())).collect();
            eval_list.map(|eval_list| match ast {
                MalValue::Round(_) => MalValue::Round(eval_list),
                MalValue::Square(_) => MalValue::Square(eval_list),
                MalValue::Curly(_) => MalValue::Curly(eval_list),
                MalValue::Mal(_) => MalValue::Mal(eval_list),
                _ => unreachable!(),
            })
        }
        _ => Ok(ast.clone()),
    }
}

fn eval(ast: &MalValue, env: Rc<RefCell<Env>>) -> Result<MalValue> {
    match ast {
        // Case for evaluating a single symbol
        MalValue::Symbol(s) => {
            // Borrow the environment to look up the symbol
            let env_borrowed = env.borrow();
            if let Some(value) = env_borrowed.get(s) {
                // If the symbol is found, return its value
                Ok(value.clone())
            } else {
                // If the symbol is not found, return an error
                Err(format!("Symbol '{}' not found in environment", s).into())
            }
        }

        // Case for evaluating a list (represented as a Round value)
        MalValue::Round(list) => {
            if list.is_empty() {
                return Ok(MalValue::Round(list.clone()));
            }

            // Extract the first element (function name) and the rest (arguments)
            let name = &list[0];
            let rest = &list[1..];

            if let MalValue::Symbol(s) = name {
                // Look up the function in the environment
                let func = env.borrow().get(s);
                if let Some(MalValue::BuiltinFunction(Function::WithEnv(func, func_env))) = func {
                    // If the function is a built-in with an environment, call it with the arguments and environment
                    return func(rest, func_env.clone());
                } else if let Some(MalValue::BuiltinFunction(Function::Builtin(func))) = func {
                    // If the function is a simple built-in, call it with the arguments
                    let eval_rest: Vec<MalValue> = rest.iter().map(|x| eval(x, env.clone())).collect::<Result<Vec<MalValue>>>()?;
                    return func(&eval_rest);
                } else if let Some(value) = func {
                    // If the function is found but not callable, return its value
                    return Ok(value.clone());
                } else {
                    // If the function is not found, return an error
                    return Err(format!("Symbol '{}' not found in environment", s).into());
                }
            }

            // If the first element is not a symbol, evaluate the list elements
            let eval_list: Vec<MalValue> = list.iter().map(|x| eval(x, env.clone())).collect::<Result<Vec<MalValue>>>()?;
            Ok(MalValue::Round(eval_list))
        }

        // Case for other types of AST nodes, delegated to eval_ast
        _ => eval_ast(ast, env),
    }
}

fn eval_all(input: Vec<MalValue>, env: Rc<RefCell<Env>>) -> Result<Vec<MalValue>> {
    input.into_iter().map(|x| eval(&x, env.clone())).collect()
}

fn print(input: Vec<MalValue>) -> String {
    for node in input.iter() {
        print_node(node);
        print!(" "); // Add space after each top-level element
    }
    String::new()
}

fn rep(input: String, env: Rc<RefCell<Env>>) -> String {
    match read(input) {
        Ok(parsed) => match eval_all(parsed, env.clone()) {
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
                let result = rep(line, repl_env.clone());
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
