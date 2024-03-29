use crate::core::lexer::*;
use crate::core::value::*;

#[derive(Debug, Clone)]
pub enum Ast {
    // base
    Ident(String),
    FuncCall(Box<Ast>, Vec<Ast>),
    Block(Vec<Ast>),
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

    // comparisons
    Eq(Box<Ast>, Box<Ast>),
    Gt(Box<Ast>, Box<Ast>),
    Lt(Box<Ast>, Box<Ast>),
    Ge(Box<Ast>, Box<Ast>),
    Le(Box<Ast>, Box<Ast>),
    Ne(Box<Ast>, Box<Ast>),

    // flow
    If(Box<Ast>, Box<Ast>, Box<Ast>),
    While(Box<Ast>, Box<Ast>),
    Return(Box<Ast>),

    // variables
    Declair(String, Box<Ast>),
    Assign(String, Box<Ast>),

    // misc
    Array(Vec<Ast>),

    // defintions
    FuncDef(FuncDef),
}

#[derive(Debug, Clone)]
pub struct FuncDef {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: TypeDef,
    pub body: Box<Ast>,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub param_type: TypeDef,
}

fn check(lex: &mut Lexer, token: Token) -> bool {
    let save = lex.save();
    if lex.next() == token {
        return true;
    } else {
        lex.load(save);
        return false;
    }
}

fn parse_value(lex: &mut Lexer) -> Ast {
    match lex.next() {
        Token::Sub => Ast::Negative(Box::new(parse_value(lex))),
        Token::Open('(') => {
            let expr = parse_expr(lex);

            if lex.next() == Token::Close(')') {
                expr
            } else {
                Ast::Error
            }
        }
        Token::I32(value) => Ast::I32(value),
        Token::F64(value) => Ast::F64(value),
        Token::Ident("true") => Ast::Bool(true),
        Token::Ident("false") => Ast::Bool(false),
        Token::Ident("while") => Ast::While(Box::new(parse_expr(lex)), Box::new(parse_expr(lex))),
        Token::Ident("return") => Ast::Return(Box::new(parse_expr(lex))),
        Token::Ident("let") => {
            let name = if let Token::Ident(name) = lex.next() {
                name.to_string()
            } else {
                return Ast::Error;
            };
            lex.next(); // =
            Ast::Declair(name, Box::new(parse_expr(lex)))
        }
        Token::Ident("if") => {
            let c = Box::new(parse_expr(lex));
            let a = Box::new(parse_expr(lex));
            if check(lex, Token::Ident("else")) {
                let b = Box::new(parse_expr(lex));
                Ast::If(c, a, b)
            } else {
                Ast::If(c, a, Box::new(Ast::Block(vec![])))
            }
        }
        Token::Ident(ident) => {
            let ident = ident.to_string();
            let save = lex.save();
            if check(lex, Token::Set) {
                Ast::Assign(ident, Box::new(parse_expr(lex)))
            } else if check(lex, Token::Open('(')) {
                let mut params = vec![];
                if !check(lex, Token::Close(')')) {
                    loop {
                        match parse_param(lex) {
                            Some(param) => params.push(param),
                            None => {
                                lex.load(save);
                                return Ast::Ident(ident);
                            }
                        }

                        if lex.next() == Token::Close(')') {
                            break;
                        }
                    }
                }

                if lex.next() != Token::Colon {
                    lex.load(save);
                    return Ast::Ident(ident);
                }

                // parse return type
                let return_type = parse_type(lex);

                // parse body
                let body = Box::new(parse_expr(lex));

                return Ast::FuncDef(FuncDef {
                    name: ident,
                    params,
                    return_type,
                    body,
                });
            } else {
                Ast::Ident(ident)
            }
        }
        Token::Open('{') => {
            let mut statements = vec![];
            while !check(lex, Token::Close('}')) {
                statements.push(parse_expr(lex));
            }
            Ast::Block(statements)
        }
        Token::Open('[') => {
            let mut values = vec![];
            while !check(lex, Token::Close(']')) {
                values.push(parse_expr(lex));
                check(lex, Token::Comma);
            }
            Ast::Array(values)
        }
        _ => Ast::Error,
    }
}

fn parse_func_call(lex: &mut Lexer) -> Ast {
    let value = parse_value(lex);

    if check(lex, Token::Open('(')) {
        let mut params = vec![];

        if !check(lex, Token::Close(')')) {
            params.push(parse_expr(lex));

            while lex.next() != Token::Close(')') {
                params.push(parse_expr(lex));
            }
        }

        return Ast::FuncCall(Box::new(value), params);
    } else {
        return value;
    }
}

fn parse_mul(lex: &mut Lexer) -> Ast {
    let a = parse_func_call(lex);

    let save = lex.save();

    match lex.next() {
        Token::Mul => Ast::Mul(Box::new(a), Box::new(parse_mul(lex))),
        Token::Div => Ast::Div(Box::new(a), Box::new(parse_mul(lex))),
        _ => {
            lex.load(save);
            a
        }
    }
}

fn parse_add(lex: &mut Lexer) -> Ast {
    let a = parse_mul(lex);

    let save = lex.save();

    match lex.next() {
        Token::Add => Ast::Add(Box::new(a), Box::new(parse_add(lex))),
        Token::Sub => Ast::Sub(Box::new(a), Box::new(parse_add(lex))),
        _ => {
            lex.load(save);
            a
        }
    }
}

fn parse_cmp(lex: &mut Lexer) -> Ast {
    let a = parse_add(lex);

    let save = lex.save();

    match lex.next() {
        Token::Eq => Ast::Eq(Box::new(a), Box::new(parse_add(lex))),
        Token::Ne => Ast::Ne(Box::new(a), Box::new(parse_add(lex))),
        Token::Gt => Ast::Gt(Box::new(a), Box::new(parse_add(lex))),
        Token::Ge => Ast::Ge(Box::new(a), Box::new(parse_add(lex))),
        Token::Lt => Ast::Lt(Box::new(a), Box::new(parse_add(lex))),
        Token::Le => Ast::Le(Box::new(a), Box::new(parse_add(lex))),
        _ => {
            lex.load(save);
            a
        }
    }
}

fn parse_expr(lex: &mut Lexer) -> Ast {
    return parse_cmp(lex);
}

fn parse_type(lex: &mut Lexer) -> TypeDef {
    match lex.next() {
        Token::Ident("I32") => TypeDef::I32,
        Token::Ident("F64") => TypeDef::F64,
        Token::Ident("Bool") => TypeDef::Bool,
        _ => unimplemented!(),
    }
}

fn parse_param(lex: &mut Lexer) -> Option<Param> {
    let tok = lex.next();

    let name = if let Token::Ident(name) = tok {
        name.to_string()
    } else {
        return None;
    };

    if lex.next() != Token::Colon {
        return None;
    }

    return Some(Param {
        name,
        param_type: parse_type(lex),
    });
}

pub fn parse(src: &str) -> Vec<Ast> {
    let mut funcs = vec![];

    let mut lex = Lexer::new(src);

    while !lex.is_done() {
        funcs.push(parse_expr(&mut lex))
    }

    return funcs;
}
