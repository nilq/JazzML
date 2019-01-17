extern crate jazz_ml_vm;

use jazz_ml_vm::opcodes::Opcode;
use jazz_ml_vm::value::*;
use jazz_ml_vm::vm::VirtualMachine;
use std::cell::RefCell;

pub fn native(_vm: &mut VirtualMachine, _args: Vec<Value>) -> Value {
    println!("Hello,world!");
    Value::Null
}

fn main() {
    let mut vm = VirtualMachine::new();

    let func = Function {
        args: vec![],
        nargs: -1,
        kind: FuncKind::Interpret(
            vec![
                Opcode::PushStr("Hello,world!".into()),
                Opcode::Ret,
            ]
        )
    };

    let fid = vm.register_predefiend_func(func);

    let mut object = Object::new();
    object.store(Value::Str("method".into()), Value::FuncRef(fid));

    let obj_id = vm.register_object(object);

    let f = vm.get_func(&fid);
    println!("{:?}",f.borrow().kind);

    let func = vm.register_func("main".into(), vec![
        Opcode::PushObject(obj_id),
        Opcode::PushStr("method".into()),
        Opcode::CallObj(0),
        Opcode::Ret,
    ], 0,vec![]);

    println!("{:?}",vm.run_func(func, vec![]));
}
