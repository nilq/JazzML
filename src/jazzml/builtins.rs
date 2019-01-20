use super::value::*;
use super::vm::VirtualMachine;

use std::f64;

pub fn value(value: Value) -> Value {
    value
}

pub fn add(vm: &mut VirtualMachine, args: Vec<Value>) -> Value {
    if args.len() == 0 || args.len() > 2 {
        return value(Value::Null);
    }
    let x: &Value = &args[0];
    let y: &Value = &args[1];
    match (x, y) {
        (Value::Float(f), v2) => {
            return value(Value::Float((f64::from_bits(*f) + v2.as_f64(vm)).to_bits()));
        }
        (Value::Int(i), Value::Float(f2)) => {
            return value(Value::Float((*i as f64 + f64::from_bits(*f2)).to_bits()));
        }
        (Value::Int(i), v2) => return value(Value::Int(*i + v2.as_int(vm))),
        (Value::Str(s), v) => {
            let str: &str = &v.as_str(vm);
            let mut buff = s.clone();
            buff.push_str(str);
            return value(Value::Str(buff));
        }
        (Value::Array(arr1), Value::Array(arr2)) => unimplemented!(),
        _ => panic!(""),
    }
}

pub fn sub(vm: &mut VirtualMachine, args: Vec<Value>) -> Value {
    if args.len() == 0 || args.len() > 2 {
        return value(Value::Null);
    }
    let x: &Value = &args[0];
    let y: &Value = &args[1];
    match (x, y) {
        (Value::Float(f), v2) => {
            return value(Value::Float((f64::from_bits(*f) - v2.as_f64(vm)).to_bits()));
        }
        (Value::Int(i), Value::Float(f2)) => {
            return value(Value::Float((*i as f64 - f64::from_bits(*f2)).to_bits()));
        }
        (Value::Int(i), v2) => return value(Value::Int(*i - v2.as_int(vm))),
        _ => panic!(""),
    }
}

pub fn mul(vm: &mut VirtualMachine, args: Vec<Value>) -> Value {
    if args.len() == 0 || args.len() > 2 {
        return value(Value::Null);
    }
    let x: &Value = &args[0];
    let y: &Value = &args[1];
    match (x, y) {
        (Value::Float(f), v2) => {
            return value(Value::Float((f64::from_bits(*f) * v2.as_f64(vm)).to_bits()));
        }
        (Value::Int(i), Value::Float(f2)) => {
            return value(Value::Float((*i as f64 * f64::from_bits(*f2)).to_bits()));
        }
        (Value::Int(i), v2) => return value(Value::Int(*i * v2.as_int(vm))),
        _ => panic!(""),
    }
}

pub fn div(vm: &mut VirtualMachine, args: Vec<Value>) -> Value {
    if args.len() == 0 || args.len() > 2 {
        return value(Value::Null);
    }
    let x: &Value = &args[0];
    let y: &Value = &args[1];
    match (x, y) {
        (Value::Float(f), v2) => {
            return value(Value::Float((f64::from_bits(*f) / v2.as_f64(vm)).to_bits()));
        }
        (Value::Int(i), Value::Float(f2)) => {
            return value(Value::Float((*i as f64 / f64::from_bits(*f2)).to_bits()));
        }
        (Value::Int(i), v2) => return value(Value::Int(*i / v2.as_int(vm))),
        _ => panic!(""),
    }
}

pub fn rem(vm: &mut VirtualMachine, args: Vec<Value>) -> Value {
    if args.len() == 0 || args.len() > 2 {
        return value(Value::Null);
    }
    let x: &Value = &args[0];
    let y: &Value = &args[1];
    match (x, y) {
        (Value::Float(f), v2) => {
            return value(Value::Float((f64::from_bits(*f) % v2.as_f64(vm)).to_bits()));
        }
        (Value::Int(i), Value::Float(f2)) => {
            return value(Value::Float((*i as f64 % f64::from_bits(*f2)).to_bits()));
        }
        (Value::Int(i), v2) => return value(Value::Int(*i % v2.as_int(vm))),
        _ => panic!(""),
    }
}

pub fn shr(_vm: &mut VirtualMachine, args: Vec<Value>) -> Value {
    if args.len() == 0 || args.len() > 2 {
        return value(Value::Null);
    }
    let x: &Value = &args[0];
    let y: &Value = &args[1];
    match (x, y) {
        (Value::Int(i1), Value::Int(i2)) => return value(Value::Int(i1 >> i2)),
        _ => return value(Value::Null),
    }
}

pub fn shl(_vm: &mut VirtualMachine, args: Vec<Value>) -> Value {
    if args.len() == 0 || args.len() > 2 {
        return value(Value::Null);
    }
    let x: &Value = &args[0];
    let y: &Value = &args[1];
    match (x, y) {
        (Value::Int(i1), Value::Int(i2)) => return value(Value::Int(i1 << i2)),
        _ => return value(Value::Null),
    }
}

pub fn band(_vm: &mut VirtualMachine, args: Vec<Value>) -> Value {
    if args.len() == 0 || args.len() > 2 {
        return value(Value::Null);
    }
    let x: &Value = &args[0];
    let y: &Value = &args[1];
    match (x, y) {
        (Value::Int(i1), Value::Int(i2)) => return value(Value::Int(i1 & i2)),
        _ => return value(Value::Null),
    }
}

pub fn bor(_vm: &mut VirtualMachine, args: Vec<Value>) -> Value {
    if args.len() == 0 || args.len() > 2 {
        return value(Value::Null);
    }
    let x: &Value = &args[0];
    let y: &Value = &args[1];
    match (x, y) {
        (Value::Int(i1), Value::Int(i2)) => return value(Value::Int(i1 | i2)),
        _ => return value(Value::Null),
    }
}
pub fn bxor(_vm: &mut VirtualMachine, args: Vec<Value>) -> Value {
    if args.len() == 0 || args.len() > 2 {
        return value(Value::Null);
    }
    let x: &Value = &args[0];
    let y: &Value = &args[1];
    match (x, y) {
        (Value::Int(i1), Value::Int(i2)) => return value(Value::Int(i1 | i2)),
        _ => return value(Value::Null),
    }
}

pub fn gt(vm: &mut VirtualMachine, args: Vec<Value>) -> Value {
    if args.len() == 0 || args.len() > 2 {
        return value(Value::Null);
    }
    let x: &Value = &args[0];
    let y: &Value = &args[1];
    match (x, y) {
        (Value::Float(f), v2) => {
            return value(Value::Bool(f64::from_bits(*f) > v2.as_f64(vm)));
        }
        (Value::Int(i), Value::Float(f2)) => {
            return value(Value::Bool(*i as f64 > f64::from_bits(*f2)));
        }
        (Value::Int(i), v2) => return value(Value::Bool(*i > v2.as_int(vm))),
        (Value::Array(a1), Value::Array(a2)) => return value(Value::Bool(a1.len() > a2.len())),
        (Value::Str(s1), Value::Str(s2)) => return value(Value::Bool(s1 < s2)),
        _ => panic!(""),
    }
}

pub fn lt(vm: &mut VirtualMachine, args: Vec<Value>) -> Value {
    if args.len() == 0 || args.len() > 2 {
        return value(Value::Null);
    }
    let x: &Value = &args[0];
    let y: &Value = &args[1];
    match (x, y) {
        (Value::Float(f), v2) => {
            return value(Value::Bool(f64::from_bits(*f) < v2.as_f64(vm)));
        }
        (Value::Int(i), Value::Float(f2)) => {
            return value(Value::Bool((*i as f64) < f64::from_bits(*f2)));
        }
        (Value::Int(i), v2) => return value(Value::Bool(*i < v2.as_int(vm))),
        (Value::Array(a1), Value::Array(a2)) => return value(Value::Bool(a1.len() < a2.len())),
        (Value::Str(s1), Value::Str(s2)) => return value(Value::Bool(s1 < s2)),
        _ => panic!(""),
    }
}

pub fn eq(vm: &mut VirtualMachine, args: Vec<Value>) -> Value {
    if args.len() == 0 || args.len() > 2 {
        return Value::Null;
    }
    let x: &Value = &args[0];
    let y: &Value = &args[1];
    match (x, y) {
        (Value::Float(f), v2) => {
            return Value::Bool(f64::from_bits(*f) == v2.as_f64(vm));
        }
        (Value::Int(i), Value::Float(f2)) => {
            return Value::Bool((*i as f64) == f64::from_bits(*f2));
        }
        (Value::Int(i), v2) => return value(Value::Bool(*i == v2.as_int(vm))),
        (Value::Array(a1), Value::Array(a2)) => return Value::Bool(a1 == a2),
        (Value::Str(s1), Value::Str(s2)) => return Value::Bool(s1 == s2),
        _ => panic!(""),
    }
}

pub fn neq(vm: &mut VirtualMachine, args: Vec<Value>) -> Value {
    if args.len() == 0 || args.len() > 2 {
        return Value::Null;
    }
    let x: &Value = &args[0];
    let y: &Value = &args[1];
    match (x, y) {
        (Value::Float(f), v2) => {
            return Value::Bool(f64::from_bits(*f) != v2.as_f64(vm));
        }
        (Value::Int(i), Value::Float(f2)) => {
            return Value::Bool((*i as f64) != f64::from_bits(*f2));
        }
        (Value::Int(i), v2) => return Value::Bool(*i != v2.as_int(vm)),
        (Value::Array(a1), Value::Array(a2)) => return Value::Bool(a1 != a2),
        (Value::Str(s1), Value::Str(s2)) => return Value::Bool(s1 != s2),
        _ => panic!(""),
    }
}

pub fn or(vm: &mut VirtualMachine, args: Vec<Value>) -> Value {
    if args.len() == 0 || args.len() > 2 {
        return Value::Null;
    }
    let x: &Value = &args[0];
    let y: &Value = &args[1];

    match (x, y) {
        (Value::Bool(b1), Value::Bool(b2)) => return Value::Bool(*b1 || *b2),
        _ => return Value::Null,
    }
}

pub fn and(vm: &mut VirtualMachine, args: Vec<Value>) -> Value {
    if args.len() == 0 || args.len() > 2 {
        return Value::Null;
    }
    let x: &Value = &args[0];
    let y: &Value = &args[1];

    match (x, y) {
        (Value::Bool(b1), Value::Bool(b2)) => return Value::Bool(*b1 && *b2),
        _ => return Value::Null,
    }
}

pub fn new_obj(vm: &mut VirtualMachine, _args: Vec<Value>) -> Value {
    return Value::ObjectRef(vm.new_object());
}
