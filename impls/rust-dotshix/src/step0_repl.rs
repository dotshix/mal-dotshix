use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};

fn read(input: String) -> String {
    input
}

fn eval(input: String) -> String {
    input
}

fn print(input: String) -> String {
    input
}

fn rep(input: String) -> String {
    print(eval(read(input)))
}

fn main() -> Result<()> {
    let mut rl = DefaultEditor::new()?;
    rl.set_auto_add_history(true);

    loop {
        let readline = rl.readline("user> ");

        match readline {
            Ok(line) => {
                println!("{}", rep(line));
            }

            Err(ReadlineError::Interrupted) => {
                break
            }

            Err(ReadlineError::Eof) => {
                break
            }

            Err(err) => {
                eprintln!("Error {}", err);
                break
            }
        }
    }

    Ok(())
}
