use crate::module::*;
use crate::parser::*;
use crate::value::*;

type Var = usize;
type FuncId = usize;
type Block = usize;

#[derive(Debug)]
pub struct Func {
    pub name: String,
    pub return_type: Type,
    pub ir: Blocks,
}

impl Func {
    pub fn new(module: &Module, func_def: FuncDef) -> Func {
        let mut ir = Blocks::new(&func_def.params);

        let scope = &mut module.scope.child();

        for (i, param) in func_def.params.into_iter().enumerate() {
            scope.set(param.name, i);
            ir.var_decl.push(0); // TODO: !!
            ir.var_type.push(param.param_type);
        }

        ir.add(&func_def.body, scope);

        return Func {
            name: func_def.name,
            return_type: func_def.return_type,
            ir,
        };
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Op {
    // math ops
    Add,
    Sub,
    Mul,
    Div,

    // logical ops
    Eq,
    Ne,
    Le,
    Lt,
    Ge,
    Gt,
}

#[derive(Debug, Clone, Copy)]
pub enum UOp {
    Neg,
    Not,
}

#[derive(Debug, Clone)]
pub enum Inst {
    // operators
    Op(Var, Op, Var, Var),
    UOp(Var, UOp, Var),

    // misc
    Call(Var, FuncId, Vec<Var>),
    Const(Var, Value),

    // control flow
    Branch(Var, (Block, Block)),
    JumpTo(Block, Var),
    Return(Var),
}

// impl Inst {
//     fn get_type(&self, ir: &Blocks) -> Type {
//         match self {
//             Inst::Add(val, ..)
//             | Inst::Sub(val, ..)
//             | Inst::Div(val, ..)
//             | Inst::Mul(val, ..)
//             | Inst::Neg(val, ..)
//             | Inst::Move(val) => ir.var_type[*val],

//             Inst::Not(..)
//             | Inst::Eq(..)
//             | Inst::Ne(..)
//             | Inst::Le(..)
//             | Inst::Lt(..)
//             | Inst::Ge(..)
//             | Inst::Gt(..) => Type::Bool,

//             Inst::Call(..) => Type::Bool, // TODO: make this return correct type
//             _ => unimplemented!(),
//         }
//     }
// }

#[derive(Debug)]
pub struct Blocks {
    pub insts: Vec<Inst>,

    pub num_vars: usize,
    pub var_decl: Vec<usize>,
    pub var_type: Vec<Type>,

    pub blocks: Vec<usize>,
    pub block_params: Vec<usize>,
}

impl Blocks {
    fn new(params: &Vec<Param>) -> Self {
        return Blocks {
            insts: vec![],

            num_vars: params.len(),
            var_decl: vec![],
            var_type: vec![],

            blocks: vec![],
            block_params: vec![],
        };
    }

    fn new_var(&mut self) -> Var {
        self.var_decl.push(self.insts.len() + 1);
        self.num_vars += 1;
        return self.num_vars - 1;
    }

    fn new_block(&mut self) -> Block {
        self.blocks.push(0);
        self.block_params.push(0);
        return self.blocks.len() - 1;
    }

    fn add_label(&mut self, block: Block) -> Var {
        let var = self.new_var();
        self.blocks[block] = self.insts.len();
        self.block_params[block] = var;
        var
    }

    fn add_op(&mut self, op: Op, a: Var, b: Var) -> usize {
        let var = self.new_var();
        // self.var_type.push(inst.get_type(self));
        self.insts.push(Inst::Op(var, op, a, b));
        return var;
    }

    fn add_uop(&mut self, op: UOp, a: Var) -> usize {
        let var = self.new_var();
        // self.var_type.push(inst.get_type(self));
        self.insts.push(Inst::UOp(var, op, a));
        return var;
    }

    fn add_consts(&mut self, value: Value) -> usize {
        let reg = self.new_var();
        // self.var_type.push(value.get_type());
        self.insts.push(Inst::Const(reg, value));
        return reg;
    }

    fn add(&mut self, ast: &Ast, scope: &mut Scope) -> usize {
        match ast {
            Ast::I32(num) => self.add_consts(Value::I32(*num)),
            Ast::F64(num) => self.add_consts(Value::F64(*num)),
            Ast::Bool(val) => self.add_consts(Value::Bool(*val)),
            Ast::Add(a, b) => {
                let a = self.add(a, scope);
                let b = self.add(b, scope);
                self.add_op(Op::Add, a, b)
            }
            Ast::Sub(a, b) => {
                let a = self.add(a, scope);
                let b = self.add(b, scope);
                self.add_op(Op::Sub, a, b)
            }
            Ast::Mul(a, b) => {
                let a = self.add(a, scope);
                let b = self.add(b, scope);
                self.add_op(Op::Mul, a, b)
            }
            Ast::Div(a, b) => {
                let a = self.add(a, scope);
                let b = self.add(b, scope);
                self.add_op(Op::Div, a, b)
            }
            Ast::Eq(a, b) => {
                let a = self.add(a, scope);
                let b = self.add(b, scope);
                self.add_op(Op::Eq, a, b)
            }
            Ast::Ne(a, b) => {
                let a = self.add(a, scope);
                let b = self.add(b, scope);
                self.add_op(Op::Ne, a, b)
            }
            Ast::Lt(a, b) => {
                let a = self.add(a, scope);
                let b = self.add(b, scope);
                self.add_op(Op::Lt, a, b)
            }
            Ast::Le(a, b) => {
                let a = self.add(a, scope);
                let b = self.add(b, scope);
                self.add_op(Op::Le, a, b)
            }
            Ast::Gt(a, b) => {
                let a = self.add(a, scope);
                let b = self.add(b, scope);
                self.add_op(Op::Gt, a, b)
            }
            Ast::Ge(a, b) => {
                let a = self.add(a, scope);
                let b = self.add(b, scope);
                self.add_op(Op::Ge, a, b)
            }
            Ast::Negative(val) => {
                let val = self.add(val, scope);
                self.add_uop(UOp::Neg, val)
            }
            Ast::If(cond, a, b) => {
                let then_block = self.new_block();
                let else_block = self.new_block();
                let out_block = self.new_block();

                // if
                let cond = self.add(cond, scope);
                self.insts
                    .push(Inst::Branch(cond, (then_block, else_block)));

                // then
                self.add_label(then_block);
                let a = self.add(a, scope);
                self.insts.push(Inst::JumpTo(out_block, a));

                // else
                self.add_label(else_block);
                let b = self.add(b, scope);
                self.insts.push(Inst::JumpTo(out_block, b));

                // continue
                return self.add_label(out_block);
            }
            Ast::Ident(name) => scope.get(name).unwrap_or(usize::MAX),
            Ast::FuncCall(func, args) => {
                let func = self.add(func, scope);
                let arg_regs = args.iter().map(|arg| self.add(arg, scope)).collect();
                let var = self.new_var();
                self.insts.push(Inst::Call(var, func, arg_regs));
                var
            }
            Ast::Block(nodes) => {
                let scope = &mut scope.child();
                for node in nodes {
                    self.add(node, scope);
                }
                0
            }
            Ast::Declair(name, node) => {
                let var = self.add(&node, scope);
                scope.set(name.clone(), var);
                var
            }
            Ast::Assign(name, node) => {
                let var = self.add(&node, scope);
                // scope.update(name, var);
                var
            }
            Ast::While(cond, block) => {
                // let label = self.next_block();
                // let cond = self.add(&cond, scope);
                // let cond = self.add_op(Inst::Not(cond));
                // let branch = self.add_placeholder();
                // self.add(&block, scope);
                // self.blocks.push(BlockData::JumpTo(label));
                // self.fill_placeholder(branch, BlockData::Branch(cond, self.next_block()));
                0
            }
            Ast::Return(node) => {
                let reg = self.add(&node, scope);
                self.insts.push(Inst::Return(reg));
                0
            }
            Ast::Error => self.add_consts(Value::Err),
            Ast::Array(_values) => 0,
        }
    }

    pub fn log(&self, args: &Vec<Value>) {
        println!("==========>");
        println!("'start{args:?}");
        for (i, inst) in self.insts.iter().enumerate() {
            for block in 0..self.blocks.len() {
                if self.blocks[block] == i {
                    println!("'{} (v{}):", block, self.block_params[block]);
                }
            }

            match inst {
                Inst::Branch(cond, (a, b)) => println!("  if v{cond} then '{a} else '{b}"),
                Inst::Const(var, val) => println!("  v{var} = {val:?}"),
                Inst::Op(var, op, a, b) => println!("  v{var} = ({op:?} v{a} v{b})"),
                Inst::UOp(var, op, a) => println!("  v{var} = ({op:?} v{a})"),
                Inst::Return(var) => println!("  return v{var}"),
                Inst::JumpTo(block, arg) => println!("  '{block}(v{arg})"),
                Inst::Call(var, func_id, args) => println!("  v{var} = ${func_id}{args:?}"),
                _ => println!("  {:?}", inst),
            }
        }
    }
}

pub fn exec_ir(func: &Func, funcs: &Vec<Func>, mem: &mut Vec<Value>, args: Vec<Value>) -> Value {
    let mut step = 0;
    let mut regs: Vec<Value> = vec![Value::Unit; func.ir.num_vars];

    for (i, arg) in args.into_iter().enumerate() {
        regs[i] = arg;
    }

    loop {
        step += 1;
        match &func.ir.insts[step - 1] {
            Inst::Op(var, op, a, b) => {
                regs[*var] = match op {
                    Op::Add => regs[*a].add(regs[*b]),
                    Op::Sub => regs[*a].sub(regs[*b]),
                    Op::Mul => regs[*a].mul(regs[*b]),
                    Op::Div => regs[*a].div(regs[*b]),
                    Op::Eq => regs[*a].eq(regs[*b]),
                    Op::Ne => regs[*a].ne(regs[*b]),
                    Op::Le => regs[*a].le(regs[*b]),
                    Op::Lt => regs[*a].lt(regs[*b]),
                    Op::Ge => regs[*a].ge(regs[*b]),
                    Op::Gt => regs[*a].gt(regs[*b]),
                };
            }
            Inst::UOp(var, op, a) => {
                regs[*var] = match op {
                    UOp::Neg => regs[*a].neg(),
                    UOp::Not => regs[*a].not(),
                };
            }
            Inst::Const(var, val) => {
                regs[*var] = *val;
            }
            Inst::Call(var, func_id_reg, param_regs) => {
                let args = param_regs.iter().map(|reg| regs[*reg]).collect();
                regs[*var] = exec_ir(&funcs[*func_id_reg], funcs, mem, args);
            }
            Inst::JumpTo(block, var) => {
                step = func.ir.blocks[*block];
                let arg = func.ir.block_params[*block];
                regs[arg] = regs[*var];
            }
            Inst::Branch(cond, (a, b)) => {
                if regs[*cond].as_bool() {
                    step = func.ir.blocks[*a];
                } else {
                    step = func.ir.blocks[*b];
                }
            }
            Inst::Return(var) => {
                return regs[*var];
            }
        }
    }
}
