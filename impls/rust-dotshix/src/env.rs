use crate::MalValue;
use std::collections::HashMap;
use std::result::Result as StdResult;
use std::rc::Rc;
use std::cell::RefCell;

pub type Function = fn(&[MalValue]) -> Result<MalValue>;
type Result<T> = StdResult<T, String>;
type EnvRef = Option<Rc<RefCell<Env>>>;

pub struct Env {
    data: HashMap<String, Function>,
    outer: EnvRef,
}

impl Env {
    pub fn new(outer: EnvRef) -> Self {
        Env {
            data: HashMap::new(),
            outer,
        }
    }

    pub fn set(&mut self, key: String, func: Function) {
        self.data.insert(key, func);
    }

    pub fn get(&self, key: &str) -> Option<Function> {
        match self.data.get(key) {
            Some(func) => Some(*func),
            None => {
                if let Some(ref outer) = self.outer {
                    outer.borrow().get(key) // <-- Recursive call
                } else {
                    None
                }
            }
        }
    }
}

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

pub fn create_repl_env() -> Rc<RefCell<Env>> {
    let repl_env = Rc::new(RefCell::new(Env::new(None)));

    repl_env.borrow_mut().set("+".to_string(), add);
    repl_env.borrow_mut().set("-".to_string(), sub);
    repl_env.borrow_mut().set("*".to_string(), mult);
    repl_env.borrow_mut().set("/".to_string(), divide);

    repl_env
}
