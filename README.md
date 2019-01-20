# JazzML
ML-like language used for scripting


# Examples

## Data

```ocaml
let TokenType = enum {
  String
  Number
  Identifier
  Symbol
  EOF
}

let Token = struct {
  position: (int, str) # line number and actual line
  slice:    (int, int) # beginning and end of lexeme on line

  type:   TokenType
  lexeme: str          # content of token
}
```

## Other

```ocaml
let fibonacci = func(a: int) : int {
  # implicit returns
  if a > 2 {
    fib(a - 1) + fib(a - 2)
  } else {
    a
  }
}
```


# Known issues
- Bad perfomance
  Because of using Rc<RefCell<Value>> for ValueRef VM perfomance is slow


# Unimplemented
- Tail calls

# TODO
- x86_64 JIT Compiler
