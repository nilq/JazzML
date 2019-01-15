use crate::value::*;
use crate::vm::VirtualMachine;
use crate::frame::value;
use std::f64;

pub fn add(vm: &mut VirtualMachine,args: Vec<ValueRef>) -> ValueRef {
    if args.len() == 0 || args.len() > 2 {
        return value(Value::Null);
    }
    let x: &Value = &args[0].borrow();
    let y: &Value = &args[1].borrow();
    match (x,y) {
        (Value::Float(f),v2) => return value(Value::Float((f64::from_bits(*f) + v2.as_f64(vm)).to_bits())),
        (Value::Int(i),  Value::Float(f2)) => return value(Value::Float((*i as f64 + f64::from_bits(*f2)).to_bits())),
        (Value::Int(i), v2) => return value(Value::Int(*i + v2.as_int(vm))),
        (Value::Str(s),v) => {
            let str: &str = &v.as_str(vm);
            let mut buff = s.clone();
            buff.push_str(str);
            return value(Value::Str(buff));
        }
        (Value::Array(arr1),Value::Array(arr2)) => {
            unimplemented!()
        }
        _ => panic!("")
    }
}

pub fn sub(vm: &mut VirtualMachine,args: Vec<ValueRef>) -> ValueRef {
    if args.len() == 0 || args.len() > 2 {
        return value(Value::Null);
    }
    let x: &Value = &args[0].borrow();
    let y: &Value = &args[1].borrow();
    match (x,y) {
        (Value::Float(f),v2) => return value(Value::Float((f64::from_bits(*f) - v2.as_f64(vm)).to_bits())),
        (Value::Int(i),  Value::Float(f2)) => return value(Value::Float((*i as f64 - f64::from_bits(*f2)).to_bits())),
        (Value::Int(i), v2) => return value(Value::Int(*i - v2.as_int(vm))),
        _ => panic!("")
    }
}

pub fn mul(vm: &mut VirtualMachine,args: Vec<ValueRef>) -> ValueRef {
    if args.len() == 0 || args.len() > 2 {
        return value(Value::Null);
    }
    let x: &Value = &args[0].borrow();
    let y: &Value = &args[1].borrow();
    match (x,y) {
        (Value::Float(f),v2) => return value(Value::Float((f64::from_bits(*f) * v2.as_f64(vm)).to_bits())),
        (Value::Int(i),  Value::Float(f2)) => return value(Value::Float((*i as f64 * f64::from_bits(*f2)).to_bits())),
        (Value::Int(i), v2) => return value(Value::Int(*i * v2.as_int(vm))),
        _ => panic!("")
    }
}

pub fn div(vm: &mut VirtualMachine,args: Vec<ValueRef>) -> ValueRef {
    if args.len() == 0 || args.len() > 2 {
        return value(Value::Null);
    }
    let x: &Value = &args[0].borrow();
    let y: &Value = &args[1].borrow();
    match (x,y) {
        (Value::Float(f),v2) => return value(Value::Float((f64::from_bits(*f) / v2.as_f64(vm)).to_bits())),
        (Value::Int(i),  Value::Float(f2)) => return value(Value::Float((*i as f64 / f64::from_bits(*f2)).to_bits())),
        (Value::Int(i), v2) => return value(Value::Int(*i / v2.as_int(vm))),
        _ => panic!("")
    }
}

pub fn rem(vm: &mut VirtualMachine,args: Vec<ValueRef>) -> ValueRef {
    if args.len() == 0 || args.len() > 2 {
        return value(Value::Null);
    }
    let x: &Value = &args[0].borrow();
    let y: &Value = &args[1].borrow();
    match (x,y) {
        (Value::Float(f),v2) => return value(Value::Float((f64::from_bits(*f) % v2.as_f64(vm)).to_bits())),
        (Value::Int(i),  Value::Float(f2)) => return value(Value::Float((*i as f64 % f64::from_bits(*f2)).to_bits())),
        (Value::Int(i), v2) => return value(Value::Int(*i % v2.as_int(vm))),
        _ => panic!("")
    }
}