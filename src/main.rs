mod ir;
mod lexer;
mod parser;
mod value;

use crate::ir::*;
use crate::lexer::*;
use crate::parser::*;
use crate::value::*;

fn main() {}

/// evaluate an expression and returns the the value
pub fn eval(src: &str) -> Value {
    return exec(&format!("fn main() {{ return {} }}", src));
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
        .map(|(_name, params, body)| Func::new(params, &body, &scope))
        .collect();

    if let Some(func_id) = scope.get("main") {
        return exec_ir(&funcs[func_id], &funcs, vec![]);
    } else {
        return Value::Err;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loop() {
        assert_eq!(
            exec(
                "
                fn main() {
                    let x = 1
                    while x < 10 {
                        x = x + 1
                    }
                    return x
                }
                "
            ),
            Value::I32(10)
        );
    }

    #[test]
    fn test_redefine_variable() {
        assert_eq!(
            exec(
                "
                fn main() {
                    let x = 1
                    x = x + 1
                    return x
                }
                "
            ),
            Value::I32(2)
        );
        assert_eq!(
            exec(
                "
                fn main() {
                    let x = 1
                    if true {
                        x = x + 1
                    } else {
                        x = x + 2
                    }
                    return x
                }
                "
            ),
            Value::I32(2)
        );
        assert_eq!(
            exec(
                "
                fn main() {
                    let x = 1
                    if true {
                        let x = 5
                        x = x + 1
                    }
                    return x
                }
                "
            ),
            Value::I32(1)
        );
    }

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
        assert_eq!(
            exec(
                "
                fn main() {
                    if false {
                        return 1
                    }
                    return 2
                }
                "
            ),
            Value::I32(2)
        );
        assert_eq!(
            exec(
                "
                fn main() {
                    let x = 1
                    {
                        let x = 2
                    }
                    return x
                }
                "
            ),
            Value::I32(1)
        );
        assert_eq!(
            exec(
                "
                fn bla() {
                    let x = 2
                    return 0
                }

                fn main() {
                    let x = 1
                    bla()
                    return x
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
                    fn add(a, b) {
                        return a + b
                    }

                    fn main() {
                        return add(1, 2)
                    }
                "
            ),
            Value::I32(3)
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
