mod ast;
mod ir;
mod lexer;
mod value;

use crate::ast::*;
use crate::ir::*;
use crate::lexer::*;
use crate::value::*;

fn main() {}

pub fn parse_value(lex: &mut Lexer) -> Ast {
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
        Token::Ident("if") => {
            let c = Box::new(parse_expr(lex));
            let a = Box::new(parse_expr(lex));
            lex.next(); // else
            let b = Box::new(parse_expr(lex));
            Ast::If(c, a, b)
        }
        Token::Ident(ident) => Ast::Ident(ident.to_string()),
        Token::OpenB => {
            let mut statements = vec![];
            while let Some(statement) = parse_statement(lex) {
                statements.push(statement);
            }
            Ast::Block(statements)
        }
        _ => Ast::Error,
    }
}

pub fn parse_close_paren(lex: &mut Lexer) -> bool {
    let save = lex.save();
    if lex.next() == Token::CloseP {
        return true;
    } else {
        lex.load(save);
        return false;
    }
}

pub fn parse_func_call(lex: &mut Lexer) -> Ast {
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

pub fn parse_mul(lex: &mut Lexer) -> Ast {
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

pub fn parse_add(lex: &mut Lexer) -> Ast {
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

pub fn parse_cmp(lex: &mut Lexer) -> Ast {
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

/// evaluate an expression and returns the the value
pub fn eval(src: &str) -> Value {
    let lex = &mut Lexer::new(src);
    let ast = parse_expr(lex);
    let val = exec_ir(
        &Func {
            body: ast_to_ir(ast),
            params: vec![],
        },
        &vec![],
        Value::Unit,
    );

    return val;
}

pub fn parse_statement(lex: &mut Lexer) -> Option<Statement> {
    match lex.next() {
        Token::Ident("let") => {
            let name = if let Token::Ident(name) = lex.next() {
                name.to_string()
            } else {
                return None;
            };

            lex.next(); // =

            let value = parse_expr(lex);

            Some(Statement::Assign(name, value))
        }
        Token::Ident("return") => Some(Statement::Return(parse_expr(lex))),
        _ => None,
    }
}

pub fn parse_func_def(lex: &mut Lexer) -> Option<(String, Vec<String>, Vec<Statement>)> {
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

    lex.next(); // {

    let mut body = vec![];
    while let Some(statement) = parse_statement(lex) {
        body.push(statement);
    }

    return Some((name.to_string(), params, body));
}

/// run the main function from source code and returns the result
pub fn exec(src: &str) -> Value {
    // parse all the functions
    let lex = &mut Lexer::new(src);
    let mut funcs = vec![];
    while let Some(func) = parse_func_def(lex) {
        funcs.push(func)
    }

    // register the functions in the scope
    let mut scope = Scope::default();
    for i in 0..funcs.len() {
        scope.set(funcs[i].0.clone(), i);
    }

    // turn the functions in to ir
    let funcs: Vec<Func> = funcs
        .into_iter()
        .map(|(_name, params, body)| Func {
            body: func_to_ir(body, &scope),
            params,
        })
        .collect();

    if let Some(func_id) = scope.get("main") {
        return exec_ir(&funcs[func_id], &funcs, Value::Unit);
    } else {
        return Value::Err;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_flow() {
        assert_eq!(
            exec(
                "
                fn main() {
                    if true {
                        return 1
                    } else {
                        return 2
                    }
                }
                "
            ),
            Value::I32(1)
        );
    }

    #[test]
    fn test_variables() {
        assert_eq!(
            exec(
                "
                    fn main() {
                        let x = 5
                        return 40 + x
                    }
                "
            ),
            Value::I32(45)
        );

        assert_eq!(
            exec(
                "
                    fn main() {
                        let x = 5
                        let x = x + 10
                        return x
                    }
                "
            ),
            Value::I32(15)
        );
    }

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
                        return forty() + 2
                    }
                "
            ),
            Value::I32(42)
        );

        assert_eq!(
            exec(
                "
                    fn fib(num) {
                        return
                            if (num == 1) 1
                            else if (num == 0) 0
                            else fib(num - 1) + fib(num - 2)
                    }

                    fn main() {
                        return fib(7)
                    }
                "
            ),
            Value::I32(13)
        );
    }

    #[test]
    fn test_if() {
        assert_eq!(eval("if true 1 else 2"), Value::I32(1));
        assert_eq!(eval("1 + if true 1 else 2"), Value::I32(2));
        assert_eq!(eval("if true 1 else 2 + 1"), Value::I32(1));
        assert_eq!(eval("if false 1 else 2 + 1"), Value::I32(3));
        assert_eq!(eval("if (false) 1 else 2"), Value::I32(2));
        assert_eq!(eval("if (false) 1 else if (false) 2 else 3"), Value::I32(3));
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
