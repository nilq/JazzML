extern crate colored;

pub mod jazzml;

use self::jazzml::opcodes::Opcode;
use self::jazzml::value::*;
use self::jazzml::vm::VirtualMachine;

use self::jazzml::lexer::*;
use self::jazzml::source::*;

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


    let test_code = r#"
let fib = $ func(a: int) : int {
  return switch a {
      0 => 0
      1 => 1
      _ => fib(a)
  }
}


    "#;

    let source = Source::from("test.jazzml", test_code.lines().map(|x| x.into()).collect::<Vec<String>>());
    let lexer  = Lexer::default(test_code.chars().collect(), &source);

    let mut tokens = Vec::new();

    for token_result in lexer {
        if let Ok(token) = token_result {
            tokens.push(token)
        } else {
            return
        }
    }

    println!("{:#?}", tokens)
}
