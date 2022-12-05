mod ir;
mod lexer;
mod module;
mod parser;
mod value;

use crate::ir::*;
use crate::module::*;
use crate::value::*;

use std::io::Write;

fn main() {}

/// run the main function from source code and returns the result
pub fn exec(src: &str) -> Value {
    Module::from_src(src).exec("main", vec![])
}

/// evaluate an expression and returns the the value
pub fn eval(src: &str) -> Value {
    return exec(&format!("fn main() I32 {{ return {} }}", src));
}

pub fn exec_wasm<T: wasmtime::WasmResults>(src: &str) -> T {
    let wat = compile(src).unwrap();

    println!("{}", std::str::from_utf8(&wat).unwrap());

    let engine = wasmtime::Engine::default();
    let module = wasmtime::Module::new(&engine, wat).unwrap();

    let mut store = wasmtime::Store::new(&engine, 4);
    let instance = wasmtime::Instance::new(&mut store, &module, &[]).unwrap();
    let main = instance
        .get_typed_func::<(), T, _>(&mut store, "main")
        .unwrap();

    // And finally we can call the wasm!
    return main.call(&mut store, ()).unwrap();
}

/// transpile atlas to wat file
pub fn compile(src: &str) -> std::io::Result<Vec<u8>> {
    fn rep(t: Type) -> String {
        match t {
            Type::F64 => "f64",
            Type::I32 => "i32",
            Type::Bool => "bool",
        }
        .to_string()
    }

    let module = Module::from_src(src);
    let mut b = Vec::new();

    writeln!(b, "(module")?;

    fn add(b: &mut Vec<u8>, func: &Func, block: usize) -> std::io::Result<()> {
        match &func.ir.blocks[block] {
            ir::BlockData::Consts(_, value) => match value {
                Value::F64(v) => writeln!(b, "\t\tf64.const {}", v)?,
                Value::I32(v) => writeln!(b, "\t\ti32.const {}", v)?,
                _ => {}
            },
            ir::BlockData::Assign(_, inst) => match inst {
                Inst::Neg(val) => match func.ir.var_type[*val] {
                    Type::I32 => {
                        writeln!(b, "\t\ti32.const 0")?;
                        add(b, func, func.ir.var_decl[*val])?;
                        writeln!(b, "\t\ti32.sub")?;
                    }
                    Type::F64 => {
                        add(b, func, func.ir.var_decl[*val])?;
                        writeln!(b, "\t\tf64.neg")?;
                    }
                    _ => unimplemented!(),
                },
                _ => {}
            },
            _ => {}
        };

        return Ok(());
    }

    for (i, func) in module.funcs.iter().enumerate() {
        writeln!(b, "\t(func ${}", i)?;
        writeln!(b, "\t\t(export \"{}\")", func.name)?;
        writeln!(b, "\t\t(result {})", rep(func.return_type))?;

        if let Some(BlockData::Return(var)) = func.ir.blocks.last() {
            let decl = func.ir.var_decl[*var];
            add(&mut b, func, decl)?;
        } else {
            panic!("expeted function to end with return statment!")
        }

        writeln!(b, "\t)")?;
    }

    writeln!(b, ")")?;

    return Ok(b);
}

#[cfg(test)]
#[rustfmt::skip]
mod test_wasm {
    use crate::exec_wasm;

    #[test]
    fn test_negative_int() {
        assert_eq!(exec_wasm::<i32>("
            fn main() I32 {
                return -42
            }
        "), -42);

        assert_eq!(exec_wasm::<f64>("
            fn main() F64 {
                return -42.0
            }
        "), -42.0);
    }

    #[test]
    fn test_numbers() {
        assert_eq!(exec_wasm::<i32>("
            fn main() I32 {
                return 0
            }
        "), 0);

        assert_eq!(exec_wasm::<i32>("
            fn main() I32 {
                return 42
            }
        "), 42);

        assert_eq!(exec_wasm::<f64>("
            fn main() F64 {
                return 12.3
            }
        "), 12.3);
    }
}

#[cfg(test)]
#[rustfmt::skip]
mod tests_ir {
    use super::*;

    #[test]
    #[ignore]
    fn test_memory() {
        assert_eq!(exec("
            fn main() I32 {
                let address = alloc(100)
                store(address, 42)
                return load(address)
            }
        "), Value::I32(42))
    }

    #[test]
    fn test_comment() {
        assert_eq!(exec("
            // aofhawf
            fn ret() I32 {
                // are //
                return 42 // 23agr 3
            }

            // oy9y84gh
            fn main() I32 {
                // [0ug8y 48y ]
                let x = ret() // oauyifg
                // 8wy4ihg 
                return x // ouahf
            }
            // a;oehf
        "), Value::I32(42))
    }

    #[test]
    fn test_loop() {
        assert_eq!(exec("
            fn main() I32 {
                let x = 1
                while x < 10 {
                    x = x + 1
                }
                return x
            }
        "), Value::I32(10));
    }

    #[test]
    fn test_redefine_variable() {
        assert_eq!(exec("
            fn main() I32 {
                let x = 1
                x = x + 1
                return x
            }
        "), Value::I32(2));

        assert_eq!(exec("
            fn main() I32 {
                let x = 1
                if true {
                    x = x + 1
                } else {
                    x = x + 2
                }
                return x
            }
        "), Value::I32(2));

        assert_eq!(exec("
            fn main() I32 {
                let x = 1
                if true {
                    let x = 5
                    x = x + 1
                }
                return x
            }
        "), Value::I32(1));
    }

    #[test]
    fn test_branch_flow() {
        assert_eq!(exec("
            fn main() I32 {
                if true {
                    return 1
                } else {
                    return 2
                }
            }
        "), Value::I32(1));
    
        assert_eq!(exec("
            fn main() I32 {
                if false {
                    return 1
                }
                return 2
            }
        "), Value::I32(2));

        assert_eq!(exec("
            fn main() I32 {
                let x = 1
                {
                    let x = 2
                }
                return x
            }
        "), Value::I32(1));

        assert_eq!(exec("
            fn bla() I32 {
                let x = 2
                return 0
            }

            fn main() I32 {
                let x = 1
                bla()
                return x
            }
        "), Value::I32(1));
    }

    #[test]
    fn test_variables() {
        assert_eq!(exec("
            fn main() I32 {
                let x = 5
                return 40 + x
            }
        "), Value::I32(45));

        assert_eq!(exec("
            fn main() I32 {
                let x = 5
                let x = x + 10
                return x
            }
        "), Value::I32(15));
    }

    #[test]
    fn test_func_def() {
        assert_eq!(exec("
            fn main() I32 {
                return 40 + 2
            }
        "), Value::I32(42));

        assert_eq!(exec("
            fn forty() I32 {
                return 20 * 2
            }

            fn main() I32 {
                return forty() + 2
            }
        "), Value::I32(42));

        assert_eq!(exec("
            fn add(a: I32, b: I32) I32 {
                return a + b
            }

            fn main() I32 {
                return add(1, 2)
            }
        "), Value::I32(3));

        assert_eq!(exec("
            fn fib(num: I32) I32 {
                return
                    if (num == 1) 1
                    else if (num == 0) 0
                    else fib(num - 1) + fib(num - 2)
            }

            fn main() I32 {
                return fib(7)
            }
        "), Value::I32(13));
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
