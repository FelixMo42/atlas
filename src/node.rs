use crate::core::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    // base
    Ident(String),
    FuncCall(Box<Node>, Vec<Node>),
    Error,

    // literals
    I32(i32),
    F64(f64),
    Bool(bool),

    // unary operator
    Negative(Box<Node>),

    // operators
    Add(Box<Node>, Box<Node>),
    Sub(Box<Node>, Box<Node>),
    Mul(Box<Node>, Box<Node>),
    Div(Box<Node>, Box<Node>),
    Eq(Box<Node>, Box<Node>),

    // flow
    If(Box<Node>, Box<Node>, Box<Node>),
}

impl Node {
    pub fn eval(&self) -> Value {
        match self {
            Node::I32(val) => Value::I32(*val),
            Node::F64(val) => Value::F64(*val),
            Node::Bool(val) => Value::Bool(*val),
            Node::Negative(node) => node.eval().neg(),
            Node::Add(a, b) => a.eval().add(b.eval()),
            Node::Sub(a, b) => a.eval().sub(b.eval()),
            Node::Mul(a, b) => a.eval().mul(b.eval()),
            Node::Div(a, b) => a.eval().div(b.eval()),
            Node::Eq(a, b) => a.eval().eq(b.eval()),
            Node::Ident(_ident) => Value::I32(4200),
            Node::FuncCall(func, args) => call_func(func, args),
            Node::If(cond, then, el) => match cond.eval() {
                Value::Bool(true) => then.eval(),
                Value::Bool(false) => el.eval(),
                _ => Value::Err,
            },
            Node::Error => Value::Err,
        }
    }
}

pub fn call_func(func: &Box<Node>, args: &Vec<Node>) -> Value {
    match func.as_ref() {
        Node::Ident(ident) => match ident.as_str() {
            "add" => {
                let mut v = args[0].eval();
                for arg in &args[1..] {
                    v = v.add(arg.eval())
                }
                v
            }
            "sub" => {
                let mut v = args[0].eval();
                for arg in &args[1..] {
                    v = v.sub(arg.eval())
                }
                v
            }
            _ => Value::Err,
        },
        _ => Value::Err,
    }
}
