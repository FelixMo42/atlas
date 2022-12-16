use crate::ir::*;
use crate::leb128::*;
use crate::module::*;
use crate::utils::*;
use crate::value::*;

use std::io::Write;

impl Type {
    fn to_wat(&self) -> &str {
        match self {
            Type::I32 => "i32",
            Type::Bool => "i32",
            Type::F64 => "f64",
        }
    }

    fn to_wasm(&self) -> u8 {
        match self {
            Type::Bool | Type::I32 => 0x7F,
            Type::F64 => 0x7C,
        }
    }
}

impl<'a> Module<'a> {
    pub fn to_wat(&self) -> std::io::Result<String> {
        let mut b = vec![];

        // open module
        writeln!(b, "(module")?;

        // add the funcs
        for (i, func) in self.funcs.iter().enumerate() {
            // open function
            writeln!(b, "\t(func ${}", i)?;

            // all functions should be exported
            writeln!(b, "\t\t(export \"{}\")", func.name)?;

            // add params
            for var in 0..func.num_params {
                writeln!(b, "\t\t(param ${var} {})", func.ir.var_type[var].to_wat())?;
            }

            // result type
            writeln!(b, "\t\t(result {})", func.return_type.to_wat())?;

            // add locals
            for var in func.num_params..func.ir.num_vars {
                writeln!(b, "\t\t(local ${var} {})", func.ir.var_type[var].to_wat())?;
            }

            // add code
            reloop(&mut b, func, 0)?;

            // we need to add a return value at the end to satify the checks
            match func.return_type {
                Type::I32 => writeln!(b, "i32.const {}", i32::MAX)?,
                Type::Bool => writeln!(b, "i32.const {}", i32::MAX)?,
                Type::F64 => writeln!(b, "f64.const {}", f64::MAX)?,
            };

            // close function
            writeln!(b, "\t)")?;
        }

        // close module
        writeln!(b, ")")?;

        return Ok(String::from_utf8(b).unwrap());
    }

    pub fn to_wasm(&self) -> std::io::Result<Vec<u8>> {
        let mut b = vec![];

        write!(b, "\x00\x61\x73\x6D")?; // magic number
        write!(b, "\x01\x00\x00\x00")?; // version number

        add_section(&mut b, WASM_TYPE_SECTION, |b| {
            self.funcs.len().write_leb128(b); // how many types?

            for func in &self.funcs {
                b.push(0x60);

                func.num_params.write_leb128(b); // how many params?
                for i in 0..func.num_params {
                    b.push(func.ir.var_type[i].to_wasm());
                }

                1usize.write_leb128(b); // how many values returns?
                b.push(func.return_type.to_wasm());
            }
        });

        add_section(&mut b, WASM_FUNCTION_SECTION, |b| {
            self.funcs.len().write_leb128(b); // how many functions?

            for i in 0..self.funcs.len() {
                i.write_leb128(b);
            }
        });

        add_section(&mut b, WASM_EXPORT_SECTION, |b| {
            self.funcs.len().write_leb128(b); // how many functions exported?

            for i in 0..self.funcs.len() {
                // write the name
                let name = &self.funcs[i].name;
                name.as_bytes().len().write_leb128(b);
                write!(b, "{}", name);

                b.push(0x00); // were exporting a function
                i.write_leb128(b); // function id
            }
        });

        add_section(&mut b, WASM_CODE_SECTION, |b| {
            self.funcs.len().write_leb128(b); // how many functions?

            for func in &self.funcs {
                write_with_length(b, |b| {
                    func.ir.num_vars.write_leb128(b); // how many locals?
                    for i in 0..func.ir.num_vars {
                        1usize.write_leb128(b); // how many of this type
                        b.push(func.ir.var_type[i].to_wasm()); // local type
                    }

                    // body
                    reloop_bin(b, func, 0);

                    // Tell wasm this is unreachable so it dosent complain
                    // about not having values in the stack.
                    b.push(0x00);

                    // end inst
                    b.push(0x0B);
                });
            }
        });

        return Ok(b);
    }
}

fn write_with_length(b: &mut Vec<u8>, builder: impl FnOnce(&mut Vec<u8>) -> ()) {
    let mut content = vec![];
    builder(&mut content);
    content.len().write_leb128(b);
    b.append(&mut content);
}

fn add_section(b: &mut Vec<u8>, section_id: u8, builder: impl FnOnce(&mut Vec<u8>) -> ()) {
    b.push(section_id);
    write_with_length(b, builder);
}

const WASM_TYPE_SECTION: u8 = 1;
const WASM_FUNCTION_SECTION: u8 = 3;
const WASM_EXPORT_SECTION: u8 = 7;
const WASM_CODE_SECTION: u8 = 10;

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
            Inst::Call(var, call, args) => {
                for arg in args {
                    writeln!(f, "\t\tget_local ${arg}")?;
                }
                writeln!(f, "\t\tcall ${call}")?;
                writeln!(f, "\t\tset_local ${var}")?;
            }
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
                    _ => unreachable!(),
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

fn reloop_bin(f: &mut Vec<u8>, func: &Func, block: usize) -> Option<usize> {
    let next_block = if is_loop(func, block) {
        f.push(0x03);
        f.push(0x40); // block type, empty for now
        let next_block = add_block_bin(f, func, block);
        f.push(0x0B);
        next_block
    } else {
        add_block_bin(f, func, block)
    };

    return next_block;
}

fn set_local(f: &mut Vec<u8>, local: usize) {
    f.push(0x21);
    local.write_leb128(f);
}

fn get_local(f: &mut Vec<u8>, local: usize) {
    f.push(0x20);
    local.write_leb128(f);
}

fn add_i32_const(f: &mut Vec<u8>, value: i32) {
    f.push(0x41);
    value.write_leb128(f);
}

fn add_f64_const(f: &mut Vec<u8>, value: f64) {
    f.push(0x44);
    for byte in value.to_le_bytes() {
        f.push(byte);
    }
}

fn add_block_bin(f: &mut Vec<u8>, func: &Func, block: usize) -> Option<usize> {
    for inst in &func.ir.insts[func.ir.blocks[block]..] {
        match inst {
            Inst::Call(var, call, args) => {
                for arg in args {
                    get_local(f, *arg);
                }
                f.push(0x10); // func call inst
                call.write_leb128(f);
                set_local(f, *var);
            }
            Inst::Op(var, op, a, b) => {
                get_local(f, *a);
                get_local(f, *b);
                match (op, func.ir.var_type[*a]) {
                    (Op::Add, Type::I32) => f.push(0x6A),
                    (Op::Sub, Type::I32) => f.push(0x6B),
                    (Op::Mul, Type::I32) => f.push(0x6C),
                    (Op::Div, Type::I32) => f.push(0x6D),

                    (Op::Add, Type::F64) => f.push(0xA0),
                    (Op::Sub, Type::F64) => f.push(0xA1),
                    (Op::Mul, Type::F64) => f.push(0xA2),
                    (Op::Div, Type::F64) => f.push(0xA3),

                    (Op::Eq, Type::Bool) => f.push(0x46),
                    (Op::Eq, Type::I32) => f.push(0x46),
                    (Op::Eq, Type::F64) => f.push(0x61),

                    (Op::Ne, Type::Bool) => f.push(0x47),
                    (Op::Ne, Type::I32) => f.push(0x47),
                    (Op::Ne, Type::F64) => f.push(0x62),

                    (Op::Ge, Type::I32) => f.push(0x4E),
                    (Op::Gt, Type::I32) => f.push(0x4A),
                    (Op::Le, Type::I32) => f.push(0x4C),
                    (Op::Lt, Type::I32) => f.push(0x48),

                    (Op::Ge, Type::F64) => f.push(0x66),
                    (Op::Gt, Type::F64) => f.push(0x64),
                    (Op::Le, Type::F64) => f.push(0x65),
                    (Op::Lt, Type::F64) => f.push(0x63),

                    _ => unimplemented!(),
                }
                set_local(f, *var)
            }
            Inst::UOp(var, op, a) => {
                match op {
                    UOp::Neg => match func.ir.var_type[*a] {
                        Type::I32 => {
                            add_i32_const(f, 0);
                            get_local(f, *a);
                            f.push(0x6B); // i32.sub
                        }
                        Type::F64 => {
                            get_local(f, *a);
                            f.push(0x9A); // f64.neg
                        }
                        _ => unimplemented!(),
                    },
                    UOp::Not => unimplemented!(),
                }
                set_local(f, *var)
            }
            Inst::Const(var, val) => {
                match val {
                    Value::Bool(true) => add_i32_const(f, 1),
                    Value::Bool(false) => add_i32_const(f, 0),
                    Value::I32(val) => add_i32_const(f, *val),
                    Value::F64(val) => add_f64_const(f, *val),
                    _ => unreachable!(),
                }
                set_local(f, *var);
            }
            Inst::Return(var) => {
                get_local(f, *var);
                f.push(0x0F);

                return None;
            }
            Inst::Branch(cond, (a, b)) => {
                get_local(f, *cond); // if
                f.push(0x04); // then
                f.push(0x40); // block type, empty for now
                let a = reloop_bin(f, func, *a);
                f.push(0x05); // else
                let b = reloop_bin(f, func, *b);
                f.push(0x0B); // end

                return match (a, b) {
                    _ => None,
                };
            }
            Inst::JumpTo(target, args) => {
                // pass the paramaters
                let start = func.ir.block_params[*target].0;
                for i in 0..args.len() {
                    get_local(f, args[i]);
                    set_local(f, start + i);
                }

                // move on to the next block
                if is_parent_of(func, *target, block) {
                    f.push(0x0C);
                    1usize.write_leb128(f);
                    return None;
                } else if dominates(func, block, *target) {
                    return reloop_bin(f, func, *target);
                } else {
                    return Some(*target);
                }
            }
        };
    }

    panic!("Block didn't end!")
}
