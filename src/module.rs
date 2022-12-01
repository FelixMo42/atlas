use crate::ir::*;
use crate::lexer::*;
use crate::parser::*;
use crate::value::*;

#[derive(Default)]
pub struct Module<'a> {
    pub scope: Scope<'a>,
    pub funcs: Vec<Func>,
}

impl<'a> Module<'a> {
    pub fn from_src(src: &str) -> Self {
        // parse all the functions
        let lex = &mut Lexer::new(src);
        let mut funcs_ast = vec![];
        while let Some(func) = parse_func_def(lex) {
            funcs_ast.push(func)
        }

        let mut module = Module::default();

        // load the standard library
        module.add_std();

        // register the functions in the scope
        for i in 0..funcs_ast.len() {
            module
                .scope
                .set(funcs_ast[i].0.clone(), module.funcs.len() + i);
        }

        // turn the functions in to ir
        for (name, params, ast) in funcs_ast {
            module
                .funcs
                .push(Func::new(name, params, &ast, &module.scope));
        }

        return module;
    }

    pub fn exec(&self, name: &str, args: Vec<Value>) -> Value {
        if let Some(func_id) = self.scope.get(name) {
            let memory = &mut vec![];
            return exec_ir(&self.funcs[func_id], &self.funcs, memory, args);
        } else {
            return Value::Err;
        }
    }
}

impl<'a> Module<'a> {
    fn add(&mut self, func: Func) {
        self.scope.set(func.name.clone(), self.funcs.len());
        self.funcs.push(func);
    }

    fn add_std(&mut self) {
        self.add(Func {
            name: "alloc".to_string(),
            num_vars: 1,
            body: vec![BlockData::Assign(0, Inst::Alloc(0)), BlockData::Return(0)],
        });

        self.add(Func {
            name: "store".to_string(),
            num_vars: 2,
            body: vec![
                BlockData::Assign(0, Inst::Store(0, 1)),
                BlockData::Return(0),
            ],
        });

        self.add(Func {
            name: "load".to_string(),
            num_vars: 1,
            body: vec![BlockData::Assign(0, Inst::Load(0)), BlockData::Return(0)],
        });
    }
}
