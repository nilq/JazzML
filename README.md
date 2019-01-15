# JazzML
ML-like language used for scripting


# Examples
```ocaml
enum type BinOp = {
    Add,
    Sub,
}

enum type Expr = {
    Int of Int
    Binary of Expr * BinOp * Expr
};

let function visit_expr e = {
    match e {
        Int: i -> return i,
        Binary: lhs, op, rhs -> {
            let lhs = visit_expr lhs
            let rhs = visit_expr rhs

            match op {
                Add -> return lhs + rhs
                Sub -> return lhs - rhs
            }
        }
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
