use crate::parser::*;
use crate::value::*;

use std::collections::HashMap;

type Reg = usize;
type FuncId = usize;
type Block = usize;

#[derive(Default)]
pub struct Scope<'a> {
    vars: HashMap<String, usize>,
    parent: Option<&'a Scope<'a>>,
}

impl<'a> Scope<'a> {
    pub fn get(&self, name: &str) -> Option<usize> {
        if let Some(value) = self.vars.get(name) {
            return Some(value.clone());
        } else if let Some(parent) = self.parent {
            return parent.get(name);
        } else {
            return None;
        }
    }

    pub fn set(&mut self, name: String, value: usize) {
        self.vars.insert(name, value);
    }
}

impl<'a> Scope<'a> {
    pub fn child(&self) -> Scope {
        return Scope {
            vars: HashMap::new(),
            parent: Some(self),
        };
    }
}

#[derive(Debug)]
pub struct Func {
    body: Vec<BlockData>,
    num_vars: usize,
}

impl Func {
    pub fn new(params: Vec<String>, ast: &Ast, scope: &Scope) -> Func {
        let mut builder = IrBuilder {
            blocks: vec![BlockData::JumpTo(1)],
            num_vars: params.len(),
        };

        let scope = &mut scope.child();

        for (i, param) in params.into_iter().enumerate() {
            scope.set(param, i);
        }

        builder.add(ast, scope);

        builder.blocks.push(BlockData::Return(0));

        return Func {
            body: builder.blocks,
            num_vars: builder.num_vars,
        };
    }
}

#[derive(Debug, Clone)]
pub enum Inst {
    // math ops
    Add(Reg, Reg),
    Sub(Reg, Reg),
    Mul(Reg, Reg),
    Div(Reg, Reg),
    Neg(Reg),

    // logical ops
    Not(Reg),

    // boolean ops
    Eq(Reg, Reg),
    Ne(Reg, Reg),
    Le(Reg, Reg),
    Lt(Reg, Reg),
    Ge(Reg, Reg),
    Gt(Reg, Reg),

    // misc
    Move(Reg),
    Call(FuncId, Vec<Reg>),
}

#[derive(Debug)]
pub enum BlockData {
    Assign(Reg, Inst),
    Branch(Reg, Block),
    Consts(Reg, Value),
    JumpTo(Block),
    Return(Reg),
}

pub fn exec_ir(func: &Func, funcs: &Vec<Func>, args: Vec<Value>) -> Value {
    let mut step = 0;
    let mut regs: Vec<Value> = vec![Value::Unit; func.num_vars];

    for (i, arg) in args.into_iter().enumerate() {
        regs[i] = arg;
    }

    loop {
        match &func.body[step] {
            BlockData::Assign(value, inst) => {
                regs[*value] = match inst {
                    Inst::Add(a, b) => regs[*a].add(regs[*b].clone()),
                    Inst::Sub(a, b) => regs[*a].sub(regs[*b].clone()),
                    Inst::Mul(a, b) => regs[*a].mul(regs[*b].clone()),
                    Inst::Div(a, b) => regs[*a].div(regs[*b].clone()),
                    Inst::Eq(a, b) => regs[*a].eq(regs[*b].clone()),
                    Inst::Ne(a, b) => regs[*a].ne(regs[*b].clone()),
                    Inst::Lt(a, b) => regs[*a].lt(regs[*b].clone()),
                    Inst::Le(a, b) => regs[*a].le(regs[*b].clone()),
                    Inst::Gt(a, b) => regs[*a].gt(regs[*b].clone()),
                    Inst::Ge(a, b) => regs[*a].ge(regs[*b].clone()),
                    Inst::Not(a) => regs[*a].not(),
                    Inst::Neg(a) => regs[*a].neg(),
                    Inst::Move(a) => regs[*a].clone(),
                    Inst::Call(func_id_reg, param_regs) => {
                        let args = param_regs.iter().map(|reg| regs[*reg].clone()).collect();
                        exec_ir(&funcs[*func_id_reg], funcs, args)
                    }
                };
                step += 1;
            }
            BlockData::Branch(cond, block) => {
                if regs[*cond].as_bool() {
                    step = *block
                } else {
                    step += 1;
                }
            }
            BlockData::Consts(reg, value) => {
                regs[*reg] = value.clone();
                step += 1;
            }
            BlockData::JumpTo(block) => step = *block,
            BlockData::Return(value) => {
                return regs[*value].clone();
            }
        }
    }
}

struct IrBuilder {
    blocks: Vec<BlockData>,
    num_vars: usize,
}

impl IrBuilder {
    fn new_var(&mut self) -> Reg {
        self.num_vars += 1;
        return self.num_vars - 1;
    }

    fn add_inst(&mut self, inst: Inst) -> usize {
        let reg = self.new_var();
        self.blocks.push(BlockData::Assign(reg, inst));
        return reg;
    }

    fn add_consts(&mut self, value: Value) -> usize {
        let reg = self.new_var();
        self.blocks.push(BlockData::Consts(reg, value));
        return reg;
    }

    fn add_placeholder(&mut self) -> usize {
        let location = self.next_block();
        self.blocks.push(BlockData::JumpTo(usize::MAX));
        return location;
    }

    fn fill_placeholder(&mut self, placeholder: usize, block: BlockData) {
        self.blocks[placeholder] = block;
    }

    fn next_block(&self) -> usize {
        return self.blocks.len();
    }

    fn add(&mut self, ast: &Ast, scope: &mut Scope) -> usize {
        match ast {
            Ast::I32(num) => self.add_consts(Value::I32(*num)),
            Ast::F64(num) => self.add_consts(Value::F64(*num)),
            Ast::Bool(val) => self.add_consts(Value::Bool(*val)),
            Ast::Add(a, b) => {
                let a = self.add(a, scope);
                let b = self.add(b, scope);
                self.add_inst(Inst::Add(a, b))
            }
            Ast::Sub(a, b) => {
                let a = self.add(a, scope);
                let b = self.add(b, scope);
                self.add_inst(Inst::Sub(a, b))
            }
            Ast::Mul(a, b) => {
                let a = self.add(a, scope);
                let b = self.add(b, scope);
                self.add_inst(Inst::Mul(a, b))
            }
            Ast::Div(a, b) => {
                let a = self.add(a, scope);
                let b = self.add(b, scope);
                self.add_inst(Inst::Div(a, b))
            }
            Ast::Eq(a, b) => {
                let a = self.add(a, scope);
                let b = self.add(b, scope);
                self.add_inst(Inst::Eq(a, b))
            }
            Ast::Ne(a, b) => {
                let a = self.add(a, scope);
                let b = self.add(b, scope);
                self.add_inst(Inst::Ne(a, b))
            }
            Ast::Lt(a, b) => {
                let a = self.add(a, scope);
                let b = self.add(b, scope);
                self.add_inst(Inst::Lt(a, b))
            }
            Ast::Le(a, b) => {
                let a = self.add(a, scope);
                let b = self.add(b, scope);
                self.add_inst(Inst::Le(a, b))
            }
            Ast::Gt(a, b) => {
                let a = self.add(a, scope);
                let b = self.add(b, scope);
                self.add_inst(Inst::Gt(a, b))
            }
            Ast::Ge(a, b) => {
                let a = self.add(a, scope);
                let b = self.add(b, scope);
                self.add_inst(Inst::Ge(a, b))
            }
            Ast::Negative(val) => {
                let val = self.add(val, scope);
                self.add_inst(Inst::Neg(val))
            }
            Ast::If(cond, a, b) => {
                let cond = self.add(cond, scope);

                let branch = self.add_placeholder();

                let b = self.add(b, scope);
                let jump = self.add_placeholder();

                let branch_target = self.next_block();
                let a = self.add(a, scope);
                self.blocks.push(BlockData::Assign(b, Inst::Move(a)));

                self.fill_placeholder(branch, BlockData::Branch(cond, branch_target));
                self.fill_placeholder(jump, BlockData::JumpTo(self.next_block()));

                return b;
            }
            Ast::Ident(name) => scope.get(name).unwrap_or(0),
            Ast::FuncCall(func, args) => {
                let func = self.add(func, scope);
                let arg_regs = args.iter().map(|arg| self.add(arg, scope)).collect();
                self.add_inst(Inst::Call(func, arg_regs))
            }
            Ast::Block(nodes) => {
                let scope = &mut scope.child();
                for node in nodes {
                    self.add(node, scope);
                }
                0
            }
            Ast::Declair(name, node) => {
                let reg = self.add(&node, scope);
                scope.set(name.clone(), reg);
                reg
            }
            Ast::Assign(name, node) => {
                let reg = self.add(&node, scope);
                if let Some(var) = scope.get(name) {
                    self.blocks.push(BlockData::Assign(var, Inst::Move(reg)));
                }
                reg
            }
            Ast::While(cond, block) => {
                let label = self.next_block();
                let cond = self.add(&cond, scope);
                let cond = self.add_inst(Inst::Not(cond));
                let branch = self.add_placeholder();
                self.add(&block, scope);
                self.blocks.push(BlockData::JumpTo(label));
                self.fill_placeholder(branch, BlockData::Branch(cond, self.next_block()));
                0
            }
            Ast::Return(node) => {
                let reg = self.add(&node, scope);
                self.blocks.push(BlockData::Return(reg));
                0
            }
            Ast::Error => self.add_consts(Value::Err),
        }
    }
}
