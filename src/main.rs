mod core;
// mod server;
// mod targets;
mod utils;

use crate::core::*;

fn main() {
    let module = Module::from_src(
        r"
            main() {
                let x = 4242
                let y = 8008
                return x
            }
        ",
    );

    // module.log()

    module.exec("main", vec![]);
}
