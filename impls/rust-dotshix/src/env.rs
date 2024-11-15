use crate::MalValue;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::result::Result as StdResult;

// Type Definitions
type Result<T> = StdResult<T, String>;
type BindingsHandle = Rc<RefCell<Bindings>>;

// Function Enum for  different function types
pub enum Function {
    Builtin(fn(&[MalValue]) -> Result<MalValue>),
    SpecialForm(fn(&[MalValue], Rc<RefCell<Env>>) -> Result<MalValue>),
    // WithEnv(
    //     fn(&[MalValue], Rc<RefCell<Env>>) -> Result<MalValue>,
    //     Rc<RefCell<Env>>,
    // ),
    UserDefined {
        params: Vec<String>,
        rest_param: Option<String>,
        body: Vec<MalValue>,
        env: Rc<RefCell<Env>>,
    },
}

// Implementations for Debug and Clone for Function
impl fmt::Debug for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Function::Builtin(_) => write!(f, "Builtin Function"),
            // Function::WithEnv(_, _) => write!(f, "WithEnv Function"),
            Function::UserDefined { .. } => write!(f, "UserDefined Function"),
            Function::SpecialForm(_) => write!(f, "SpecialForm"),
        }
    }
}

impl Clone for Function {
    fn clone(&self) -> Self {
        match self {
            Function::Builtin(func) => Function::Builtin(*func),
            // Function::WithEnv(func, env) => Function::WithEnv(*func, Rc::clone(env)),
            Function::SpecialForm(func) => Function::SpecialForm(*func),
            Function::UserDefined { params, rest_param, body, env } => Function::UserDefined {
                params: params.clone(),
                rest_param: rest_param.clone(),
                body: body.clone(),
                env: env.clone(),
            },
        }
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Function::Builtin(f1), Function::Builtin(f2)) => f1 == f2,
            (Function::SpecialForm(f1), Function::SpecialForm(f2)) => f1 == f2,
            (
                Function::UserDefined {
                    params: p1,
                    rest_param: rp1,
                    body: b1,
                    ..
                },
                Function::UserDefined {
                    params: p2,
                    rest_param: rp2,
                    body: b2,
                    ..
                },
            ) => p1 == p2 && b1 == b2 && rp1 == rp2, // Ignore the environment, compare only params and body
            _ => false,
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
        let bindings = Rc::new(RefCell::new(Bindings::new(parent)));

        Env { bindings }
    }

    pub fn set(&self, key: String, value: MalValue) {
        self.bindings.borrow_mut().set(key, value);
    }

    pub fn get(&self, key: &String) -> Option<MalValue> {
        self.bindings.borrow().get(key)
    }

    pub fn get_bindings(&self) -> Rc<RefCell<Bindings>> {
        Rc::clone(&self.bindings)
    }
}
