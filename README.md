### Atlas

Yet another toy langauge to play with. This time its to learn about SSA/basic
blocks and give strict TDD a try.

# Learn x in y minutes (where x = Atlas)

This is hard too do since it dosen't have comments yet, but luckily its a very
simple language, so that shouldn't be too much of a problem.

```
fn fib(you_can_only_have_one_param) {
    let n = you_can_only_have_one_param
    return
        if (n == 0) 0
        else if (n == 1) 1
        else fib(n - 1) + fib(n - 2)
}

fn main() {
    let main_function_called_by_default = 5
    return fib(5)
}
```

# How do I use it?

You want to use this? Why?

But, sorry. No can do. This entire language is built around satisfing my unit
test. There is no CLI. Not yet (duh duh dun)!
