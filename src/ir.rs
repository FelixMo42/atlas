use crate::module::*;
use crate::parser::*;
use crate::value::*;

type Reg = usize;
type FuncId = usize;
type Block = usize;

#[derive(Debug)]
pub struct Func {
    pub name: String,
    pub num_vars: usize,
    pub return_type: Type,
    pub body: Vec<BlockData>,
}

impl Func {
    pub fn new(module: &Module, func_def: FuncDef) -> Func {
        let mut builder = IrBuilder {
            blocks: vec![],
            num_vars: func_def.params.len(),
        };

        let scope = &mut module.scope.child();

        for (i, param) in func_def.params.into_iter().enumerate() {
            scope.set(param, i);
        }

        builder.add(&func_def.body, scope);

        builder.blocks.push(BlockData::Return(0));

        return Func {
            name: func_def.name,
            body: builder.blocks,
            return_type: func_def.return_type,
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

    // memmory
    Alloc(Reg),
    Store(Reg, Reg),
    Load(Reg),
}

#[derive(Debug)]
pub enum BlockData {
    Assign(Reg, Inst),
    Branch(Reg, Block),
    Consts(Reg, Value),
    JumpTo(Block),
    Return(Reg),
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
            Ast::Ident(name) => scope.get(name).unwrap_or(usize::MAX),
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
            Ast::Array(_values) => 0,
        }
    }
}

pub fn exec_ir(func: &Func, funcs: &Vec<Func>, mem: &mut Vec<Value>, args: Vec<Value>) -> Value {
    let mut step = 0;
    let mut regs: Vec<Value> = vec![Value::Unit; func.num_vars];

    for (i, arg) in args.into_iter().enumerate() {
        regs[i] = arg;
    }

    loop {
        match &func.body[step] {
            BlockData::Assign(value, inst) => {
                regs[*value] = match inst {
                    Inst::Store(address, value) => {
                        mem[*address] = regs[*value];
                        return regs[*value];
                    }
                    Inst::Load(address) => mem[*address],
                    Inst::Add(a, b) => regs[*a].add(regs[*b]),
                    Inst::Sub(a, b) => regs[*a].sub(regs[*b]),
                    Inst::Mul(a, b) => regs[*a].mul(regs[*b]),
                    Inst::Div(a, b) => regs[*a].div(regs[*b]),
                    Inst::Eq(a, b) => regs[*a].eq(regs[*b]),
                    Inst::Ne(a, b) => regs[*a].ne(regs[*b]),
                    Inst::Lt(a, b) => regs[*a].lt(regs[*b]),
                    Inst::Le(a, b) => regs[*a].le(regs[*b]),
                    Inst::Gt(a, b) => regs[*a].gt(regs[*b]),
                    Inst::Ge(a, b) => regs[*a].ge(regs[*b]),
                    Inst::Not(a) => regs[*a].not(),
                    Inst::Neg(a) => regs[*a].neg(),
                    Inst::Move(a) => regs[*a],
                    Inst::Call(func_id_reg, param_regs) => {
                        let args = param_regs.iter().map(|reg| regs[*reg]).collect();
                        exec_ir(&funcs[*func_id_reg], funcs, mem, args)
                    }
                    Inst::Alloc(a) => {
                        let start = mem.len();
                        mem.extend((0..regs[*a].as_i32()).map(|_| Value::Unit));
                        return Value::I32(start as i32);
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
                regs[*reg] = *value;
                step += 1;
            }
            BlockData::JumpTo(block) => step = *block,
            BlockData::Return(value) => {
                return regs[*value];
            }
        }
    }
}
