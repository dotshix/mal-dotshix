use crate::eval;
use crate::MalValue;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::result::Result as StdResult;

// Type Definitions
type Result<T> = StdResult<T, String>;
type BindingsHandle = Rc<RefCell<Bindings>>;

// Function Enum for Builtin and WithEnv functions
pub enum Function {
    Builtin(fn(&[MalValue]) -> Result<MalValue>),
    WithEnv(
        fn(&[MalValue], Rc<RefCell<Env>>) -> Result<MalValue>,
        Rc<RefCell<Env>>,
    ),
}

// Implementations for Debug and Clone for Function
impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Function::Builtin(_) => write!(f, "BuiltinFunction"),
            Function::WithEnv(_, _) => write!(f, "WithEnvFunction"),
        }
    }
}

impl Clone for Function {
    fn clone(&self) -> Self {
        match self {
            Function::Builtin(func) => Function::Builtin(*func),
            Function::WithEnv(func, env) => Function::WithEnv(*func, Rc::clone(env)),
        }
    }
}

// Struct for Bindings
pub struct Bindings {
    current_level: HashMap<String, MalValue>,
    parent: Option<BindingsHandle>,
}

// Implementation for Bindings
impl Bindings {
    pub fn new(parent: Option<BindingsHandle>) -> Self {
        Bindings {
            current_level: HashMap::new(),
            parent,
        }
    }

    pub fn set(&mut self, key: String, value: MalValue) {
        self.current_level.insert(key, value);
    }

    pub fn get(&self, key: &String) -> Option<MalValue> {
        match self.current_level.get(key) {
            Some(value) => Some(value.clone()),
            None => {
                if let Some(ref parent) = self.parent {
                    parent.borrow().get(key)
                } else {
                    None
                }
            }
        }
    }
}

// Struct for Env
pub struct Env {
    bindings: BindingsHandle,
}

// Implementation for Env
impl Env {
    pub fn new(parent: Option<BindingsHandle>) -> Self {
        Env {
            bindings: Rc::new(RefCell::new(Bindings::new(parent))),
        }
    }

    pub fn set(&mut self, key: String, value: MalValue) {
        self.bindings.borrow_mut().set(key, value);
    }

    pub fn get(&self, key: &String) -> Option<MalValue> {
        self.bindings.borrow().get(key)
    }
}

// Utility Functions for Arithmetic Operations
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

// Builtin Functions
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

pub fn def_bang(args: &[MalValue], env: Rc<RefCell<Env>>) -> Result<MalValue> {
    if args.len() != 2 {
        return Err("def! requires exactly two arguments".to_string());
    }

    let key = match &args[0] {
        MalValue::Symbol(s) => s.clone(),
        _ => return Err("def! first argument must be a symbol".to_string()),
    };

    let value = eval(&args[1], env.clone())?;
    env.borrow_mut().set(key.clone(), value.clone());
    Ok(value)
}

pub fn let_star(args: &[MalValue], env: Rc<RefCell<Env>>) -> Result<MalValue> {
    if args.len() != 2 {
        return Err("let* requires exactly two arguments".to_string());
    }

    let bindings = match &args[0] {
        MalValue::Round(v) => v,
        MalValue::Square(v) => v,
        _ => return Err("let* first argument must be a list of bindings".to_string()),
    };

    // Create a new environment using the current environment as the outer value
    // 1. &env.borrow().bindings  -- Borrow bindings immuatably from current env
    // 2. Rc::clone(&env.borrow().bindings) -- Create a new reference counter pointer to the bindings
    // 3. Env::new(Rc::clone(&env.borrow().bindings)) -- Create a new environment with the cloned bindings as the parent
    // 4. RefCell::new(Env::new(Rc::clone(&env.borrow().bindings))) -- Wrap the new environment in a RefCell to allow interior mutability
    // 5. Rc::new(RefCell::new(Env::new(Rc::clone(&env.borrow().bindings)))) -- Wrap the RefCell in an Rc to allow shared ownership
    let new_env = Rc::new(RefCell::new(Env::new(Some(Rc::clone(
        &env.borrow().bindings,
    )))));

    // Iterate over bindings in pairs
    for pair in bindings.chunks(2) {
        if pair.len() != 2 {
            return Err("Bindings must be pairs".to_string());
        }

        // Extract key and value
        let key = match &pair[0] {
            MalValue::Symbol(s) => s.clone(),
            _ => return Err("Bindings must start with a symbol".to_string()),
        };

        let value = &pair[1];
        // Evaluate the value in the new_env environment
        let evaluated_value = eval(value, Rc::clone(&new_env))?;
        // Set the evaluated value in the new let_env environment
        new_env.borrow_mut().set(key, evaluated_value);
    }

    // Evaluate the body of the let* form in the new let_env environment
    let body = args[1].clone();
    eval(&body, Rc::clone(&new_env))
}

// Function to create the REPL environment with built-in functions
pub fn create_repl_env() -> Rc<RefCell<Env>> {
    let repl_env = Rc::new(RefCell::new(Env::new(None)));

    repl_env.borrow_mut().set(
        "+".to_string(),
        MalValue::BuiltinFunction(Function::Builtin(add)),
    );
    repl_env.borrow_mut().set(
        "-".to_string(),
        MalValue::BuiltinFunction(Function::Builtin(sub)),
    );
    repl_env.borrow_mut().set(
        "*".to_string(),
        MalValue::BuiltinFunction(Function::Builtin(mult)),
    );
    repl_env.borrow_mut().set(
        "/".to_string(),
        MalValue::BuiltinFunction(Function::Builtin(divide)),
    );
    repl_env.borrow_mut().set(
        "def!".to_string(),
        MalValue::BuiltinFunction(Function::WithEnv(def_bang, Rc::clone(&repl_env))),
    );

    repl_env.borrow_mut().set(
        "let*".to_string(),
        MalValue::BuiltinFunction(Function::WithEnv(let_star, Rc::clone(&repl_env))),
    );

    repl_env
}
