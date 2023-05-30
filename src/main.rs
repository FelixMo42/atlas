mod core;
mod server;
mod targets;
mod utils;

use crate::core::*;

fn main() {
    let module = Module::from_src(
        r"
            main(): I32 {
                let x = 4242
                let y = 8008
                return x + y
            }
        ",
    );

    println!("{:?}", module.exec("main", vec![]).as_i32());
}
