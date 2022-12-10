use crate::ir::*;
use crate::module::*;
use crate::value::*;

use std::io::Write;

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
            Type::Bool => "i32",
        }
        .to_string()
    }

    let module = Module::from_src(src);
    let mut f = Vec::new();

    writeln!(f, "(module")?;

    for (i, func) in module.funcs.iter().enumerate() {
        writeln!(f, "\t(func ${}", i)?;
        writeln!(f, "\t\t(export \"{}\")", func.name)?;
        writeln!(f, "\t\t(result {})", rep(func.return_type))?;

        for var in 0..func.ir.num_vars {
            writeln!(f, "\t\t(local ${var} {})", rep(func.ir.var_type[var]))?;
        }

        for block in &func.ir.insts {
            match block {
                Inst::Op(var, op, a, b) => {
                    writeln!(f, "\t\tget_local ${a}")?;
                    writeln!(f, "\t\tget_local ${b}")?;
                    match (op, func.ir.var_type[*a]) {
                        (Op::Add, Type::I32) => writeln!(f, "\t\ti32.add")?,
                        (Op::Add, Type::F64) => writeln!(f, "\t\tf64.add")?,
                        (Op::Sub, Type::I32) => writeln!(f, "\t\ti32.sub")?,
                        (Op::Sub, Type::F64) => writeln!(f, "\t\tf64.sub")?,
                        (Op::Mul, Type::I32) => writeln!(f, "\t\ti32.mul")?,
                        (Op::Mul, Type::F64) => writeln!(f, "\t\tf64.mul")?,
                        (Op::Div, Type::I32) => writeln!(f, "\t\ti32.div_s")?,
                        (Op::Div, Type::F64) => writeln!(f, "\t\tf64.div_s")?,
                        (Op::Eq, Type::Bool) => writeln!(f, "\t\ti32.eq")?,
                        (Op::Eq, Type::I32) => writeln!(f, "\t\ti32.eq")?,
                        (Op::Eq, Type::F64) => writeln!(f, "\t\tf64.eq")?,
                        _ => {}
                    }
                    writeln!(f, "\t\tset_local ${var}")?;
                }
                Inst::UOp(var, op, a) => {
                    match op {
                        UOp::Neg => match func.ir.var_type[*a] {
                            Type::I32 => {
                                writeln!(f, "\t\ti32.const 0")?;
                                writeln!(f, "\t\tget_local ${a}")?;
                                writeln!(f, "\t\ti32.sub")?;
                            }
                            Type::F64 => {
                                writeln!(f, "\t\tget_local ${a}")?;
                                writeln!(f, "\t\tf64.neg")?;
                            }
                            _ => unimplemented!(),
                        },
                        UOp::Not => {
                            writeln!(f, "\t\tget_local ${a}")?;
                            writeln!(f, "\t\ti32.not")?;
                        }
                    }
                    writeln!(f, "\t\tset_local ${var}")?;
                }
                Inst::Const(var, val) => {
                    match val {
                        Value::Bool(true) => writeln!(f, "\t\ti32.const 1")?,
                        Value::Bool(false) => writeln!(f, "\t\ti32.const 0")?,
                        Value::F64(val) => writeln!(f, "\t\tf64.const {val}")?,
                        Value::I32(val) => writeln!(f, "\t\ti32.const {val}")?,
                        _ => unimplemented!(),
                    }
                    writeln!(f, "\t\tset_local ${var}")?;
                }
                Inst::Return(var) => {
                    writeln!(f, "\t\tget_local ${var}")?;
                }
                _ => {}
            }
        }

        writeln!(f, "\t)")?;
    }

    writeln!(f, ")")?;

    return Ok(f);
}
