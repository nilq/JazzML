extern crate colored;

pub mod jazzml;

use self::jazzml::opcodes::Opcode;
use self::jazzml::value::*;
use self::jazzml::vm::VirtualMachine;

use self::jazzml::lexer::*;
use self::jazzml::source::*;
use self::jazzml::parser::*;
use self::jazzml::visitor::*;

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
let Foo = struct {
  x: int
  y: any
}

let fib = func(a: int) : int {
    if a < 3 {
        a
    } else {
        fib(a - 1) + fib(a - 2)
    }
}

fib(100)

var bar: Foo = new Foo {
  x: 100
  y: fib(-10)
}

fib(bar)
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
    
    let mut parser = Parser::new(tokens, &source);

    match parser.parse() {
        Ok(ref ast) => {
            let mut visitor = Visitor::new(ast, &source);

            match visitor.visit() {
                Ok(_) => (),
                _     => return
            }
        },

        _ => ()
    }
}
