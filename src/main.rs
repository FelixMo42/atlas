mod ir;
mod lexer;
mod module;
mod parser;
mod value;
// mod wasm;

pub mod core {
    pub use crate::module::Module;
    // pub use crate::wasm::exec_wasm;
}

fn main() {
    let src = "
        fn main() I32 {
            let x = 0
            while x < 10 {
                x = x + 1
            }
            return x
        }
    ";

    let module = module::Module::from_src(src);

    module.log();

    println!("result: {:?}", module.exec("main", vec![]));
}

#[cfg(test)]
#[rustfmt::skip]
mod tests_ir {
    use crate::module::*;
    use crate::value::*;

    fn test_interpreter(src: &str, value: Value) {
        assert_eq!(Module::from_src(src).exec("main", vec![]), value);
    }

    // fn test_wasm(src: &str, value: Value) {
    //     match value {
    //         Value::F64(val) => assert_eq!(wasm::exec_wasm::<f64>(src), val),
    //         Value::I32(val) => assert_eq!(wasm::exec_wasm::<i32>(src), val),
    //         Value::Bool(true) => assert_eq!(wasm::exec_wasm::<i32>(src), 1),
    //         Value::Bool(false) => assert_eq!(wasm::exec_wasm::<i32>(src), 0),
    //         _ => {}
    //     }
    // }

    fn test(src: &str, value: Value) {
        test_interpreter(src, value);
        // test_wasm(src, value);
    }

    fn test_eval(src: &str, value: Value) {
        test(&match value.get_type() {
            Type::F64 => format!("fn main() F64 {{ return {} }}", src),
            Type::I32 => format!("fn main() I32 {{ return {} }}", src),
            Type::Bool => format!("fn main() Bool {{ return {} }}", src),
        }, value)
    }

    #[test]
    #[ignore]
    fn test_memory() {
        test("
            fn main() I32 {
                let address = alloc(100)
                store(address, 42)
                return load(address)
            }
        ", Value::I32(42))
    }

    #[test]
    fn test_comment() {
        test("
            // aofhawf
            fn ret() I32 {
                // are //
                return 42 // 23agr 3
                // nbu75 hgy
            }

            // oy9y84gh
            fn main() I32 {
                // [0ug8y 48y ]
                let x = ret() // oauyifg
                // 8wy4ihg 
                return x // ouahf
                // yrdfog5
            }
            // a;oehf
        ", Value::I32(42))
    }

    #[test]
    fn test_loop() {
        test("
            fn main() I32 {
                let x = 1
                while x < 10 {
                    x = x + 1
                }
                return x
            }
        ", Value::I32(10));
    }

    #[test]
    fn test_redefine_variable() {
        test("
            fn main() I32 {
                let x = 1
                x = x + 1
                return x
            }
        ", Value::I32(2));

        test("
            fn main() I32 {
                let x = 1
                {
                    x = x + 1
                }
                return x
            }
        ", Value::I32(2));

        test("
            fn main() I32 {
                let x = 1
                {
                    let x = 5
                    x = x + 1
                }
                return x
            }
        ", Value::I32(1));
    }

    #[test]
    fn test_branch_flow() {
        test("
            fn main() I32 {
                if true {
                    return 1
                } else {
                    return 2
                }
            }
        ", Value::I32(1));
    
        test("
            fn main() I32 {
                if false {
                    return 1
                }
                return 2
            }
        ", Value::I32(2));

        test("
            fn main() I32 {
                let x = 1
                {
                    let x = 2
                }
                return x
            }
        ", Value::I32(1));

        test("
            fn bla() I32 {
                let x = 2
                return 0
            }

            fn main() I32 {
                let x = 1
                bla()
                return x
            }
        ", Value::I32(1));
    }

    #[test]
    fn test_variables() {
        test("
            fn main() I32 {
                let x = 5
                return 40 + x
            }
        ", Value::I32(45));

        test("
            fn main() I32 {
                let x = 5
                let x = x + 10
                return x
            }
        ", Value::I32(15));
    }

    #[test]
    fn test_func_def() {
        test("
            fn main() I32 {
                return 40 + 2
            }
        ", Value::I32(42));

        test("
            fn forty() I32 {
                return 20 * 2
            }

            fn main() I32 {
                return forty() + 2
            }
        ", Value::I32(42));

        test("
            fn add(a: I32, b: I32) I32 {
                return a + b
            }

            fn main() I32 {
                return add(1, 2)
            }
        ", Value::I32(3));

        test("
            fn fib(num: I32) I32 {
                return
                    if (num == 1) 1
                    else if (num == 0) 0
                    else fib(num - 1) + fib(num - 2)
            }

            fn main() I32 {
                return fib(7)
            }
        ", Value::I32(13));
    }

    #[test]
    fn test_if() {
        test_eval("if true 1 else 2", Value::I32(1));
        test_eval("1 + if true 1 else 2", Value::I32(2));
        test_eval("if true 1 else 2 + 1", Value::I32(1));
        test_eval("if (false) 1 else 2", Value::I32(2));
        test_eval("if (false) 1 else if (false) 2 else 3", Value::I32(3));
    }

    #[test]
    fn test_bool() {
        test_eval("true", Value::Bool(true));
        test_eval("false", Value::Bool(false));
        test_eval("12 == 12", Value::Bool(true));
        test_eval("12 == 12 == true", Value::Bool(true));
        test_eval("8 + 4 == 10 + 2", Value::Bool(true));
    }

    #[test]
    fn test_num() {
        test_eval("0", Value::I32(0));
        test_eval("1", Value::I32(1));
        test_eval("42", Value::I32(42));
        test_eval("42.0", Value::F64(42.0));
        test_eval("42.2", Value::F64(42.2));
    }

    #[test]
    fn test_num_negative() {
        test_eval("-42", Value::I32(-42));
        test_eval("-42.2", Value::F64(-42.2));
    }

    #[test]
    fn test_op() {
        test_eval("1+1", Value::I32(2));
        test_eval("1 + 1", Value::I32(2));
        test_eval("40 + 2", Value::I32(42));
        test_eval("38.2 + 3.8", Value::F64(42.0));

        test_eval("40 * 2", Value::I32(80));
        test_eval("40 / 2", Value::I32(20));
        test_eval("40 - 2", Value::I32(38));
        test_eval("2 - 40", Value::I32(-38));
        test_eval("2 + -40", Value::I32(-38));

        test_eval("80 + 40 - 78", Value::I32(42));
        test_eval("2 + 20 * 2", Value::I32(42));
        test_eval("20 * 2 + 2", Value::I32(42));
        test_eval("1 + 20 * 2 + 1", Value::I32(42));
        test_eval("20 * 2 + 20 / 2", Value::I32(50));
    }

    #[test]
    fn test_paren() {
        test_eval("(42)", Value::I32(42));
        test_eval("(40) + 2", Value::I32(42));
        test_eval("(40 + 2)", Value::I32(42));
        test_eval("40 + (2)", Value::I32(42));
        test_eval("(((40)) + (2))", Value::I32(42));
    }
}
