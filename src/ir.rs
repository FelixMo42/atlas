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
    pub params: Vec<String>,
    pub body: Vec<BlockData>,
}

#[derive(Debug, Clone, Copy)]
pub enum Inst {
    // math ops
    Add(Reg, Reg),
    Sub(Reg, Reg),
    Mul(Reg, Reg),
    Div(Reg, Reg),
    Neg(Reg),

    // boolean ops
    Eq(Reg, Reg),

    // misc
    Move(Reg),
    Call(FuncId, Reg),
}

#[derive(Debug)]
pub enum BlockData {
    Assign(Reg, Inst),
    Branch(Reg, Block),
    Consts(Reg, Value),
    JumpTo(Block),
    Return(Reg),
}

pub fn exec_ir(func: &Func, funcs: &Vec<Func>, args: Value) -> Value {
    let mut step = 0;
    let mut regs: Vec<Value> = vec![Value::Unit; func.body.len()];

    regs[0] = args;

    loop {
        match &func.body[step] {
            BlockData::Assign(value, inst) => {
                regs[*value] = match inst {
                    Inst::Add(a, b) => regs[*a].add(regs[*b].clone()),
                    Inst::Sub(a, b) => regs[*a].sub(regs[*b].clone()),
                    Inst::Mul(a, b) => regs[*a].mul(regs[*b].clone()),
                    Inst::Div(a, b) => regs[*a].div(regs[*b].clone()),
                    Inst::Eq(a, b) => regs[*a].eq(regs[*b].clone()),
                    Inst::Neg(a) => regs[*a].neg(),
                    Inst::Move(a) => regs[*a].clone(),
                    Inst::Call(func_id_reg, param_reg) => {
                        exec_ir(&funcs[*func_id_reg], funcs, regs[*param_reg].clone())
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

pub fn func_to_ir(ast: &Ast, scope: &Scope) -> Vec<BlockData> {
    let mut builder = IrBuilder {
        blocks: vec![BlockData::JumpTo(1)],
        map: scope.child(),
    };

    builder.add(ast);

    return builder.blocks;
}

pub fn expr_to_ir(ast: Ast) -> Vec<BlockData> {
    let mut builder = IrBuilder {
        blocks: vec![],
        map: Scope::default(),
    };

    let reg = builder.add(&ast);
    builder.blocks.push(BlockData::Return(reg));

    return builder.blocks;
}

struct IrBuilder<'a> {
    blocks: Vec<BlockData>,
    map: Scope<'a>,
}

impl<'a> IrBuilder<'a> {
    fn add_inst(&mut self, inst: Inst) -> usize {
        let block = self.next_block();
        self.blocks.push(BlockData::Assign(block, inst));
        return block;
    }

    fn add_consts(&mut self, value: Value) -> usize {
        let block = self.next_block();
        self.blocks.push(BlockData::Consts(block, value));
        return block;
    }

    fn add_placeholder(&mut self) -> usize {
        let location = self.blocks.len();
        self.blocks.push(BlockData::JumpTo(usize::MAX));
        return location;
    }

    fn fill_placeholder(&mut self, placeholder: usize, block: BlockData) {
        self.blocks[placeholder] = block;
    }

    fn next_block(&self) -> usize {
        return self.blocks.len();
    }

    fn add(&mut self, ast: &Ast) -> usize {
        match ast {
            Ast::I32(num) => self.add_consts(Value::I32(*num)),
            Ast::F64(num) => self.add_consts(Value::F64(*num)),
            Ast::Bool(val) => self.add_consts(Value::Bool(*val)),
            Ast::Add(a, b) => {
                let a = self.add(a);
                let b = self.add(b);
                self.add_inst(Inst::Add(a, b))
            }
            Ast::Sub(a, b) => {
                let a = self.add(a);
                let b = self.add(b);
                self.add_inst(Inst::Sub(a, b))
            }
            Ast::Mul(a, b) => {
                let a = self.add(a);
                let b = self.add(b);
                self.add_inst(Inst::Mul(a, b))
            }
            Ast::Div(a, b) => {
                let a = self.add(a);
                let b = self.add(b);
                self.add_inst(Inst::Div(a, b))
            }
            Ast::Eq(a, b) => {
                let a = self.add(a);
                let b = self.add(b);
                self.add_inst(Inst::Eq(a, b))
            }
            Ast::Negative(val) => {
                let val = self.add(val);
                self.add_inst(Inst::Neg(val))
            }
            Ast::If(cond, a, b) => {
                let cond = self.add(cond);

                let branch = self.add_placeholder();

                let b = self.add(b);
                let jump = self.add_placeholder();

                let branch_target = self.next_block();
                let a = self.add(a);
                self.blocks.push(BlockData::Assign(b, Inst::Move(a)));

                self.fill_placeholder(branch, BlockData::Branch(cond, branch_target));
                self.fill_placeholder(jump, BlockData::JumpTo(self.next_block()));

                return b;
            }
            Ast::Ident(name) => self.map.get(name).unwrap_or(0),
            Ast::FuncCall(func, args) => {
                let func = self.add(func);
                let args = if args.len() > 0 {
                    self.add(&args[0])
                } else {
                    0
                };
                self.add_inst(Inst::Call(func, args))
            }
            Ast::Block(nodes) => {
                for node in nodes {
                    self.add(node);
                }
                0
            }
            Ast::Assign(name, node) => {
                let reg = self.add(&node);
                self.map.set(name.clone(), reg);
                0
            }
            Ast::Return(node) => {
                let reg = self.add(&node);
                self.blocks.push(BlockData::Return(reg));
                0
            }
            Ast::Error => self.add_consts(Value::Err),
        }
    }
}
