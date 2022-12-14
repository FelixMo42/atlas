use crate::ir::*;
use crate::module::*;
use crate::value::*;

use std::collections::HashSet;
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

        // add locals
        for var in 0..func.ir.num_vars {
            writeln!(f, "\t\t(local ${var} {})", rep(func.ir.var_type[var]))?;
        }

        reloop(&mut f, func, 0)?;

        match func.return_type {
            Type::I32 => writeln!(f, "i32.const {}", i32::MAX)?,
            Type::Bool => writeln!(f, "i32.const {}", i32::MAX)?,
            Type::F64 => writeln!(f, "f64.const {}", f64::MAX)?,
        };

        writeln!(f, "\t)")?;
    }

    writeln!(f, ")")?;

    return Ok(f);
}

/// Get the final instruction of a block.
fn get_exit_inst(func: &Func, block: usize) -> Inst {
    for inst in &func.ir.insts[func.ir.blocks[block]..] {
        match inst {
            Inst::Return(..) | Inst::Branch(..) | Inst::JumpTo(..) => return inst.clone(),
            _ => {}
        };
    }

    panic!("Block didn't end!")
}

/// Get the list of immidate children of <block>.
fn get_children(func: &Func, block: usize) -> Vec<usize> {
    match get_exit_inst(func, block) {
        Inst::Return(..) => vec![],
        Inst::Branch(_, (a, b)) => vec![a, b],
        Inst::JumpTo(target, _) => vec![target],
        _ => unreachable!(),
    }
}

/// Does <a> lead into <b>?
fn is_parent_of(func: &Func, a: usize, b: usize) -> bool {
    let mut todo = vec![a];
    let mut seen = HashSet::new();

    while let Some(block) = todo.pop() {
        for child in get_children(func, block) {
            if child == b {
                return true;
            } else if !seen.contains(&b) {
                seen.insert(b);
                todo.push(b);
            }
        }
    }

    return false;
}

/// Do all path to <b> go throght <a>?
fn dominates(f: &Func, a: usize, b: usize) -> bool {
    return true;
}

/// Does a child of <block> point towards <block>?
fn is_loop(f: &Func, block: usize) -> bool {
    for inst in &f.ir.insts[f.ir.blocks[block]..] {
        match inst {
            Inst::Branch(_, (a, b)) => {
                if *a == block || *b == block {
                    return true;
                }
            }
            Inst::JumpTo(target, _) => {
                if *target == block {
                    return true;
                }
            }
            _ => {}
        }
    }

    return false;
}

fn reloop(f: &mut Vec<u8>, func: &Func, block: usize) -> std::io::Result<Option<usize>> {
    let next_block = if is_loop(func, block) {
        writeln!(f, "\t(loop")?;
        let next_block = add_block(f, func, block);
        writeln!(f, "\t)")?;
        next_block
    } else {
        add_block(f, func, block)
    };

    return next_block;
}

fn add_block(f: &mut Vec<u8>, func: &Func, block: usize) -> std::io::Result<Option<usize>> {
    for inst in &func.ir.insts[func.ir.blocks[block]..] {
        match inst {
            Inst::Call(var, call, args) => unimplemented!(),
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

                    (Op::Ne, Type::Bool) => writeln!(f, "\t\ti32.ne")?,
                    (Op::Ne, Type::I32) => writeln!(f, "\t\ti32.ne")?,
                    (Op::Ne, Type::F64) => writeln!(f, "\t\tf64.ne")?,

                    (Op::Ge, Type::I32) => writeln!(f, "\t\ti32.ge_s")?,
                    (Op::Ge, Type::F64) => writeln!(f, "\t\tf64.ge_s")?,
                    (Op::Gt, Type::I32) => writeln!(f, "\t\ti32.gt_s")?,
                    (Op::Gt, Type::F64) => writeln!(f, "\t\tf64.gt_s")?,

                    (Op::Le, Type::I32) => writeln!(f, "\t\ti32.le_s")?,
                    (Op::Le, Type::F64) => writeln!(f, "\t\tf64.le_s")?,
                    (Op::Lt, Type::I32) => writeln!(f, "\t\ti32.lt_s")?,
                    (Op::Lt, Type::F64) => writeln!(f, "\t\tf64.lt_s")?,

                    _ => unimplemented!(),
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
                writeln!(f, "\t\treturn")?;

                return Ok(None);
            }
            Inst::Branch(cond, (a, b)) => {
                writeln!(f, "\t\tget_local ${}", cond)?;
                writeln!(f, "\t\t(if")?;
                writeln!(f, "\t\t(then")?;
                let a = reloop(f, func, *a)?;
                writeln!(f, "\t\t)")?;
                writeln!(f, "\t\t(else")?;
                let b = reloop(f, func, *b)?;
                writeln!(f, "\t\t)")?;
                writeln!(f, "\t\t)")?;

                return match (a, b) {
                    _ => Ok(None),
                };
            }
            Inst::JumpTo(target, args) => {
                // pass the paramaters
                let start = func.ir.block_params[*target].0;
                for i in 0..args.len() {
                    writeln!(f, "\t\tget_local ${}", args[i])?;
                    writeln!(f, "\t\tset_local ${}", start + i)?;
                }

                // move on to the next block
                if is_parent_of(func, *target, block) {
                    writeln!(f, "\t\tbr 1")?;
                    return Ok(None);
                } else if dominates(func, block, *target) {
                    return reloop(f, func, *target);
                } else {
                    return Ok(Some(*target));
                }
            }
        };
    }

    panic!("Block didn't end!")
}
