extern crate peace;


use peace::vm::VirtualMachine;
use peace::opcodes::Opcode;
use peace::value::*;
use peace::frame::value;

pub fn native(_vm: &mut VirtualMachine,_args: Vec<ValueRef>) -> ValueRef {
    println!("Hello,world!");
    value(Value::Null)
}



fn main() {
    let mut vm = VirtualMachine::new();

    vm.register_native_func("native".into(), &native, -1);

    let result = vm.run_instructions(vec![
        Opcode::PushInt(2),
        Opcode::PushInt(3),
        Opcode::Add,
        Opcode::Ret
    ]);

    println!("{:?}",result);
}
