mod printer;
mod reader;
mod env;

use env_logger;
use pest::error::Error;
use printer::pr_str;
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
            if let Some(value) = env.borrow().get(s) {
                Ok(value.clone())
            } else {
                Err(format!("Symbol '{}' not found in environment", s).into())
            }
        }

        // Case for evaluating a list (represented as a Round value)
        MalValue::Round(list) => {
            if list.is_empty() {
                return Ok(MalValue::Round(list.clone()));
            }

            // Evaluate the first element to get the function
            let func = eval(&list[0], env.clone())?;

            match func {
                MalValue::BuiltinFunction(Function::SpecialForm(func)) => {
                    // Pass unevaluated arguments to the special form
                    func(&list[1..], env.clone())
                }
                // MalValue::BuiltinFunction(Function::WithEnv(func, func_env)) => {
                //     // Evaluate the arguments
                //     let args: Vec<MalValue> = list[1..]
                //         .iter()
                //         .map(|x| eval(x, env.clone()))
                //         .collect::<Result<Vec<MalValue>>>()?;
                //     func(&args, func_env.clone())
                // }
                MalValue::BuiltinFunction(Function::Builtin(func)) => {
                    // Evaluate the arguments
                    let args: Vec<MalValue> = list[1..]
                        .iter()
                        .map(|x| eval(x, env.clone()))
                        .collect::<Result<Vec<MalValue>>>()?;
                    func(&args)
                }
                MalValue::BuiltinFunction(Function::UserDefined { params, body, env: func_env }) => {
                    // Evaluate the arguments
                    let args: Vec<MalValue> = list[1..]
                        .iter()
                        .map(|x| eval(x, env.clone()))
                        .collect::<Result<Vec<MalValue>>>()?;

                    if params.len() != args.len() {
                        return Err(format!(
                            "Expected {} arguments but got {}",
                            params.len(),
                            args.len()
                        ));
                    }

                    // Create a new environment for the function
                    let new_env = Rc::new(RefCell::new(Env::new(
                        Some(Rc::clone(&func_env.borrow().get_bindings())),
                    )));

                    // Bind parameters to arguments
                    for (param, arg) in params.iter().zip(args.iter()) {
                        new_env.borrow_mut().set(param.clone(), arg.clone());
                    }

                    // Evaluate the function body
                    let mut result = MalValue::Nil;
                    for expr in body.iter() {
                        result = eval(expr, Rc::clone(&new_env))?;
                    }

                    Ok(result)
                }
                _ => Err("First element is not a function".to_string()),
            }
        }

        // Other cases, delegate to eval_ast
        _ => eval_ast(ast, env),
    }
}

fn eval_all(input: Vec<MalValue>, env: Rc<RefCell<Env>>) -> Result<Vec<MalValue>> {
    input.into_iter().map(|x| eval(&x, env.clone())).collect()
}

fn print(input: Vec<MalValue>) -> String {
    input
        .iter()
        .map(|node| pr_str(node, true))
        .collect::<Vec<String>>()
        .join(" ")
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
