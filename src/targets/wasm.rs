use crate::core::*;
use crate::utils::*;

use std::io::Write;

impl TypeDef {
    fn to_wat(&self) -> &str {
        match self {
            TypeDef::I32 => "i32",
            TypeDef::Bool => "i32",
            TypeDef::F64 => "f64",
            _ => unimplemented!(),
        }
    }

    fn to_wasm(&self) -> u8 {
        match self {
            TypeDef::Bool | TypeDef::I32 => 0x7F,
            TypeDef::F64 => 0x7C,
            _ => unimplemented!(),
        }
    }
}

const TAB: &'static str = "\t";

impl<'a> Module<'a> {
    pub fn to_wat(&self) -> Vec<u8> {
        let mut b = vec![];

        // open module
        let _ = writeln!(b, "(module");

        // add the funcs
        for (i, func) in self.funcs.iter().enumerate() {
            // open function
            let _ = writeln!(b, "(func ${}", i);

            // all functions should be exported
            let _ = writeln!(b, "{TAB}(export \"{}\")", func.name);

            // add params
            for var in 0..func.num_params {
                let _ = writeln!(b, "{TAB}(param ${var} {})", func.ir.var_type[var].to_wat());
            }

            // result type
            let _ = writeln!(b, "{TAB}(result {})", func.return_type.to_wat());

            // add locals
            for var in func.num_params..func.ir.num_vars {
                let _ = writeln!(b, "{TAB}(local ${var} {})", func.ir.var_type[var].to_wat());
            }

            // add code
            let mut builder = WatBuilder::new();
            build(&mut builder, func);
            b.append(&mut builder.buffer);

            // close function
            let _ = writeln!(b, ")");
        }

        // close module
        let _ = writeln!(b, ")");

        return b;
    }

    pub fn to_wasm(&self) -> Vec<u8> {
        let mut b = vec![];

        b.append(&mut vec![0x00, 0x61, 0x73, 0x6D]); // magic number
        b.append(&mut vec![0x01, 0x00, 0x00, 0x00]); // version number

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
                for byte in name.bytes() {
                    b.push(byte);
                }

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

                    // add code
                    let mut builder = WasmBuilder::new();
                    build(&mut builder, func);
                    b.append(&mut builder.buffer);

                    // end inst
                    b.push(0x0B);
                });
            }
        });

        return b;
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

fn build(builder: &mut impl WasmOrWatBuilder, func: &Func) {
    reloop(builder, func, 0);
    builder.add_inst(WasmInst::Unreachable)
}

fn reloop(f: &mut impl WasmOrWatBuilder, func: &Func, block: usize) -> Option<usize> {
    let next_block = if is_loop(func, block) {
        f.start_loop();
        let next_block = add_block(f, func, block);
        f.close_loop();
        next_block
    } else {
        add_block(f, func, block)
    };

    return next_block;
}

fn add_block(f: &mut impl WasmOrWatBuilder, func: &Func, block: usize) -> Option<usize> {
    for inst in &func.ir.insts[func.ir.blocks[block]..] {
        match inst {
            Inst::Call(var, call, args) => {
                for arg in args {
                    f.get_local(*arg);
                }
                f.add_func_call(*call);
                f.set_local(*var);
            }
            Inst::Op(var, op, a, b) => {
                f.get_local(*a);
                f.get_local(*b);
                match (op, func.ir.var_type[*a].clone()) {
                    (Op::Add, TypeDef::I32) => f.add_inst(WasmInst::I32Add),
                    (Op::Add, TypeDef::F64) => f.add_inst(WasmInst::F64Add),
                    (Op::Sub, TypeDef::I32) => f.add_inst(WasmInst::I32Sub),
                    (Op::Sub, TypeDef::F64) => f.add_inst(WasmInst::F64Sub),
                    (Op::Mul, TypeDef::I32) => f.add_inst(WasmInst::I32Mul),
                    (Op::Mul, TypeDef::F64) => f.add_inst(WasmInst::F64Mul),
                    (Op::Div, TypeDef::I32) => f.add_inst(WasmInst::I32DivS),
                    (Op::Div, TypeDef::F64) => f.add_inst(WasmInst::F64DivS),
                    (Op::Eq, TypeDef::Bool) => f.add_inst(WasmInst::I32Eq),
                    (Op::Eq, TypeDef::I32) => f.add_inst(WasmInst::I32Eq),
                    (Op::Eq, TypeDef::F64) => f.add_inst(WasmInst::F64Eq),
                    (Op::Ne, TypeDef::Bool) => f.add_inst(WasmInst::I32Ne),
                    (Op::Ne, TypeDef::I32) => f.add_inst(WasmInst::I32Ne),
                    (Op::Ne, TypeDef::F64) => f.add_inst(WasmInst::F64Ne),
                    (Op::Ge, TypeDef::I32) => f.add_inst(WasmInst::I32GeS),
                    (Op::Ge, TypeDef::F64) => f.add_inst(WasmInst::F64GeS),
                    (Op::Gt, TypeDef::I32) => f.add_inst(WasmInst::I32GtS),
                    (Op::Gt, TypeDef::F64) => f.add_inst(WasmInst::F64GtS),
                    (Op::Le, TypeDef::I32) => f.add_inst(WasmInst::I32LeS),
                    (Op::Le, TypeDef::F64) => f.add_inst(WasmInst::F64LeS),
                    (Op::Lt, TypeDef::I32) => f.add_inst(WasmInst::I32LtS),
                    (Op::Lt, TypeDef::F64) => f.add_inst(WasmInst::F64LtS),

                    _ => unimplemented!(),
                }
                f.set_local(*var);
            }
            Inst::UOp(var, op, a) => {
                match op {
                    UOp::Neg => match func.ir.var_type[*a] {
                        TypeDef::I32 => {
                            f.add_const_i32(0);
                            f.get_local(*a);
                            f.add_inst(WasmInst::I32Sub);
                        }
                        TypeDef::F64 => {
                            f.get_local(*a);
                            f.add_inst(WasmInst::F64Neg);
                        }
                        _ => unimplemented!(),
                    },
                    UOp::Not => unimplemented!(),
                }
                f.set_local(*var);
            }
            Inst::Const(var, val) => {
                match val.get_type() {
                    TypeDef::Bool => f.add_const_i32(if val.as_bool() { 1 } else { 0 }),
                    TypeDef::F64 => f.add_const_f64(val.as_f64()),
                    TypeDef::I32 => f.add_const_i32(val.as_i32()),
                    _ => unimplemented!(),
                }
                f.set_local(*var);
            }
            Inst::Return(var) => {
                f.get_local(*var);
                f.add_return();

                return None;
            }
            Inst::Branch(cond, (a, b)) => {
                f.get_local(*cond);

                f.if_block();
                let a = reloop(f, func, *a);
                f.else_block();
                let b = reloop(f, func, *b);
                f.end_block();

                return match (a, b) {
                    _ => None,
                };
            }
            Inst::JumpTo(target, args) => {
                // pass the paramaters
                let start = func.ir.block_params[*target].0;
                for i in 0..args.len() {
                    f.get_local(args[i]);
                    f.set_local(start + i);
                }

                // move on to the next block
                if is_parent_of(func, *target, block) {
                    f.add_break(1);
                    return None;
                } else if dominates(func, block, *target) {
                    return reloop(f, func, *target);
                } else {
                    return Some(*target);
                }
            }
        };
    }

    panic!("Block didn't end!")
}

///
struct WasmBuilder {
    buffer: Vec<u8>,
}

impl WasmBuilder {
    fn new() -> Self {
        return WasmBuilder { buffer: vec![] };
    }
}

impl WasmOrWatBuilder for WasmBuilder {
    fn start_loop(&mut self) {
        self.buffer.push(0x03); // this is a loop
        self.buffer.push(0x40); // that returns nothing
    }

    fn if_block(&mut self) {
        self.buffer.push(0x04); // if
        self.buffer.push(0x40); // returns nothing
    }

    fn else_block(&mut self) {
        self.buffer.push(0x05); // else
    }

    fn end_block(&mut self) {
        self.buffer.push(0x0B); // end
    }

    fn close_loop(&mut self) {
        self.end_block()
    }

    fn get_local(&mut self, var: usize) {
        self.buffer.push(0x20);
        var.write_leb128(&mut self.buffer);
    }

    fn set_local(&mut self, var: usize) {
        self.buffer.push(0x21);
        var.write_leb128(&mut self.buffer);
    }

    fn add_break(&mut self, label: usize) {
        self.buffer.push(0x0C);
        label.write_leb128(&mut self.buffer);
    }

    fn add_func_call(&mut self, func_id: usize) {
        self.buffer.push(0x10); // func call inst
        func_id.write_leb128(&mut self.buffer);
    }

    fn add_const_f64(&mut self, value: f64) {
        self.buffer.push(0x44);
        for byte in value.to_le_bytes() {
            self.buffer.push(byte);
        }
    }

    fn add_const_i32(&mut self, value: i32) {
        self.buffer.push(0x41);
        value.write_leb128(&mut self.buffer);
    }

    fn add_return(&mut self) {
        self.buffer.push(0x0F);
    }

    fn add_inst(&mut self, inst: WasmInst) {
        match inst {
            WasmInst::I32Add => self.buffer.push(0x6A),
            WasmInst::I32Sub => self.buffer.push(0x6B),
            WasmInst::I32Mul => self.buffer.push(0x6C),
            WasmInst::I32DivS => self.buffer.push(0x6D),

            WasmInst::F64Add => self.buffer.push(0xA0),
            WasmInst::F64Sub => self.buffer.push(0xA1),
            WasmInst::F64Mul => self.buffer.push(0xA2),
            WasmInst::F64DivS => self.buffer.push(0xA3),

            WasmInst::I32Eq => self.buffer.push(0x46),
            WasmInst::F64Eq => self.buffer.push(0x61),

            WasmInst::I32Ne => self.buffer.push(0x47),
            WasmInst::F64Ne => self.buffer.push(0x62),

            WasmInst::I32GeS => self.buffer.push(0x4E),
            WasmInst::I32GtS => self.buffer.push(0x4A),
            WasmInst::I32LeS => self.buffer.push(0x4C),
            WasmInst::I32LtS => self.buffer.push(0x48),

            WasmInst::F64GeS => self.buffer.push(0x66),
            WasmInst::F64GtS => self.buffer.push(0x64),
            WasmInst::F64LeS => self.buffer.push(0x65),
            WasmInst::F64LtS => self.buffer.push(0x63),

            WasmInst::F64Neg => self.buffer.push(0x9A),

            WasmInst::Unreachable => self.buffer.push(0x00),
        };
    }
}

///
struct WatBuilder {
    buffer: Vec<u8>,
    tab: usize,
}

impl WatBuilder {
    fn new() -> Self {
        return WatBuilder {
            buffer: vec![],
            tab: 2,
        };
    }

    fn write(&mut self, content: &str) {
        for _ in 1..self.tab {
            let _ = write!(self.buffer, "{TAB}");
        }
        let _ = writeln!(self.buffer, "{content}");
    }
}

impl WasmOrWatBuilder for WatBuilder {
    fn start_loop(&mut self) {
        self.write("(loop");
        self.tab += 1;
    }

    fn if_block(&mut self) {
        self.write("(if (then");
        self.tab += 1;
    }

    fn else_block(&mut self) {
        self.tab -= 1;
        self.write(") (else");
        self.tab += 1;
    }

    fn end_block(&mut self) {
        self.tab -= 1;
        self.write("))");
    }

    fn close_loop(&mut self) {
        self.tab -= 1;
        self.write(")");
    }

    fn get_local(&mut self, var: usize) {
        self.write(&format!("get_local {var}"));
    }

    fn set_local(&mut self, var: usize) {
        self.write(&format!("set_local {var}"));
    }

    fn add_break(&mut self, label: usize) {
        self.write(&format!("br {label}"));
    }

    fn add_func_call(&mut self, func_id: usize) {
        self.write(&format!("call {func_id}"));
    }

    fn add_const_f64(&mut self, value: f64) {
        self.write(&format!("f64.const {value}"));
    }

    fn add_const_i32(&mut self, value: i32) {
        self.write(&format!("i32.const {value}"));
    }

    fn add_return(&mut self) {
        self.write(&format!("return"));
    }

    fn add_inst(&mut self, inst: WasmInst) {
        match inst {
            WasmInst::I32Add => self.write("i32.add"),
            WasmInst::F64Add => self.write("f64.add"),
            WasmInst::I32Sub => self.write("i32.sub"),
            WasmInst::F64Sub => self.write("f64.sub"),
            WasmInst::I32Mul => self.write("i32.mul"),
            WasmInst::F64Mul => self.write("f64.mul"),
            WasmInst::I32DivS => self.write("i32.div_s"),
            WasmInst::F64DivS => self.write("f64.div_s"),
            WasmInst::I32Eq => self.write("i32.eq"),
            WasmInst::F64Eq => self.write("f64.eq"),
            WasmInst::I32Ne => self.write("i32.new"),
            WasmInst::F64Ne => self.write("f64.ne"),
            WasmInst::I32GeS => self.write("i32.ge_s"),
            WasmInst::F64GeS => self.write("f64.ge_s"),
            WasmInst::I32GtS => self.write("i32.gt_s"),
            WasmInst::F64GtS => self.write("f64.gt_s"),
            WasmInst::I32LeS => self.write("i32.le_s"),
            WasmInst::F64LeS => self.write("f64.le_s"),
            WasmInst::I32LtS => self.write("i32.lt_s"),
            WasmInst::F64LtS => self.write("f64.lt_s"),
            WasmInst::F64Neg => self.write("f64.neg"),
            WasmInst::Unreachable => self.write("unreachable"),
        };
    }
}

///
trait WasmOrWatBuilder {
    fn start_loop(&mut self);
    fn close_loop(&mut self);

    fn if_block(&mut self);
    fn else_block(&mut self);
    fn end_block(&mut self);

    fn get_local(&mut self, var: usize);
    fn set_local(&mut self, var: usize);

    fn add_break(&mut self, label: usize);
    fn add_func_call(&mut self, func_id: usize);
    fn add_const_i32(&mut self, value: i32);
    fn add_const_f64(&mut self, value: f64);
    fn add_return(&mut self);
    fn add_inst(&mut self, inst: WasmInst);
}

///
enum WasmInst {
    I32Add,
    F64Add,
    I32Sub,
    F64Sub,
    I32Mul,
    F64Mul,
    I32DivS,
    F64DivS,
    I32Eq,
    F64Eq,
    I32Ne,
    F64Ne,
    I32GeS,
    F64GeS,
    I32GtS,
    F64GtS,
    I32LeS,
    F64LeS,
    I32LtS,
    F64LtS,
    F64Neg,
    Unreachable,
}
