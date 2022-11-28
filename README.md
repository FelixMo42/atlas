# Atlas

Yet another toy langauge to play with. This time its to learn about SSA/basic
blocks and give strict TDD a try.

## Learn x in y minutes (where x = Atlas)

This is hard too do since it dosen't have comments yet, but luckily its a very
simple language, so that shouldn't be too much of a problem.

```
fn fib(n) {
    return
        if (n == 0) 0
        else if (n == 1) 1
        else fib(n - 1) + fib(n - 2)
}

fn add(a, b) {
    let t = 0

    if a == 0 {
        return b
    }

    if (b == 0) {
        return a
    }

    t = t + a
    t = t + b

    return t
}

fn main() {
    if true {
        return fib(5)
    } else {
        return double(3.3, 6.5)
    }
}
```

## Roadmap

- [x] version 1: make it turing complete
- [ ] version 2: switch over to using register based ir and add basic language
      features
- [ ] version 3: add linear memory (a la wasm)

## How do I use it?

You want to use this? Why?

But, sorry. No can do. This entire language is built around satisfing my unit
test. There is no CLI. Not yet (duh duh dun)!
