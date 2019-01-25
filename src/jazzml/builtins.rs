use super::value::*;
use super::vm::VirtualMachine;
use fnv::FnvHashMap;
use std::f64;

pub fn value(value: Value) -> Value {
    value
}
use std::iter::FromIterator;

pub fn chars(vm: &mut VirtualMachine, args: Vec<Value>) -> Value {
    let string: String = args[0].clone().as_str(vm);
    let chars = string.chars();
    let vec = Vec::from_iter(chars);
    let mut map = FnvHashMap::default();

    for (idx, value) in vec.iter().enumerate() {
        map.insert(Value::Int(idx as i64), Value::Str(value.to_string()));
    }
    let obj_id = vm.new_object();
    let obj: &mut Object = &mut vm.get_object(&obj_id).borrow_mut();
    obj.map = map;

    return Value::ObjectRef(obj_id);
}

pub fn arr_len(vm: &mut VirtualMachine, args: Vec<Value>) -> Value {
    let arr = args[0].as_object_id();
    let obj: &Object = &vm.get_object(&arr).borrow();

    return Value::Int(obj.map.len() as i64);
}

pub fn arr_push(vm: &mut VirtualMachine, args: Vec<Value>) -> Value {
    let arr = args[0].clone();
    let value = args[1].clone();
    match arr {
        Value::ObjectRef(obj_id) => {
            let obj: &mut Object = &mut vm.get_object(&obj_id).borrow_mut();
            let len = obj.map.len();
            obj.map.insert(Value::Int(len as i64), value);
        }
        _ => panic!("Expected array object"),
    }

    return Value::Null;
}

pub fn arr_pop(vm: &mut VirtualMachine, args: Vec<Value>) -> Value {
    let arr = args[0].clone();
    match arr {
        Value::ObjectRef(obj_id) => {
            let obj: &mut Object = &mut vm.get_object(&obj_id).borrow_mut();
            let len = obj.map.len() - 1;
            let value = obj
                .map
                .remove(&Value::Int(len as i64))
                .unwrap_or(Value::Null);
            return value;
        }
        _ => panic!("Expected array object"),
    }
}

pub fn concat(vm: &mut VirtualMachine, args: Vec<Value>) -> Value {
    let mut buff = String::new();
    for value in args.iter() {
        buff.push_str(&value.as_str(vm));
    }
    return Value::Str(buff);
}

pub fn print(vm: &mut VirtualMachine, args: Vec<Value>) -> Value {
    for value in args.iter() {
        print!("{}", value.as_str(vm));
    }

    Value::Null
}

pub fn println(vm: &mut VirtualMachine, args: Vec<Value>) -> Value {
    for value in args.iter() {
        print!("{}", value.as_str(vm));
    }
    println!("");
    Value::Null
}

extern "C" {
    fn getchar() -> u32;
    fn putchar(c: u32);
}

pub fn get_char(_: &mut VirtualMachine, _args: Vec<Value>) -> Value {
    use std::char;
    let character = char::from_u32(unsafe { getchar() }).unwrap();
    return Value::Str(character.to_string());
}

use std::char;
pub fn put_char(vm: &mut VirtualMachine, _args: Vec<Value>) -> Value {
    
    let v = match &_args[0] {
        Value::Str(s) => s.chars().nth(0).unwrap(),
        Value::Int(ch) => char::from_u32(*ch as u32).unwrap(),
        _ => panic!(),
    };
    unsafe {
        putchar(v as u32);
    }
    return Value::Null;
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
        //(Value::Array(a1), Value::Array(a2)) => return value(Value::Bool(a1.len() > a2.len())),
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
        //(Value::Array(a1), Value::Array(a2)) => return value(Value::Bool(a1.len() < a2.len())),
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

        v => panic!("{:?}",v),
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
        (v1, Value::Int(i)) => return Value::Bool(*i != v1.as_int(vm)),
        (v1, Value::Float(f)) => return Value::Bool(f64::from_bits(*f) != v1.as_f64(vm)),
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
