use crate::lexer::*;

#[derive(Debug, Clone, PartialEq)]
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
    Eq(Box<Ast>, Box<Ast>),

    // flow
    If(Box<Ast>, Box<Ast>, Box<Ast>),
    Assign(String, Box<Ast>),
    Return(Box<Ast>),
}

fn parse_value(lex: &mut Lexer) -> Ast {
    match lex.next() {
        Token::Sub => Ast::Negative(Box::new(parse_value(lex))),
        Token::OpenP => {
            let expr = parse_expr(lex);

            if lex.next() == Token::CloseP {
                expr
            } else {
                Ast::Error
            }
        }
        Token::I32(value) => Ast::I32(value),
        Token::F64(value) => Ast::F64(value),
        Token::Ident("true") => Ast::Bool(true),
        Token::Ident("false") => Ast::Bool(false),
        Token::Ident("return") => Ast::Return(Box::new(parse_expr(lex))),
        Token::Ident("let") => {
            let name = if let Token::Ident(name) = lex.next() {
                name.to_string()
            } else {
                return Ast::Error;
            };
            lex.next(); // =
            Ast::Assign(name, Box::new(parse_expr(lex)))
        }
        Token::Ident("if") => {
            let c = Box::new(parse_expr(lex));
            let a = Box::new(parse_expr(lex));
            let save = lex.save();
            if lex.next() == Token::Ident("else") {
                let b = Box::new(parse_expr(lex));
                Ast::If(c, a, b)
            } else {
                lex.load(save);
                Ast::If(c, a, Box::new(Ast::Block(vec![])))
            }
        }
        Token::Ident(ident) => Ast::Ident(ident.to_string()),
        Token::OpenB => {
            let mut statements = vec![];
            while !parse_close_braket(lex) {
                statements.push(parse_expr(lex));
            }
            Ast::Block(statements)
        }
        _ => Ast::Error,
    }
}

fn parse_close_paren(lex: &mut Lexer) -> bool {
    let save = lex.save();
    if lex.next() == Token::CloseP {
        return true;
    } else {
        lex.load(save);
        return false;
    }
}

fn parse_close_braket(lex: &mut Lexer) -> bool {
    let save = lex.save();
    if lex.next() == Token::CloseB {
        return true;
    } else {
        lex.load(save);
        return false;
    }
}

fn parse_func_call(lex: &mut Lexer) -> Ast {
    let value = parse_value(lex);

    let save = lex.save();

    match lex.next() {
        Token::OpenP => {
            let mut params = vec![];

            if !parse_close_paren(lex) {
                params.push(parse_expr(lex));

                while lex.next() != Token::CloseP {
                    params.push(parse_expr(lex));
                }
            }

            Ast::FuncCall(Box::new(value), params)
        }
        _ => {
            lex.load(save);
            value
        }
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
        _ => {
            lex.load(save);
            a
        }
    }
}

pub fn parse_expr(lex: &mut Lexer) -> Ast {
    return parse_cmp(lex);
}

pub fn parse_func_def(lex: &mut Lexer) -> Option<(String, Vec<String>, Ast)> {
    if lex.next() != Token::Ident("fn") {
        return None;
    };

    let name = if let Token::Ident(name) = lex.next() {
        name.to_string()
    } else {
        return None;
    };

    lex.next(); // (

    let mut params = vec![];
    if !parse_close_paren(lex) {
        loop {
            if let Token::Ident(name) = lex.next() {
                params.push(name.to_string());
            } else {
                return None;
            }

            if lex.next() == Token::CloseP {
                break;
            }
        }
    }

    let body = parse_expr(lex);

    return Some((name.to_string(), params, body));
}
