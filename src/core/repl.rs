use std::collections::HashMap;

use crate::core::*;
use crate::utils::*;

struct Regs {
    mem: Mem,
    current_reg: usize,
    vars: HashMap<usize, usize>,
}

impl Regs {
    fn new() -> Regs {
        return Regs {
            mem: Mem::default(),
            current_reg: 0,
            vars: HashMap::new(),
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
}

pub fn exec_ir(func: &Func, funcs: &Vec<Func>, mem: &mut Mem, args: Vec<Value>) -> Value {
    let mut step = 0;
    let mut regs = Regs::new();

    for (i, arg) in args.into_iter().enumerate() {
        regs.assign(&i, &arg);
    }

    loop {
        step += 1;
        match &func.ir.insts[step - 1] {
            Inst::Op(var, op, a, b) => {
                // regs[*var] = match op {
                //     Op::Add => regs[*a].add(regs[*b].clone()),
                //     Op::Sub => regs[*a].sub(regs[*b].clone()),
                //     Op::Mul => regs[*a].mul(regs[*b].clone()),
                //     Op::Div => regs[*a].div(regs[*b].clone()),
                //     Op::Eq => regs[*a].eq(regs[*b].clone()),
                //     Op::Ne => regs[*a].ne(regs[*b].clone()),
                //     Op::Le => regs[*a].le(regs[*b].clone()),
                //     Op::Lt => regs[*a].lt(regs[*b].clone()),
                //     Op::Ge => regs[*a].ge(regs[*b].clone()),
                //     Op::Gt => regs[*a].gt(regs[*b].clone()),
                // };

                unimplemented!()
            }
            Inst::UOp(var, op, a) => {
                // regs[*var] = match op {
                //     UOp::Neg => regs[*a].neg(),
                //     UOp::Not => regs[*a].not(),
                // };

                unimplemented!();
            }
            Inst::Const(var, val) => {
                regs.assign(var, val);
            }
            Inst::Call(var, func_id_reg, param_regs) => {
                // let args = param_regs.iter().map(|reg| regs[*reg].clone()).collect();
                // regs[*var] = exec_ir(&funcs[*func_id_reg], funcs, mem, args);

                unimplemented!();
            }
            Inst::JumpTo(block, args) => {
                // step = func.ir.blocks[*block];

                // let (first_param, num_params) = func.ir.block_params[*block];

                // for i in 0..num_params {
                //     regs[first_param + i] = regs[args[i]].clone();
                // }

                unimplemented!()
            }
            Inst::Branch(cond, (a, b)) => {
                // if regs[*cond].as_bool() {
                //     step = func.ir.blocks[*a];
                // } else {
                //     step = func.ir.blocks[*b];
                // }

                unimplemented!();
            }
            Inst::Return(var) => {
                // return regs[*var].clone();

                unimplemented!()
            }
        }
    }
}
