use crate::ir::*;

pub fn std(scope: &mut Scope, funcs: &mut Vec<Func>) {
    fn add(scope: &mut Scope, funcs: &mut Vec<Func>, name: &str, func: Func) {
        scope.set(name.to_string(), funcs.len());
        funcs.push(func);
    }

    add(
        scope,
        funcs,
        "alloc",
        Func {
            num_vars: 1,
            body: vec![
                BlockData::Assign(0, Inst::Alloc(0)),
                BlockData::Return(0),
            ],
        },
    );

    add(
        scope,
        funcs,
        "store",
        Func {
            num_vars: 2,
            body: vec![
                BlockData::Assign(0, Inst::Store(0, 1)),
                BlockData::Return(0),
            ],
        },
    );

    add(
        scope,
        funcs,
        "load",
        Func {
            num_vars: 1,
            body: vec![BlockData::Assign(0, Inst::Load(0)), BlockData::Return(0)],
        },
    );
}
