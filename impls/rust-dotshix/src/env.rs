use crate::MalValue;
use std::collections::HashMap;
use std::result::Result as StdResult;

pub type Function = fn(&[MalValue]) -> Result<MalValue>;
type Result<T> = StdResult<T, String>;

fn validate_and_extract(args: &[MalValue], func_name: &str) -> Result<(i64, i64)> {
    if args.len() != 2 {
        return Err(format!("Expected exactly two arguments for {} function", func_name).into());
    }

    if let (MalValue::Number(a), MalValue::Number(b)) = (&args[0], &args[1]) {
        Ok((*a, *b))
    } else {
        Err("Expected number arguments".into())
    }
}

fn add(args: &[MalValue]) -> Result<MalValue> {
    let (a, b) = validate_and_extract(args, "add")?;
    Ok(MalValue::Number(a + b))
}

fn sub(args: &[MalValue]) -> Result<MalValue> {
    let (a, b) = validate_and_extract(args, "subtract")?;
    Ok(MalValue::Number(a - b))
}

fn mult(args: &[MalValue]) -> Result<MalValue> {
    let (a, b) = validate_and_extract(args, "multiply")?;
    Ok(MalValue::Number(a * b))
}

fn divide(args: &[MalValue]) -> Result<MalValue> {
    let (a, b) = validate_and_extract(args, "divide")?;
    if b != 0 {
        Ok(MalValue::Number(a / b))
    } else {
        Err("Division by 0".into())
    }
}

pub fn create_repl_env() -> HashMap<String, Function> {
    let mut repl_env: HashMap<String, Function> = HashMap::new();
    repl_env.insert("+".to_string(), add);
    repl_env.insert("-".to_string(), sub);
    repl_env.insert("*".to_string(), mult);
    repl_env.insert("/".to_string(), divide);
    repl_env
}
