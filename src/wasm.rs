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
            Type::Bool => "bool",
        }
        .to_string()
    }

    let module = Module::from_src(src);
    let mut f = Vec::new();

    writeln!(f, "(module")?;

    fn add(f: &mut Vec<u8>, func: &Func, block: usize) -> std::io::Result<()> {
        match &func.ir.blocks[block] {
            BlockData::Consts(_, value) => match value {
                Value::F64(v) => writeln!(f, "\t\tf64.const {}", v)?,
                Value::I32(v) => writeln!(f, "\t\ti32.const {}", v)?,
                Value::Bool(true) => writeln!(f, "\t\ti32.const 1")?,
                Value::Bool(false) => writeln!(f, "\t\ti32.const 0")?,
                _ => unimplemented!(),
            },
            BlockData::Assign(_, inst) => match inst {
                Inst::Neg(val) => match func.ir.var_type[*val] {
                    Type::I32 => {
                        writeln!(f, "\t\ti32.const 0")?;
                        add(f, func, func.ir.var_decl[*val])?;
                        writeln!(f, "\t\ti32.sub")?;
                    }
                    Type::F64 => {
                        add(f, func, func.ir.var_decl[*val])?;
                        writeln!(f, "\t\tf64.neg")?;
                    }
                    _ => unimplemented!(),
                },
                Inst::Add(a, b) => match func.ir.var_type[*a] {
                    Type::I32 => {
                        add(f, func, func.ir.var_decl[*a])?;
                        add(f, func, func.ir.var_decl[*b])?;
                        writeln!(f, "\t\ti32.add")?;
                    }
                    Type::F64 => {
                        add(f, func, func.ir.var_decl[*a])?;
                        add(f, func, func.ir.var_decl[*b])?;
                        writeln!(f, "\t\tf64.add")?;
                    }
                    _ => unimplemented!(),
                },
                Inst::Sub(a, b) => match func.ir.var_type[*a] {
                    Type::I32 => {
                        add(f, func, func.ir.var_decl[*a])?;
                        add(f, func, func.ir.var_decl[*b])?;
                        writeln!(f, "\t\ti32.sub")?;
                    }
                    Type::F64 => {
                        add(f, func, func.ir.var_decl[*a])?;
                        add(f, func, func.ir.var_decl[*b])?;
                        writeln!(f, "\t\tf64.sub")?;
                    }
                    _ => unimplemented!(),
                },
                Inst::Mul(a, b) => match func.ir.var_type[*a] {
                    Type::I32 => {
                        add(f, func, func.ir.var_decl[*a])?;
                        add(f, func, func.ir.var_decl[*b])?;
                        writeln!(f, "\t\ti32.mul")?;
                    }
                    Type::F64 => {
                        add(f, func, func.ir.var_decl[*a])?;
                        add(f, func, func.ir.var_decl[*b])?;
                        writeln!(f, "\t\tf64.mul")?;
                    }
                    _ => unimplemented!(),
                },
                Inst::Div(a, b) => match func.ir.var_type[*a] {
                    Type::I32 => {
                        add(f, func, func.ir.var_decl[*a])?;
                        add(f, func, func.ir.var_decl[*b])?;
                        writeln!(f, "\t\ti32.div_s")?;
                    }
                    Type::F64 => {
                        add(f, func, func.ir.var_decl[*a])?;
                        add(f, func, func.ir.var_decl[*b])?;
                        writeln!(f, "\t\tf64.div_s")?;
                    }
                    _ => unimplemented!(),
                },
                Inst::Eq(a, b) => match func.ir.var_type[*a] {
                    Type::I32 | Type::Bool => {
                        add(f, func, func.ir.var_decl[*a])?;
                        add(f, func, func.ir.var_decl[*b])?;
                        writeln!(f, "\t\ti32.eq")?;
                    }
                    Type::F64 => {
                        add(f, func, func.ir.var_decl[*a])?;
                        add(f, func, func.ir.var_decl[*b])?;
                        writeln!(f, "\t\tf64.eq")?;
                    }
                },
                _ => {}
            },
            _ => {}
        };

        return Ok(());
    }

    for (i, func) in module.funcs.iter().enumerate() {
        writeln!(f, "\t(func ${}", i)?;
        writeln!(f, "\t\t(export \"{}\")", func.name)?;
        writeln!(f, "\t\t(result {})", rep(func.return_type))?;

        if let Some(BlockData::Return(var)) = func.ir.blocks.last() {
            let decl = func.ir.var_decl[*var];
            add(&mut f, func, decl)?;
        } else {
            panic!("expeted function to end with return statment!")
        }

        writeln!(f, "\t)")?;
    }

    writeln!(f, ")")?;

    return Ok(f);
}
