mod lexer;
mod value;

pub mod core {
    pub use super::value::Value;
}

use crate::core::*;
use crate::lexer::*;

fn main() {}

pub enum Node {
    // base
    Value(Value),
    Ident(String),
    FuncCall(Box<Node>, Vec<Node>),
    Error,

    // unary operator
    Negative(Box<Node>),

    // operators
    Add(Box<Node>, Box<Node>),
    Sub(Box<Node>, Box<Node>),
    Mul(Box<Node>, Box<Node>),
    Div(Box<Node>, Box<Node>),
}

impl Node {
    fn exec(&self) -> Value {
        match self {
            Node::Value(value) => *value,
            Node::Negative(node) => node.exec().neg(),
            Node::Add(a, b) => a.exec().add(b.exec()),
            Node::Sub(a, b) => a.exec().sub(b.exec()),
            Node::Mul(a, b) => a.exec().mul(b.exec()),
            Node::Div(a, b) => a.exec().div(b.exec()),
            Node::Ident(ident) => Value::I32(4200),
            Node::FuncCall(func, args) => call_func(func, args),
            Node::Error => Value::Err,
        }
    }
}

pub fn call_func(func: &Box<Node>, args: &Vec<Node>) -> Value {
    match func.as_ref() {
        Node::Ident(ident) => match ident.as_str() {
            "add" => {
                let mut v = args[0].exec();
                for arg in &args[1..] {
                    v = v.add(arg.exec())
                }
                v
            }
            "sub" => {
                let mut v = args[0].exec();
                for arg in &args[1..] {
                    v = v.sub(arg.exec())
                }
                v
            }
            _ => Value::Err,
        },
        _ => Value::Err,
    }
}

pub fn parse_value(lex: &mut Lexer) -> Node {
    match lex.next() {
        Token::Op('-') => Node::Negative(Box::new(parse_value(lex))),
        Token::Op('(') => {
            let expr = parse_expr(lex);

            if lex.next() == Token::Op(')') {
                expr
            } else {
                Node::Error
            }
        }
        Token::Value(value) => Node::Value(value),
        Token::Ident("pi") => Node::Value(Value::F64(std::f64::consts::PI)),
        Token::Ident(ident) => Node::Ident(ident.to_string()),
        _ => Node::Error,
    }
}

fn parse_close_paren(lex: &mut Lexer) -> bool {
    let save = lex.save();
    if lex.next() == Token::Op(')') {
        return true;
    } else {
        lex.load(save);
        return false;
    }
}

pub fn parse_func_call(lex: &mut Lexer) -> Node {
    let value = parse_value(lex);

    let save = lex.save();

    match lex.next() {
        Token::Op('(') => {
            let mut params = vec![];

            if !parse_close_paren(lex) {
                params.push(parse_expr(lex));

                while lex.next() != Token::Op(')') {
                    params.push(parse_expr(lex));
                }
            }

            Node::FuncCall(Box::new(value), params)
        }
        _ => {
            lex.load(save);
            value
        }
    }
}

pub fn parse_mul(lex: &mut Lexer) -> Node {
    let a = parse_func_call(lex);

    let save = lex.save();

    match lex.next() {
        Token::Op('*') => Node::Mul(Box::new(a), Box::new(parse_mul(lex))),
        Token::Op('/') => Node::Div(Box::new(a), Box::new(parse_mul(lex))),
        _ => {
            lex.load(save);
            a
        }
    }
}

pub fn parse_add(lex: &mut Lexer) -> Node {
    let a = parse_mul(lex);

    let save = lex.save();

    match lex.next() {
        Token::Op('+') => Node::Add(Box::new(a), Box::new(parse_add(lex))),
        Token::Op('-') => Node::Sub(Box::new(a), Box::new(parse_add(lex))),
        _ => {
            lex.load(save);
            a
        }
    }
}

pub fn parse_expr(lex: &mut Lexer) -> Node {
    return parse_add(lex);
}

pub fn exec(src: &str) -> Value {
    let lex = &mut Lexer::new(src);
    return parse_expr(lex).exec();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_func_call() {
        assert_eq!(exec("add(42)"), Value::I32(42));
        assert_eq!(exec("add(12, 30)"), Value::I32(42));
        assert_eq!(exec("add(12, 10, 20)"), Value::I32(42));
        assert_eq!(exec("sub(50, 8)"), Value::I32(42));
        assert_eq!(exec("sub(50, 4 + 4)"), Value::I32(42));
    }

    #[test]
    fn test_build_in_values() {
        assert_eq!(exec("pi"), Value::F64(std::f64::consts::PI))
    }

    #[test]
    fn test_num() {
        assert_eq!(exec("0"), Value::I32(0));
        assert_eq!(exec("1"), Value::I32(1));
        assert_eq!(exec("42"), Value::I32(42));
        assert_eq!(exec("42.0"), Value::F64(42.0));
        assert_eq!(exec("42.2"), Value::F64(42.2));
    }

    #[test]
    fn test_num_negative() {
        assert_eq!(exec("-42"), Value::I32(-42));
        assert_eq!(exec("-42.2"), Value::F64(-42.2));
    }

    #[test]
    fn test_op() {
        assert_eq!(exec("1+1"), Value::I32(2));
        assert_eq!(exec("1 + 1"), Value::I32(2));
        assert_eq!(exec("40 + 2"), Value::I32(42));
        assert_eq!(exec("38.2 + 3.8"), Value::F64(42.0));
        assert_eq!(exec("38 + 3.8"), Value::Err);

        assert_eq!(exec("40 * 2"), Value::I32(80));
        assert_eq!(exec("40 / 2"), Value::I32(20));
        assert_eq!(exec("40 - 2"), Value::I32(38));
        assert_eq!(exec("2 - 40"), Value::I32(-38));
        assert_eq!(exec("2 + -40"), Value::I32(-38));

        assert_eq!(exec("80 + 40 - 78"), Value::I32(42));
        assert_eq!(exec("2 + 20 * 2"), Value::I32(42));
        assert_eq!(exec("20 * 2 + 2"), Value::I32(42));
        assert_eq!(exec("1 + 20 * 2 + 1"), Value::I32(42));
        assert_eq!(exec("20 * 2 + 20 / 2"), Value::I32(50));
    }

    #[test]
    fn test_paren() {
        assert_eq!(exec("(42)"), Value::I32(42));
        assert_eq!(exec("(40) + 2"), Value::I32(42));
        assert_eq!(exec("(40 + 2)"), Value::I32(42));
        assert_eq!(exec("40 + (2)"), Value::I32(42));
        assert_eq!(exec("(((40)) + (2))"), Value::I32(42));
    }
}
