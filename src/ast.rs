use crate::value::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Ast {
    // base
    Ident(String),
    FuncCall(Box<Ast>, Vec<Ast>),
    Block(Vec<Statement>),
    Error,

    // literals
    I32(i32),
    F64(f64),
    Bool(bool),

    // unary operator
    Negative(Box<Ast>),

    // operators
    Add(Box<Ast>, Box<Ast>),
    Sub(Box<Ast>, Box<Ast>),
    Mul(Box<Ast>, Box<Ast>),
    Div(Box<Ast>, Box<Ast>),
    Eq(Box<Ast>, Box<Ast>),

    // flow
    If(Box<Ast>, Box<Ast>, Box<Ast>),
}

impl Ast {
    pub fn add(a: Ast, b: Ast) -> Ast {
        Ast::Add(Box::new(a), Box::new(b))
    }

    pub fn sub(a: Ast, b: Ast) -> Ast {
        Ast::Sub(Box::new(a), Box::new(b))
    }

    pub fn mul(a: Ast, b: Ast) -> Ast {
        Ast::Mul(Box::new(a), Box::new(b))
    }

    pub fn div(a: Ast, b: Ast) -> Ast {
        Ast::Div(Box::new(a), Box::new(b))
    }
}
