use crate::printer::pr_str;
use crate::Function;
use std::cell::RefCell;
use std::rc::Rc;
use crate::eval;
use crate::MalValue;
use crate::Env;

use std::result::Result as StdResult;
type Result<T> = StdResult<T, String>;


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

pub fn do_func(args: &[MalValue], env: Rc<RefCell<Env>>) -> Result<MalValue> {
    let mut res = MalValue::Nil;

    for expr in args {
        res = eval(expr, Rc::clone(&env))?;
    }

    Ok(res)
}

pub fn if_special_form(args: &[MalValue], env: Rc<RefCell<Env>>) -> Result<MalValue> {
    if args.len() < 2 || args.len() > 3 {
        return Err("if requires two or three arguments".to_string());
    }

    let condition = &args[0];
    let then_expr = &args[1];
    let else_expr = if args.len() == 3 {
        Some(&args[2])
    } else {
        None
    };

    // eval condition
    let condition_res = eval(condition, Rc::clone(&env))?;

    // Determine if the condition is truthy (anything other than nil or false)
    let is_truthy = match condition_res {
        MalValue::Nil => false,
        MalValue::Bool(false) => false,
        _ => true,
    };

    if is_truthy {
        // Evaluate and return then_expr
        eval(then_expr, env)
    } else if let Some(else_expr) = else_expr {
        // Evaluate and return else_expr
        eval(else_expr, env)
    } else {
        // No else_expr provided, return nil
        Ok(MalValue::Nil)
    }
}

pub fn fn_star(args: &[MalValue], env: Rc<RefCell<Env>>) -> Result<MalValue> {
    if args.len() != 2 {
        return Err("fn* requires exactly two arguments".to_string());
    }

    let param_list = match &args[0] {
        MalValue::Round(r) if r.is_empty() => Vec::new(), // Empty parameter list
        MalValue::Square(s) | MalValue::Round(s) => s.clone(),
        _ => {
            return Err(
                "fn* first argument must be a vector that defines the function's parameters"
                    .to_string(),
            )
        }
    };


    // Find the position of '&' if it exists
    let amp_pos = param_list.iter().position(|p| matches!(p, MalValue::Symbol(s) if s == "&"));

    // Parse fixed parameters and optional variadic parameter
    let (fixed_params, rest_param) = match amp_pos {
        Some(pos) => {
            // '&' must not be the last element
            if pos + 1 >= param_list.len() {
                return Err("Expected symbol after &".to_string());
            }
            // '&' must be followed by exactly one symbol
            if pos + 2 != param_list.len() {
                return Err("Unexpected parameter after rest parameter".to_string());
            }

            // Extract the variadic parameter name
            let rest_param = match &param_list[pos + 1] {
                MalValue::Symbol(s) => s.clone(),
                _ => return Err("Expected symbol after &".to_string()),
            };

            // Collect fixed parameters before '&'
            let fixed_params = param_list[..pos]
                .iter()
                .map(|p| match p {
                    MalValue::Symbol(s) => Ok(s.clone()),
                    _ => Err("fn* Parameters must be Symbols".to_string()),
                })
                .collect::<Result<Vec<String>>>()?;

            (fixed_params, Some(rest_param))
        }
        None => {
            // No variadic parameter; collect all as fixed parameters
            let fixed_params = param_list
                .iter()
                .map(|p| match p {
                    MalValue::Symbol(s) => Ok(s.clone()),
                    _ => Err("fn* Parameters must be Symbols".to_string()),
                })
                .collect::<Result<Vec<String>>>()?;
            (fixed_params, None)
        }
    };

    let body = vec![args[1].clone()]; // Store the body as a vector of expressions

    let func = Function::UserDefined {
        params: fixed_params,
        rest_param,
        body,
        env: Rc::clone(&env),
    };

    Ok(MalValue::BuiltinFunction(func))
}

pub fn let_star(args: &[MalValue], env: Rc<RefCell<Env>>) -> Result<MalValue> {
    if args.len() != 2 {
        return Err("let* requires exactly two arguments".to_string());
    }

    let bindings_list = match &args[0] {
        MalValue::Round(v) => v,
        MalValue::Square(v) => v,
        _ => return Err("let* first argument must be a list of bindings".to_string()),
    };

    // Ensure bindings list has an even number of elements
    if bindings_list.len() % 2 != 0 {
        return Err("Bindings must be pairs".to_string());
    }

    // Create a new environment using the current environment as the outer value
    // 1. &env.borrow().bindings  -- Borrow bindings immuatably from current env
    // 2. Rc::clone(&env.borrow().bindings) -- Create a new reference counter pointer to the bindings
    // 3. Env::new(Rc::clone(&env.borrow().bindings)) -- Create a new environment with the cloned bindings as the parent
    // 4. RefCell::new(Env::new(Rc::clone(&env.borrow().bindings))) -- Wrap the new environment in a RefCell to allow interior mutability
    // 5. Rc::new(RefCell::new(Env::new(Rc::clone(&env.borrow().bindings)), None None)) -- Wrap the RefCell in an Rc to allow shared ownership
    let new_env = Rc::new(RefCell::new(Env::new(Some(Rc::clone(
        &env.borrow().get_bindings(),
    )))));

    // Iterate over bindings in pairs
    for pair in bindings_list.chunks(2) {
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

pub fn list(args: &[MalValue]) -> Result<MalValue> {
    Ok(MalValue::Round(args.to_vec()))
}

pub fn list_question(args: &[MalValue]) -> Result<MalValue> {
    if args.len() != 1 {
        return Err("list? requires at least one argument".to_string());
    }
    match args[0] {
        MalValue::Round(_) => Ok(MalValue::Bool(true)),
        _ => Ok(MalValue::Bool(false)),
    }
}

pub fn empty_question(args: &[MalValue]) -> Result<MalValue> {
    if args.len() != 1 {
        return Err("empty? requires exactly one argument".to_string());
    }

    match &args[0] {
        MalValue::Round(list) | MalValue::Square(list) => Ok(MalValue::Bool(list.is_empty())),
        MalValue::String(s) => Ok(MalValue::Bool(s.is_empty())),
        _ => Ok(MalValue::Bool(false)), // Non-collection types are not empty
    }
}

pub fn count(args: &[MalValue]) -> Result<MalValue> {
    if args.len() != 1 {
        return Err("Count requires exactly one argument".to_string());
    }

    match &args[0] {
        MalValue::Round(list) | MalValue::Square(list) => Ok(MalValue::Number(list.len() as i64)),
        MalValue::String(list) => Ok(MalValue::Number(list.len() as i64)),
        MalValue::Nil => Ok(MalValue::Number(0)),
        _ => Ok(MalValue::Nil),
    }
}

pub fn equals(args: &[MalValue]) -> Result<MalValue> {
    if args.len() != 2 {
        return Err("= requires exactly two argument".to_string());
    }

    Ok(MalValue::Bool(args[0] == args[1]))
}

pub fn comparison_operator(op: &str, args: &[MalValue]) -> Result<MalValue> {
    if args.len() != 2 {
        return Err(format!("{} requires exactly two arguments", op));
    }

    let (a, b) = match (args.get(0), args.get(1)) {
        (Some(MalValue::Number(a)), Some(MalValue::Number(b))) => (*a, *b),
        _ => return Err("Arguments must be numbers".into()),
    };

    let result = match op {
        "<" => a < b,
        "<=" => a <= b,
        ">" => a > b,
        ">=" => a >= b,
        _ => return Err(format!("Unsupported operator: {}", op)),
    };

    Ok(MalValue::Bool(result))
}

pub fn prn_fn(args: &[MalValue]) -> Result<MalValue> {
    let strs = args.iter()
        .map(|v| pr_str(v, true))
        .collect::<Vec<String>>()
        .join(" ");
    println!("{}", strs);
    Ok(MalValue::Nil)
}

pub fn pr_str_fn(args: &[MalValue]) -> Result<MalValue> {
    let strs = args.iter()
        .map(|v| pr_str(v, true))
        .collect::<Vec<String>>()
        .join(" ");
    Ok(MalValue::String(strs))
}

pub fn str_fn(args: &[MalValue]) -> Result<MalValue> {
    let strs = args.iter()
        .map(|v| pr_str(v, false))
        .collect::<Vec<String>>()
        .join("");

    Ok(MalValue::String(strs))
}

pub fn println_fn(args: &[MalValue]) -> Result<MalValue> {
    let strs = args.iter()
        .map(|v| pr_str(v, false))
        .collect::<Vec<String>>()
        .join(" ");
    println!("{}", strs);
    Ok(MalValue::Nil)
}


// Function to create the REPL environment with built-in functions
pub fn create_repl_env() -> Rc<RefCell<Env>> {
    let repl_env = Rc::new(RefCell::new(Env::new(None)));

    // Wrapper functions for comparison operators
    fn less_than(args: &[MalValue]) -> Result<MalValue> {
        comparison_operator("<", args)
    }

    fn less_than_or_equal(args: &[MalValue]) -> Result<MalValue> {
        comparison_operator("<=", args)
    }

    fn greater_than(args: &[MalValue]) -> Result<MalValue> {
        comparison_operator(">", args)
    }

    fn greater_than_or_equal(args: &[MalValue]) -> Result<MalValue> {
        comparison_operator(">=", args)
    }

    // Array of built-in functions
    let builtins: &[(&str, fn(&[MalValue]) -> Result<MalValue>)] = &[
        ("+", add),
        ("-", sub),
        ("*", mult),
        ("/", divide),
        ("list", list),
        ("list?", list_question),
        ("empty?", empty_question),
        ("count", count),
        ("=", equals),
        ("prn", prn_fn),
        ("pr-str", pr_str_fn),
        ("str", str_fn),
        ("println", println_fn),
        ("<", less_than),
        ("<=", less_than_or_equal),
        (">", greater_than),
        (">=", greater_than_or_equal),
        // Add more built-in functions as needed
    ];

    // Array of special forms
    let special_forms: &[(&str, fn(&[MalValue], Rc<RefCell<Env>>) -> Result<MalValue>)] = &[
        ("def!", def_bang),
        ("let*", let_star),
        ("do", do_func),
        ("fn*", fn_star),
        ("if", if_special_form),
        // Add more special forms as needed
    ];

    // Register built-in functions
    for &(name, func) in builtins {
        repl_env.borrow_mut().set(
            name.to_string(),
            MalValue::BuiltinFunction(Function::Builtin(func)),
        );
    }

    // Register special forms
    for &(name, func) in special_forms {
        repl_env.borrow_mut().set(
            name.to_string(),
            MalValue::BuiltinFunction(Function::SpecialForm(func)),
        );
    }

    repl_env
}
