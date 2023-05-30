use std::collections::HashMap;

use crate::core::*;
use crate::utils::*;

struct Regs<'a> {
    mem: Mem,
    current_reg: usize,
    vars: HashMap<usize, usize>,
    func: &'a Func,
}

impl<'a> Regs<'a> {
    fn new(func: &Func) -> Regs {
        return Regs {
            mem: Mem::default(),
            current_reg: 0,
            vars: HashMap::new(),
            func,
        };
    }

    fn assign(&mut self, var: &usize, val: &Value) {
        if let Some(reg) = self.vars.get(var) {
            self.mem.set(*reg, val.get_bytes());
        } else {
            self.vars.insert(*var, self.current_reg);
            self.mem.set(self.current_reg, val.get_bytes());
            self.current_reg += val.get_size();
        }
    }

    fn get(&self, var: &usize) -> Value {
        return self
            .mem
            .get(*self.vars.get(var).unwrap(), self.func.get_var_type(*var));
    }
}

pub fn exec_ir(func: &Func, funcs: &Vec<Func>, mem: &mut Mem, args: Vec<Value>) -> Value {
    let mut step = 0;
    let mut regs = Regs::new(func);

    for (i, arg) in args.into_iter().enumerate() {
        regs.assign(&i, &arg);
    }

    loop {
        step += 1;
        match &func.ir.insts[step - 1] {
            Inst::Op(var, op, a, b) => {
                let a = regs.get(a);
                let b = regs.get(b);

                regs.assign(var, &do_op(op, a, b));
            }
            Inst::UOp(var, op, a) => {
                let a = regs.get(a);

                regs.assign(var, &do_uop(op, a));
            }
            Inst::Const(var, val) => {
                regs.assign(var, val);
            }
            Inst::Call(var, func_id_reg, param_regs) => {
                let args = param_regs.iter().map(|var| regs.get(var)).collect();
                regs.assign(var, &exec_ir(&funcs[*func_id_reg], funcs, mem, args));
            }
            Inst::JumpTo(block, args) => {
                step = func.ir.blocks[*block];

                let (first_param, num_params) = func.ir.block_params[*block];

                for i in 0..num_params {
                    regs.assign(&(first_param + i), &regs.get(&args[i]));
                }
            }
            Inst::Branch(cond, (a, b)) => {
                if regs.get(cond).as_bool() {
                    step = func.ir.blocks[*a];
                } else {
                    step = func.ir.blocks[*b];
                }
            }
            Inst::Return(var) => {
                return regs.get(var);
            }
        }
    }
}

fn do_op(op: &Op, a: Value, b: Value) -> Value {
    match (op, a.get_type(), b.get_type()) {
        (Op::Eq, TypeDef::Bool, TypeDef::Bool) => Value::bool(a.as_bool() == b.as_bool()),
        (Op::Ne, TypeDef::Bool, TypeDef::Bool) => Value::bool(a.as_bool() != b.as_bool()),

        (Op::Add, TypeDef::I32, TypeDef::I32) => Value::i32(a.as_i32() + b.as_i32()),
        (Op::Sub, TypeDef::I32, TypeDef::I32) => Value::i32(a.as_i32() - b.as_i32()),
        (Op::Mul, TypeDef::I32, TypeDef::I32) => Value::i32(a.as_i32() * b.as_i32()),
        (Op::Div, TypeDef::I32, TypeDef::I32) => Value::i32(a.as_i32() / b.as_i32()),

        (Op::Eq, TypeDef::I32, TypeDef::I32) => Value::bool(a.as_i32() == b.as_i32()),
        (Op::Ne, TypeDef::I32, TypeDef::I32) => Value::bool(a.as_i32() != b.as_i32()),
        (Op::Le, TypeDef::I32, TypeDef::I32) => Value::bool(a.as_i32() <= b.as_i32()),
        (Op::Lt, TypeDef::I32, TypeDef::I32) => Value::bool(a.as_i32() < b.as_i32()),
        (Op::Ge, TypeDef::I32, TypeDef::I32) => Value::bool(a.as_i32() >= b.as_i32()),
        (Op::Gt, TypeDef::I32, TypeDef::I32) => Value::bool(a.as_i32() > b.as_i32()),

        (Op::Add, TypeDef::F64, TypeDef::F64) => Value::f64(a.as_f64() + b.as_f64()),
        (Op::Sub, TypeDef::F64, TypeDef::F64) => Value::f64(a.as_f64() - b.as_f64()),
        (Op::Mul, TypeDef::F64, TypeDef::F64) => Value::f64(a.as_f64() * b.as_f64()),
        (Op::Div, TypeDef::F64, TypeDef::F64) => Value::f64(a.as_f64() / b.as_f64()),

        (Op::Eq, TypeDef::F64, TypeDef::F64) => Value::bool(a.as_f64() == b.as_f64()),
        (Op::Ne, TypeDef::F64, TypeDef::F64) => Value::bool(a.as_f64() != b.as_f64()),
        (Op::Le, TypeDef::F64, TypeDef::F64) => Value::bool(a.as_f64() <= b.as_f64()),
        (Op::Lt, TypeDef::F64, TypeDef::F64) => Value::bool(a.as_f64() < b.as_f64()),
        (Op::Ge, TypeDef::F64, TypeDef::F64) => Value::bool(a.as_f64() >= b.as_f64()),
        (Op::Gt, TypeDef::F64, TypeDef::F64) => Value::bool(a.as_f64() > b.as_f64()),

        _ => unimplemented!(),
    }
}

fn do_uop(op: &UOp, a: Value) -> Value {
    match (op, a.get_type()) {
        (UOp::Neg, TypeDef::I32) => Value::i32(-a.as_i32()),
        (UOp::Neg, TypeDef::F64) => Value::f64(-a.as_f64()),

        (UOp::Not, TypeDef::Bool) => Value::bool(!a.as_bool()),

        _ => unimplemented!(),
    }
}
