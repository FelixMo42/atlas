use crate::ir::*;

use std::collections::HashSet;

/// Get the list of immidate children of <block>.
pub fn get_children(func: &Func, block: Block) -> Vec<usize> {
    match get_exit_inst(func, block) {
        Inst::Return(..) => vec![],
        Inst::Branch(_, (a, b)) => vec![a, b],
        Inst::JumpTo(target, _) => vec![target],
        _ => unreachable!(),
    }
}

/// Does <a> lead into <b>?
pub fn is_parent_of(func: &Func, a: Block, b: Block) -> bool {
    let mut todo = vec![a];
    let mut seen = HashSet::new();

    while let Some(block) = todo.pop() {
        for child in get_children(func, block) {
            if child == b {
                return true;
            } else if !seen.contains(&b) {
                seen.insert(b);
                todo.push(b);
            }
        }
    }

    return false;
}

/// Do all path to <b> go throght <a>?
pub fn dominates(_f: &Func, _a: Block, _b: Block) -> bool {
    return true; // TODO: !!!
}

/// Does a child of <block> point towards <block>?
pub fn is_loop(f: &Func, block: Block) -> bool {
    for inst in &f.ir.insts[f.ir.blocks[block]..] {
        match inst {
            Inst::Branch(_, (a, b)) => {
                if *a == block || *b == block {
                    return true;
                }
            }
            Inst::JumpTo(target, _) => {
                if *target == block {
                    return true;
                }
            }
            _ => {}
        }
    }

    return false;
}

/// Get the final instruction of a block.
pub fn get_exit_inst(func: &Func, block: Block) -> Inst {
    for inst in &func.ir.insts[func.ir.blocks[block]..] {
        match inst {
            Inst::Return(..) | Inst::Branch(..) | Inst::JumpTo(..) => return inst.clone(),
            _ => {}
        };
    }

    panic!("Block didn't end!")
}
