mod lexer;
mod node;
mod value;

pub mod core {
    pub use super::value::Value;
}

use std::collections::HashMap;

use crate::core::*;
use crate::lexer::*;
use crate::node::Node;

fn main() {}

pub fn parse_value(lex: &mut Lexer) -> Node {
    match lex.next() {
        Token::Sub => Node::Negative(Box::new(parse_value(lex))),
        Token::OpenP => {
            let expr = parse_expr(lex);

            if lex.next() == Token::CloseP {
                expr
            } else {
                Node::Error
            }
        }
        Token::I32(value) => Node::I32(value),
        Token::F64(value) => Node::F64(value),
        Token::Ident("true") => Node::Bool(true),
        Token::Ident("false") => Node::Bool(false),
        Token::Ident("if") => {
            let c = Box::new(parse_value(lex));
            let a = Box::new(parse_value(lex));
            lex.next(); // else
            let b = Box::new(parse_value(lex));
            Node::If(c, a, b)
        }
        Token::Ident(ident) => Node::Ident(ident.to_string()),
        _ => Node::Error,
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

pub fn parse_func_call(lex: &mut Lexer) -> Node {
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
        Token::Mul => Node::Mul(Box::new(a), Box::new(parse_mul(lex))),
        Token::Div => Node::Div(Box::new(a), Box::new(parse_mul(lex))),
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
        Token::Add => Node::Add(Box::new(a), Box::new(parse_add(lex))),
        Token::Sub => Node::Sub(Box::new(a), Box::new(parse_add(lex))),
        _ => {
            lex.load(save);
            a
        }
    }
}

pub fn parse_cmp(lex: &mut Lexer) -> Node {
    let a = parse_add(lex);

    let save = lex.save();

    match lex.next() {
        Token::Eq => Node::Eq(Box::new(a), Box::new(parse_add(lex))),
        _ => {
            lex.load(save);
            a
        }
    }
}

pub fn parse_expr(lex: &mut Lexer) -> Node {
    return parse_cmp(lex);
}

/// evaluate an expression and returns the the value
pub fn eval(src: &str) -> Value {
    let lex = &mut Lexer::new(src);
    return parse_expr(lex).eval();
}

#[derive(Default)]
pub struct Scope {
    vars: HashMap<String, Value>,
}

pub fn parse_func_def(lex: &mut Lexer) -> Option<(String, Node)> {
    if lex.next() != Token::Ident("fn") {
        return None;
    };

    let name = if let Token::Ident(name) = lex.next() {
        name.to_string()
    } else {
        return None;
    };

    lex.next(); // (
    lex.next(); // (
    lex.next(); // {
    lex.next(); // return

    let body = parse_expr(lex);

    lex.next(); // }

    return Some((name.to_string(), body));
}

pub fn parse_file(lex: &mut Lexer) -> Scope {
    let mut scope = Scope::default();

    while let Some((name, node)) = parse_func_def(lex) {
        scope.vars.insert(name, Value::Func(node));
    }

    return scope;
}

/// run the main function from source code and returns the result
pub fn exec(src: &str) -> Value {
    let scope = parse_file(&mut Lexer::new(src));

    if let Some(Value::Func(root_node)) = scope.vars.get("main") {
        return root_node.eval();
    } else {
        return Value::Err;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_func_def() {
        assert_eq!(
            exec(
                "
                    fn main() {
                        return 40 + 2
                    }
                "
            ),
            Value::I32(42)
        );

        assert_eq!(
            exec(
                "
                    fn forty() {
                        return 20 * 2
                    }

                    fn main() {
                        return 42 
                    }
                "
            ),
            Value::I32(42)
        );
    }

    #[test]
    fn test_if() {
        assert_eq!(eval("if true 1 else 2"), Value::I32(1));
        assert_eq!(eval("if (false) 1 else 2"), Value::I32(2));
    }

    #[test]
    fn test_bool() {
        assert_eq!(eval("true"), Value::Bool(true));
        assert_eq!(eval("false"), Value::Bool(false));
        assert_eq!(eval("12 == 12"), Value::Bool(true));
        assert_eq!(eval("12 == 12.0"), Value::Err);
        assert_eq!(eval("12 == 12 == true"), Value::Bool(true));
        assert_eq!(eval("8 + 4 == 10 + 2"), Value::Bool(true));
    }

    #[test]
    fn test_func_call() {
        assert_eq!(eval("add(42)"), Value::I32(42));
        assert_eq!(eval("add(12, 30)"), Value::I32(42));
        assert_eq!(eval("add(12, 10, 20)"), Value::I32(42));
        assert_eq!(eval("sub(50, 8)"), Value::I32(42));
        assert_eq!(eval("sub(50, 4 + 4)"), Value::I32(42));
    }

    #[test]
    fn test_num() {
        assert_eq!(eval("0"), Value::I32(0));
        assert_eq!(eval("1"), Value::I32(1));
        assert_eq!(eval("42"), Value::I32(42));
        assert_eq!(eval("42.0"), Value::F64(42.0));
        assert_eq!(eval("42.2"), Value::F64(42.2));
    }

    #[test]
    fn test_num_negative() {
        assert_eq!(eval("-42"), Value::I32(-42));
        assert_eq!(eval("-42.2"), Value::F64(-42.2));
    }

    #[test]
    fn test_op() {
        assert_eq!(eval("1+1"), Value::I32(2));
        assert_eq!(eval("1 + 1"), Value::I32(2));
        assert_eq!(eval("40 + 2"), Value::I32(42));
        assert_eq!(eval("38.2 + 3.8"), Value::F64(42.0));
        assert_eq!(eval("38 + 3.8"), Value::Err);

        assert_eq!(eval("40 * 2"), Value::I32(80));
        assert_eq!(eval("40 / 2"), Value::I32(20));
        assert_eq!(eval("40 - 2"), Value::I32(38));
        assert_eq!(eval("2 - 40"), Value::I32(-38));
        assert_eq!(eval("2 + -40"), Value::I32(-38));

        assert_eq!(eval("80 + 40 - 78"), Value::I32(42));
        assert_eq!(eval("2 + 20 * 2"), Value::I32(42));
        assert_eq!(eval("20 * 2 + 2"), Value::I32(42));
        assert_eq!(eval("1 + 20 * 2 + 1"), Value::I32(42));
        assert_eq!(eval("20 * 2 + 20 / 2"), Value::I32(50));
    }

    #[test]
    fn test_paren() {
        assert_eq!(eval("(42)"), Value::I32(42));
        assert_eq!(eval("(40) + 2"), Value::I32(42));
        assert_eq!(eval("(40 + 2)"), Value::I32(42));
        assert_eq!(eval("40 + (2)"), Value::I32(42));
        assert_eq!(eval("(((40)) + (2))"), Value::I32(42));
    }
}
