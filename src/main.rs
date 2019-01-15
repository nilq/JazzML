extern crate peace;

use peace::frame::value;
use peace::opcodes::Opcode;
use peace::value::*;
use peace::vm::VirtualMachine;

pub fn native(_vm: &mut VirtualMachine, _args: Vec<ValueRef>) -> ValueRef {
    println!("Hello,world!");
    value(Value::Null)
}

fn main() {
    let mut vm = VirtualMachine::new();
    let obj_id = vm.new_object();

    let _result = vm.run_instructions(vec![
        Opcode::PushObject(obj_id),
        Opcode::PushStr("obj".into()),
        Opcode::StoreLocal,
        Opcode::PushInt(25),
        Opcode::PushStr("integer".into()),
        Opcode::PushStr("obj".into()),
        Opcode::LoadLocal,
        Opcode::StoreField,
        Opcode::PushNull,
        Opcode::Ret,
    ]);

    let obj = vm.get_object(&obj_id);
    let obj = obj.borrow();

    let int = obj.load(&Value::Str("integer".into()));
    println!("{:?}",int);
    
}
