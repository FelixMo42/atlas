use crate::ir::*;
use crate::parser::*;
use crate::value::*;

use std::collections::HashMap;

#[derive(Default)]
pub struct Module<'a> {
    pub scope: Scope<'a>,
    pub funcs: Vec<Func>,
}

impl<'a> Module<'a> {
    pub fn from_src(src: &str) -> Self {
        let mut module = Module::default();

        // parse all the functions
        let funcs = parse(src);

        // register the functions in the scope
        for i in 0..funcs.len() {
            module
                .scope
                .set(funcs[i].name.clone(), module.funcs.len() + i);
        }

        // turn the functions in to ir
        for func_def in funcs {
            module.funcs.push(Func::new(&module, func_def));
        }

        return module;
    }

    pub fn exec(&self, name: &str, args: Vec<Value>) -> Value {
        if let Some(func) = self.get(name) {
            let memory = &mut vec![];
            return exec_ir(func, &self.funcs, memory, args);
        } else {
            return Value::Err;
        }
    }

    pub fn get(&self, name: &str) -> Option<&Func> {
        self.scope.get(name).map(|func_id| &self.funcs[func_id])
    }
}

impl<'a> Module<'a> {
    fn add_func(mut self, func: Func) -> Self {
        self.scope.set(func.name.clone(), self.funcs.len());
        self.funcs.push(func);
        return self;
    }

    fn add_std(self) -> Self {
        return self
            .add_func(Func {
                name: "alloc".to_string(),
                num_vars: 1,
                return_type: Type::I32,
                body: vec![BlockData::Assign(0, Inst::Alloc(0)), BlockData::Return(0)],
            })
            .add_func(Func {
                name: "store".to_string(),
                num_vars: 2,
                return_type: Type::I32,
                body: vec![
                    BlockData::Assign(0, Inst::Store(0, 1)),
                    BlockData::Return(0),
                ],
            })
            .add_func(Func {
                name: "load".to_string(),
                num_vars: 1,
                return_type: Type::I32,
                body: vec![BlockData::Assign(0, Inst::Load(0)), BlockData::Return(0)],
            });
    }
}

#[derive(Default)]
pub struct Scope<'a> {
    vars: HashMap<String, usize>,
    parent: Option<&'a Scope<'a>>,
}

impl<'a> Scope<'a> {
    pub fn get(&self, name: &str) -> Option<usize> {
        if let Some(value) = self.vars.get(name) {
            return Some(*value);
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
