use crate::MalValue;
use crate::Function;

fn print_list(list: &Vec<MalValue>, open_delim: &str, close_delim: &str) {
    print!("{}", open_delim);
    let mut firsttime = true;
    for v in list {
        if !firsttime {
            print!(" ");
        }
        print_node(v);
        firsttime = false;
    }
    print!("{}", close_delim);
}

// Prints a single MalValue node
pub fn print_node(node: &MalValue) {
    match node {
        MalValue::String(s) => print!("{}", s),
        MalValue::Symbol(s) => print!("{}", s),
        MalValue::Number(n) => print!("{}", n),
        MalValue::Bool(b) => print!("{}", b),
        MalValue::Nil => print!("nil"),
        MalValue::Atom(a) => print!("{}", a),
        MalValue::Round(r) => {
            print_list(r, "(", ")");
        }
        MalValue::Square(r) => {
            print_list(r, "[", "]");
        }
        MalValue::Curly(r) => {
            print_list(r, "{", "}");
        }

        MalValue::Comment(c) => print!("{}", c),
        MalValue::NonSpecialSeq(s) => print!("{}", s),
        MalValue::Mal(content) => {
            for (i, item) in content.iter().enumerate() {
                if i > 0 {
                    print!(" ");
                }
                print_node(item);
            }
        }
        MalValue::BuiltinFunction(func) => {
            match func {
                Function::Builtin(_) => print!("<builtin function>"),
                Function::SpecialForm(_) => print!("<special form>"),
                Function::UserDefined { .. } => print!("<function>"),
            }
        }
        MalValue::EOI => {}                // Do nothing for EOI
    }
}
