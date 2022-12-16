# Atlas

Yet another toy langauge to play with. This time its to learn about SSA/basic
blocks and give strict TDD a try.

## Learn x in y minutes (where x = Atlas)

```
// Single line comments start with two slashes.
// There are no multiline comments yet.

// All code must be inside of functions.
fn part_1_basic_types_and_operators() {
    // There are two types:
    1234 // i32
    1.2  // f64

    // The basic operators work.
    1 + 1     // = 2
    1.3 - .2  // = 1.1
    5 * 4     // = 20
    20 / 5    // = 4
    
    // Int division returns and int, flot division returns a float
    3 / 2     // = 1
    3.0 / 2.0 // = 1.5
    
    // Mixing types is not supported, it will return an error, but not crash the program
    3.0 / 2   // = Err

    // Order of operation can be implied.
    1 + 3 * 4 // = 13

    // Or made explicite using parentheses.
    (1 + 3) * 4 // = 16

    // There's also a boolean type.
    true;
    false;

    // Equality is ==
    1 == 1    // = true
    2 == 1    // = false
    2.0 == 1  // = Err

    // More comparisions
    1 != 1 // = false
    1 < 10 // = true
    1 > 10 // = false
    2 <= 2 // = true
    2 >= 2 // = true
}

// Any number of paramaters can be passed into a function.
fn part_2_variables_and_control_structures(a, b) {
    // Variables are declared with the `let` keyword.
    let some_name = 0

    // The same name can be reused even in the same scope.
    let some_name = 3

    // Or the variable can re assigned.
    some_name = 4

    // If statments can be return a value
    if (some_name == 4) 42 else 9 // = 42

    // Or not
    if some_name == 4 { // The parantases around the condition are optional
        let some_name = 134
    } else {
        let some_name = 13
    }
    // Some name is still 4 

    // They can also have else if blocks
    let fib = if (a == 0) 0
            else if (a == 1) 1
            else (
                part_2_variables_and_control_structures(a - 1, b)
                + part_2_variables_and_control_structures(a - 2, b)
            )

    // While loops:
    while some_name != 0 {
        some_name = some_name - 1
    }

    // A function can only return one value at the moment
    return fib
}

// The main functions is called at the start of the programe
fn main() {
    part_1_basic_types_and_operators()
    part_2_variables_and_control_structures(4, 5)
}
```

## Roadmap

- [x] step 1: make it turing complete
- [x] step 2: switch over to using register based ir and add basic language
      features
- [x] step 3: add linear memory (a la wasm)
- [x] step 4: types!
- [x] step 5: compile to wasm
- [x] step 6: dev server
- [ ] step 7: bootstraps baby

## How do I use it?

The only dependency is cargo.

Complile to wasm
`cargo run to-wasm <in> <out>`

Run the dev server
`cargo run server`

Run a file using ir
`cargo run <file>`

Run the unit test
`cargo test`
