extern crate colored;

pub mod jazzml;

use self::jazzml::opcodes::Opcode;
use self::jazzml::value::*;
use self::jazzml::vm::VirtualMachine;

use self::jazzml::codegen::Compiler;
use self::jazzml::lexer::*;
use self::jazzml::parser::*;
use self::jazzml::source::*;
use self::jazzml::visitor::*;

pub fn native(_vm: &mut VirtualMachine, _args: Vec<Value>) -> Value {
    println!("Hello,world!");
    Value::Null
}

use std::env::args;
use std::fs::File;
use std::io::Read;

fn main() {
    let file = args().nth(1).unwrap();
    let mut handle = File::open(&file).unwrap();
    let mut code = String::new();

    handle.read_to_string(&mut code).unwrap();

    let source = Source::from(
        &file,
        code.lines().map(|x| x.into()).collect::<Vec<String>>(),
    );
    let lexer = Lexer::default(code.chars().collect(), &source);

    let mut tokens = Vec::new();

    for token_result in lexer {
        if let Ok(token) = token_result {
            tokens.push(token)
        } else {
            return;
        }
    }

    let mut parser = Parser::new(tokens, &source);

    match parser.parse() {
        Ok(ref ast) => {
           
            //let mut visitor = Visitor::new(ast, &source);
            //visitor.assign("print".into(), Type::function(vec![Type::from(TypeNode::VaArgs);1], Type::from(TypeNode::Any), false));
            /*match visitor.visit() {
                Ok(_) => (),
                _     => return
            }*/
            use time::{PreciseTime};
            let mut vm = VirtualMachine::new();
            let start = PreciseTime::now();
            let mut compiler = Compiler::new(&mut vm);
            compiler.compile(ast.clone());
            let ins = compiler.finish();
            for (idx,ins) in ins.iter().enumerate() {
                println!("{}: {:?}",idx,ins);
            }
            let ret = vm.run_instructions(ins);
            let end = PreciseTime::now();
            let result = start.to(end).num_milliseconds();
            println!("{} ms",result);
            //println!("{:?}", ret);
        }

        _ => (),
    }
}
