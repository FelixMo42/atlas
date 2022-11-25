use crate::core::Value;
use crate::value::*;

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
    pub fn eval(&self, scope: &Scope) -> Value {
        match self {
            Node::I32(val) => Value::I32(*val),
            Node::F64(val) => Value::F64(*val),
            Node::Bool(val) => Value::Bool(*val),
            Node::Negative(node) => node.eval(scope).neg(),
            Node::Add(a, b) => a.eval(scope).add(b.eval(scope)),
            Node::Sub(a, b) => a.eval(scope).sub(b.eval(scope)),
            Node::Mul(a, b) => a.eval(scope).mul(b.eval(scope)),
            Node::Div(a, b) => a.eval(scope).div(b.eval(scope)),
            Node::Eq(a, b) => a.eval(scope).eq(b.eval(scope)),
            Node::Ident(ident) => scope.get(ident),
            Node::FuncCall(func, args) => call_func(func, args, scope),
            Node::If(cond, then, el) => match cond.eval(scope) {
                Value::Bool(true) => then.eval(scope),
                Value::Bool(false) => el.eval(scope),
                _ => Value::Err,
            },
            Node::Error => Value::Err,
        }
    }
}

pub fn call_func(func: &Box<Node>, args: &Vec<Node>, scope: &Scope) -> Value {
    if let Value::Func(func) = func.eval(scope) {
        if func.params.len() != args.len() {
            return Value::Err;
        }

        let mut function_scope = scope.root();

        for i in 0..args.len() {
            function_scope.set(func.params[i].clone(), args[i].eval(&scope));
            println!("{} = {:?}", func.params[i], args[i].eval(&scope));
        }

        for statement in func.body {
            match statement {
                Statement::Assign(name, value) => {
                    function_scope.set(name, value.eval(&function_scope))
                }
                Statement::Return(value) => return value.eval(&function_scope),
            }
        }

        return Value::Unit;
    } else {
        return Value::Err;
    }
}
