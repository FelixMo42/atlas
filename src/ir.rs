use crate::module::*;
use crate::parser::*;
use crate::value::*;

type Var = usize;
type FuncId = usize;
type Block = usize;

const NO_VALUE: Var = usize::MAX;

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
            scope.declair(param.name, i);
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

    pub fn log(&self) {
        println!("function {} ():", self.name);
        self.ir.log();
        println!();
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
    JumpTo(Block, Vec<Var>),
    Return(Var),
}

#[derive(Debug)]
pub struct Blocks {
    pub insts: Vec<Inst>,

    pub num_vars: usize,
    pub var_decl: Vec<usize>,
    pub var_type: Vec<Type>,

    pub blocks: Vec<usize>,
    pub block_params: Vec<(usize, usize)>,
}

impl Blocks {
    fn new(params: &Vec<Param>) -> Self {
        return Blocks {
            insts: vec![],

            num_vars: params.len(),
            var_decl: vec![],
            var_type: vec![],

            blocks: vec![0],
            block_params: vec![(0, 0)],
        };
    }

    fn new_var(&mut self, t: Type) -> Var {
        self.var_type.push(t);
        self.var_decl.push(self.insts.len() + 1);
        self.num_vars += 1;
        return self.num_vars - 1;
    }

    fn new_block(&mut self) -> Block {
        self.blocks.push(0);
        self.block_params.push((0, 0));
        return self.blocks.len() - 1;
    }

    fn add_label(&mut self, block: Block) {
        self.blocks[block] = self.insts.len();
    }

    fn add_op(&mut self, op: Op, a: Var, b: Var) -> usize {
        let var = self.new_var(match op {
            Op::Add | Op::Div | Op::Sub | Op::Mul => self.var_type[a],
            Op::Eq | Op::Ne | Op::Ge | Op::Gt | Op::Le | Op::Lt => Type::Bool,
        });
        self.insts.push(Inst::Op(var, op, a, b));
        return var;
    }

    fn add_uop(&mut self, op: UOp, a: Var) -> usize {
        let var = self.new_var(self.var_type[a]);
        self.insts.push(Inst::UOp(var, op, a));
        return var;
    }

    fn add_consts(&mut self, value: Value) -> usize {
        let reg = self.new_var(value.get_type());
        self.insts.push(Inst::Const(reg, value));
        return reg;
    }

    fn add_jump(&mut self, block: Block) -> usize {
        self.insts.push(Inst::JumpTo(block, vec![]));
        return self.insts.len() - 1;
    }

    fn add_arg_to_jump(&mut self, isnt: usize, arg: usize) {
        if let Inst::JumpTo(_, args) = &mut self.insts[isnt] {
            args.push(arg);
        };
    }

    fn add_param_to_block(&mut self, block: Block, t: Type) -> Var {
        let var = self.new_var(t);
        self.block_params[block].1 += 1;
        return var;
    }

    fn update(&mut self, block: Block, old: Var, new: Var) {
        for i in self.blocks[block]..self.insts.len() {
            match &self.insts[i] {
                Inst::Branch(cond, paths) => {
                    if *cond == old {
                        self.insts[i] = Inst::Branch(new, paths.clone())
                    }
                }
                Inst::Call(var, func, args) => {
                    if args.contains(&old) {
                        self.insts[i] = Inst::Call(
                            *var,
                            *func,
                            args.iter()
                                .map(|arg| if *arg == old { new } else { *arg })
                                .collect(),
                        )
                    }
                }
                Inst::JumpTo(block, args) => {
                    self.insts[i] = Inst::JumpTo(
                        *block,
                        args.iter()
                            .map(|arg| if *arg == old { new } else { *arg })
                            .collect(),
                    )
                }
                Inst::Return(var) => {
                    if *var == old {
                        self.insts[i] = Inst::Return(new)
                    }
                }
                Inst::Op(var, op, a, b) => {
                    if *a == old {
                        self.insts[i] = Inst::Op(*var, op.clone(), new, *b)
                    } else if *b == old {
                        self.insts[i] = Inst::Op(*var, op.clone(), *a, new)
                    }
                }
                Inst::UOp(var, op, a) => {
                    if *a == old {
                        self.insts[i] = Inst::UOp(*var, op.clone(), new)
                    }
                }
                _ => {}
            }
        }
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

                let (mut a_scope, mut b_scope) = scope.branch();

                // then
                self.add_label(then_block);
                let a_ret = self.add(a, &mut a_scope);
                let a_jump = self.add_jump(out_block);

                // else
                self.add_label(else_block);
                let b_ret = self.add(b, &mut b_scope);
                let b_jump = self.add_jump(out_block);

                // continue
                self.add_label(out_block);

                let a_vars = a_scope.assign;
                let b_vars = b_scope.assign;

                // phi nodes
                self.block_params[out_block].0 = self.num_vars;

                for key in a_vars.keys() {
                    self.add_arg_to_jump(a_jump, *a_vars.get(key).unwrap());

                    if !b_vars.contains_key(key) {
                        self.add_arg_to_jump(b_jump, scope.get(key).unwrap());
                    }

                    let t = self.var_type[scope.get(key).unwrap()];
                    scope.assign(key.clone(), self.add_param_to_block(out_block, t));
                }

                for key in b_vars.keys() {
                    self.add_arg_to_jump(b_jump, *b_vars.get(key).unwrap());

                    if !a_vars.contains_key(key) {
                        self.add_arg_to_jump(a_jump, scope.get(key).unwrap());
                        let t = self.var_type[scope.get(key).unwrap()];
                        scope.assign(key.clone(), self.add_param_to_block(out_block, t));
                    }
                }

                if a_ret != NO_VALUE {
                    self.add_arg_to_jump(a_jump, a_ret);
                    self.add_arg_to_jump(b_jump, b_ret);

                    self.add_param_to_block(out_block, self.var_type[a_ret])
                } else {
                    NO_VALUE
                }
            }
            Ast::Ident(name) => scope.get(name).unwrap_or(usize::MAX),
            Ast::FuncCall(func, args) => {
                // let func = self.add(func, scope);
                // let arg_regs = args.iter().map(|arg| self.add(arg, scope)).collect();
                // let var = self.new_var();
                // self.insts.push(Inst::Call(var, func, arg_regs));
                // var
                0
            }
            Ast::Block(nodes) => {
                let mut child_scope = scope.child();
                for node in nodes {
                    self.add(node, &mut child_scope);
                }
                let child_vars = child_scope.assign;
                for name in child_vars.keys() {
                    scope.assign(name.clone(), *child_vars.get(name).unwrap());
                }
                NO_VALUE
            }
            Ast::Declair(name, node) => {
                let var = self.add(&node, scope);
                scope.declair(name.clone(), var);
                var
            }
            Ast::Assign(name, node) => {
                let var = self.add(&node, scope);
                scope.assign(name.clone(), var);
                var
            }
            Ast::While(cond, body) => {
                let cond_block = self.new_block();
                let body_block = self.new_block();
                let out_block = self.new_block();

                let entry_jump = self.add_jump(cond_block);

                // cond block insts
                self.add_label(cond_block);
                let cond = self.add(&cond, scope);
                self.insts.push(Inst::Branch(cond, (body_block, out_block)));

                // blody blocks insts
                let mut body_scope = scope.child();
                self.add_label(body_block);
                let r = self.add(body, &mut body_scope);
                let body_jump = self.add_jump(cond_block);
                let body_vars = body_scope.assign;

                // cond block params
                self.block_params[cond_block].0 = self.num_vars;
                for name in body_vars.keys() {
                    let old = scope.get(name).unwrap();
                    let arg = *body_vars.get(name).unwrap();
                    let new = self.add_param_to_block(cond_block, self.var_type[old]);

                    self.add_arg_to_jump(entry_jump, old);
                    self.add_arg_to_jump(body_jump, arg);

                    scope.assign(name.clone(), new);

                    self.update(cond_block, old, new);
                }

                self.add_label(out_block);

                r
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

    pub fn log(&self) {
        for (i, inst) in self.insts.iter().enumerate() {
            for block in 0..self.blocks.len() {
                if self.blocks[block] == i {
                    let (first_param, num_params) = self.block_params[block];
                    println!(
                        "'{} ({}):",
                        block,
                        (0..num_params)
                            .map(|i| format!("v{}", first_param + i))
                            .collect::<Vec<String>>()
                            .join("\n")
                    );
                }
            }

            match inst {
                Inst::Branch(cond, (a, b)) => println!("  if v{cond} then '{a} else '{b}"),
                Inst::Const(var, val) => println!("  v{var} = {val:?}"),
                Inst::Op(var, op, a, b) => println!("  v{var} = ({op:?} v{a} v{b})"),
                Inst::UOp(var, op, a) => println!("  v{var} = ({op:?} v{a})"),
                Inst::Return(var) => println!("  return v{var}"),
                Inst::Call(var, func_id, args) => println!("  v{var} = ${func_id}{args:?}"),
                Inst::JumpTo(block, args) => {
                    println!(
                        "  '{block}({})",
                        args.iter()
                            .map(|arg| format!("v{}", arg))
                            .collect::<Vec<String>>()
                            .join("\n")
                    );
                }
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
            Inst::JumpTo(block, args) => {
                step = func.ir.blocks[*block];

                let (first_param, num_params) = func.ir.block_params[*block];

                for i in 0..num_params {
                    regs[first_param + i] = regs[args[i]];
                }
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
