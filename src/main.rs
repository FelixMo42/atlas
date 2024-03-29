mod core;
mod server;
mod targets;
mod utils;

use crate::core::*;

use std::io::Write;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    match &args.iter().map(|s| s.as_str()).collect::<Vec<&str>>()[1..] {
        ["server"] => server::start(),
        ["run", name] => {
            let src = std::fs::read_to_string(name)?;
            let module = module::Module::from_src(&src);
            println!("{:?}", module.exec("main", vec![]));
        }
        ["to-wasm", name, out] => {
            let src = std::fs::read_to_string(name)?;
            let module = module::Module::from_src(&src);
            let mut file = std::fs::File::create(out)?;
            file.write(&module.to_wasm())?;
        }
        ["to-ir", name, out] => {
            let src = std::fs::read_to_string(name)?;
            let module = module::Module::from_src(&src);
            let mut file = std::fs::File::create(out)?;
            module.log(&mut file)?;
        }
        _ => println!("ERR unknown command"),
    };

    return Ok(());
}

#[cfg(test)]
#[rustfmt::skip]
mod tests_ir {
    use crate::module::Module;
    use crate::value::*;

    fn test_interpreter(module: &Module, value: Value) {
        assert_eq!(module.exec("main", vec![]), value);
    }

    fn test_wasm(module: &Module, value: Value) {
        match value.get_type() {
            TypeDef::F64 => assert_eq!(exec_wasm::<f64>(module), value.as_f64()),
            TypeDef::I32 => assert_eq!(exec_wasm::<i32>(module), value.as_i32()),
            TypeDef::Bool => assert_eq!(exec_wasm::<i32>(module), if value.as_bool() { 1 } else {0}),
            _ => {}
        }

        match value.get_type() {
            TypeDef::F64 => assert_eq!(exec_wat::<f64>(module), value.as_f64()),
            TypeDef::I32 => assert_eq!(exec_wat::<i32>(module), value.as_i32()),
            TypeDef::Bool => assert_eq!(exec_wat::<i32>(module), if value.as_bool() { 1 } else {0}),
            _ => {}
        }
    }

    fn exec_wasm<T: wasmtime::WasmResults>(module: &Module) -> T {
        let wasm = module.to_wasm();

        let engine = wasmtime::Engine::default();
        let module = wasmtime::Module::new(&engine, wasm).unwrap();

        let mut store = wasmtime::Store::new(&engine, 4);
        let instance = wasmtime::Instance::new(&mut store, &module, &[]).unwrap();
        let main = instance
            .get_typed_func::<(), T, _>(&mut store, "main")
            .unwrap();

        // And finally we can call the wasm!
        return main.call(&mut store, ()).unwrap();
    }

    fn exec_wat<T: wasmtime::WasmResults>(module: &Module) -> T {
        let wat = module.to_wasm();

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

    fn test(src: &str, value: Value) {
        let module = &Module::from_src(src);
        test_interpreter(module, value.clone());
        test_wasm(module, value.clone());
    }

    fn test_eval(src: &str, value: Value) {
        test(&format!("main(): {} {{ return {} }}", value.get_type(), src), value)
    }

    #[test]
    #[ignore]
    fn test_memory() {
        test("
            main(): I32 {
                let address = alloc(100)
                store(address, 42)
                return load(address)
            }
        ", Value::i32(42))
    }

    #[test]
    fn test_comment() {
        test("
            // aofhawf
            ret(): I32 {
                // are //
                return 42 // 23agr 3
                // nbu75 hgy
            }

            // oy9y84gh
            main(): I32 {
                // [0ug8y 48y ]
                let x = ret() // oauyifg
                // 8wy4ihg 
                return x // ouahf
                // yrdfog5
            }
            // a;oehf
        ", Value::i32(42))
    }

    #[test]
    fn test_loop() {
        test("
            main(): I32 {
                let x = 1
                while x < 10 {
                    x = x + 1
                }
                return x
            }
        ", Value::i32(10));
    }

    #[test]
    fn test_redefine_variable() {
        test("
            main(): I32 {
                let x = 1
                x = x + 1
                return x
            }
        ", Value::i32(2));

        test("
            main(): I32 {
                let x = 1
                {
                    x = x + 1
                }
                return x
            }
        ", Value::i32(2));

        test("
            main(): I32 {
                let x = 1
                {
                    let x = 5
                    x = x + 1
                }
                return x
            }
        ", Value::i32(1));
    }

    #[test]
    fn test_branch_flow() {
        test("
            main(): I32 {
                if true {
                    return 1
                } else {
                    return 2
                }
            }
        ", Value::i32(1));
    
        test("
            main(): I32 {
                if false {
                    return 1
                }
                return 2
            }
        ", Value::i32(2));

        test("
            main(): I32 {
                let x = 1
                {
                    let x = 2
                }
                return x
            }
        ", Value::i32(1));

        test("
            bla(): I32 {
                let x = 2
                return 0
            }

            main(): I32 {
                let x = 1
                bla()
                return x
            }
        ", Value::i32(1));
    }

    #[test]
    fn test_variables() {
        test("
            main(): I32 {
                let x = 5
                return 40 + x
            }
        ", Value::i32(45));

        test("
            main(): I32 {
                let x = 5
                let x = x + 10
                return x
            }
        ", Value::i32(15));
    }

    #[test]
    fn test_func_def() {
        test("
            main(): I32 {
                return 40 + 2
            }
        ", Value::i32(42));

        test("
            forty(): I32 {
                return 20 * 2
            }

            main(): I32 {
                return forty() + 200
            }
        ", Value::i32(240));

        test("
            add(a: I32, b: I32): I32 {
                return a + b
            }

            main(): I32 {
                return add(1, 2)
            }
        ", Value::i32(3));

        test("
            fib(num: I32): I32 {
                return
                    if (num == 1) 1
                    else if (num == 0) 0
                    else fib(num - 1) + fib(num - 2)
            }

            main(): I32 {
                return fib(7)
            }
        ", Value::i32(13));
    }

    #[test]
    fn test_if() {
        test_eval("if true 1 else 2", Value::i32(1));
        test_eval("1 + if true 10 else 20", Value::i32(11));
        test_eval("if true 100 else 200 + 1", Value::i32(100));
        test_eval("if (false) 1000 else 2000", Value::i32(2000));
        test_eval("if (false) 1 else if (false) 2 else 3", Value::i32(3));
    }

    #[test]
    fn test_bool() {
        test_eval("true", Value::bool(true));
        test_eval("false", Value::bool(false));
        test_eval("12 == 12", Value::bool(true));
        test_eval("12 == 12 == true", Value::bool(true));
        test_eval("8 + 4 == 10 + 2", Value::bool(true));
    }

    #[test]
    fn test_num() {
        test_eval("0", Value::i32(0));
        test_eval("1", Value::i32(1));
        test_eval("42", Value::i32(42));
        test_eval("42.0", Value::f64(42.0));
        test_eval("42.2", Value::f64(42.2));
    }

    #[test]
    fn test_num_negative() {
        test_eval("-42", Value::i32(-42));
        test_eval("-42.2", Value::f64(-42.2));
    }

    #[test]
    fn test_op() {
        test_eval("1+1", Value::i32(2));
        test_eval("1 + 1", Value::i32(2));
        test_eval("40 + 2", Value::i32(42));
        test_eval("38.2 + 3.8", Value::f64(42.0));

        test_eval("40 * 2", Value::i32(80));
        test_eval("40 / 2", Value::i32(20));
        test_eval("40 - 2", Value::i32(38));
        test_eval("2 - 40", Value::i32(-38));
        test_eval("2 + -40", Value::i32(-38));

        test_eval("80 + 40 - 78", Value::i32(42));
        test_eval("2 + 20 * 2", Value::i32(42));
        test_eval("20 * 2 + 2", Value::i32(42));
        test_eval("1 + 20 * 2 + 1", Value::i32(42));
        test_eval("20 * 2 + 20 / 2", Value::i32(50));
    }

    #[test]
    fn test_paren() {
        test_eval("(42)", Value::i32(42));
        test_eval("(40) + 2", Value::i32(42));
        test_eval("(40 + 2)", Value::i32(42));
        test_eval("40 + (2)", Value::i32(42));
        test_eval("(((40)) + (2))", Value::i32(42));
    }
}
