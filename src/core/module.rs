use crate::core::*;
use crate::utils::*;

use std::collections::HashMap;

#[derive(Default)]
pub struct Module<'a> {
    pub scope: Scope<'a>,
    pub funcs: Vec<Func>,
}

impl<'a> Module<'a> {
    pub fn from_src(src: &str) -> Self {
        let mut module = Module::default();

        // parse the source
        let defs = parse(src);

        // get all the funcions
        let funcs = defs
            .iter()
            .filter_map(|ast| match ast {
                Ast::FuncDef(func_def) => Some(func_def),
                _ => None,
            })
            .collect::<Vec<&FuncDef>>();

        // register the functions in the scope
        for i in 0..funcs.len() {
            module
                .scope
                .declair(funcs[i].name.clone(), module.funcs.len() + i);
        }

        // turn the functions in to ir
        for func_def in funcs {
            module.funcs.push(Func::new(&module, func_def));
        }

        return module;
    }

    pub fn exec(&self, name: &str, args: Vec<Value>) -> Value {
        if let Some(func) = self.get(name) {
            let memory = &mut Mem::default();
            return exec_ir(func, &self.funcs, memory, args);
        } else {
            unimplemented!();
        }
    }

    pub fn get(&self, name: &str) -> Option<&Func> {
        self.scope.get(name).map(|func_id| &self.funcs[func_id])
    }

    pub fn log(&self, buffer: &mut impl std::io::Write) -> std::io::Result<()> {
        for func in &self.funcs {
            func.log(buffer)?;
        }
        return Ok(());
    }
}

#[derive(Default)]
pub struct Scope<'a> {
    pub assign: HashMap<String, usize>,
    pub locals: HashMap<String, usize>,
    parent: Option<&'a Scope<'a>>,
}

impl<'a> Scope<'a> {
    pub fn get(&self, name: &str) -> Option<usize> {
        if let Some(value) = self.locals.get(name) {
            return Some(*value);
        } else if let Some(value) = self.locals.get(name) {
            return Some(*value);
        } else if let Some(parent) = self.parent {
            return parent.get(name);
        } else {
            return None;
        }
    }

    pub fn declair(&mut self, name: String, value: usize) {
        self.locals.insert(name, value);
    }

    pub fn assign(&mut self, name: String, value: usize) {
        if self.locals.contains_key(&name) {
            self.locals.insert(name, value);
        } else {
            self.assign.insert(name, value);
        }
    }
}

impl<'a> Scope<'a> {
    pub fn child(&self) -> Scope {
        return Scope {
            assign: HashMap::new(),
            locals: HashMap::new(),
            parent: Some(self),
        };
    }

    pub fn branch(&self) -> (Scope, Scope) {
        return (self.child(), self.child());
    }
}
