mod reader;
mod printer;

use pest::error::Error;
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};
use std::result::Result as StdResult;
use reader::mal_parser::{MalValue, Rule, parse_input, format_pest_error};
use printer::mal_printer::print_node;
use env_logger;

fn read(input: String) -> StdResult<Vec<MalValue>, Error<Rule>> {
    parse_input(&input)
}

fn eval(input: Vec<MalValue>) -> Vec<MalValue> {
    // For now, eval just returns the input
    input
}

fn print(input: Vec<MalValue>) -> String {
    for node in input.iter() {
        print_node(node);
        print!(" "); // Add space after each top-level element
    }

    // Return empty string for now
    String::new()
}

fn rep(input: String) -> String {
    match read(input) {
        Ok(parsed) => {
            let evaluated = eval(parsed);
            print(evaluated)
        }
        Err(e) => format!("Error: {:?}", format_pest_error(e)),
    }
}

fn main() -> Result<()> {
    // Run with
    // RUST_LOG=debug,rustyline=off
    // for debug purposes
    env_logger::init();

    let mut rl = DefaultEditor::new()?;
    rl.set_auto_add_history(true);

    loop {
        let readline = rl.readline("user> ");
        match readline {
            Ok(line) => {
                let result = rep(line);
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
