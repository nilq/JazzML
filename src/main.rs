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


pub mod glfw_bindings {
    extern crate glfw;
    use std::mem::transmute;
    use glfw::ffi::*;
    use super::jazzml::codegen::Compiler;
    use super::jazzml::value::*;
    use super::jazzml::vm::VirtualMachine;
    extern "C" {
        fn malloc(size: usize) -> *mut u8;
    }
    use std::ffi::CString;
    pub fn glfw_window_init(_vm: &mut VirtualMachine,args: Vec<Value>) -> Value {
        let name = args[0].as_str(_vm);
        let width = args[1].as_int(_vm);
        let height = args[2].as_int(_vm);
        unsafe {
            let mut window = malloc(std::mem::size_of::<*mut GLFWwindow>()) as *mut _;
            glfwInit();

            window = glfwCreateWindow(width as _,height as _,CString::new(name).unwrap().as_ptr(),0 as _,0 as _);
            if window.is_null() {
                glfwTerminate();
                return Value::Null;
            }

            glfwMakeContextCurrent(window);
            return Value::Int(transmute(window));
        }



    }

    pub fn glfw_window_should_not_close(vm: &mut VirtualMachine,args: Vec<Value>) -> Value {
        let ptr = args[0].as_int(vm);

        unsafe {
            let ret = glfwWindowShouldClose(transmute(ptr));

            return Value::Bool(ret == 0);
        }
    }

    pub fn glfw_swap_buffers(vm: &mut VirtualMachine,args: Vec<Value>) -> Value {
        let ptr = args[0].as_int(vm);

        unsafe {
            glfwSwapBuffers(transmute(ptr));
        }

        Value::Null
    }
    pub fn glfw_poll_events(_vm: &mut VirtualMachine,_args: Vec<Value>) -> Value {
        unsafe {
            glfwPollEvents()
        }
        Value::Null
    }

    pub fn glfw_terminate(_vm: &mut VirtualMachine,_args: Vec<Value>) -> Value {
        unsafe {
            glfwTerminate();
        }
        Value::Null
    }

    pub fn register_funcs(c: &mut Compiler) {

        let id = c.vm.register_native_func("glfwNewWindow".into(),&glfw_window_init,-1);
        c.func_def.insert("glfwNewWindow".into(),id);
        let id = c.vm.register_native_func("glfwWindowShouldNotClose".into(),&glfw_window_should_not_close,-1);
        c.func_def.insert("glfwWindowShouldNotClose".into(),id);
        let id = c.vm.register_native_func("glfwSwapBuffers".into(),&glfw_swap_buffers,-1);
        c.func_def.insert("glfwSwapBuffers".into(),id);
        let id = c.vm.register_native_func("glfwPollEvents".into(),&glfw_poll_events,-1);
        c.func_def.insert("glfwPollEvents".into(),id);
        let id = c.vm.register_native_func("glfwTerminate".into(),&glfw_terminate,-1);
        c.func_def.insert("glfwTerminate".into(),id);

    }
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
            let any = Type::from(TypeNode::Any);
            let int = Type::from(TypeNode::Int);
            let str = Type::from(TypeNode::Str);
            let mut visitor = Visitor::new(ast, &source);
            visitor.assign_str("print",Type::function(vec![Type::from(TypeNode::Any)],Type::from(TypeNode::Any),false));
            visitor.assign_str("new_obj",Type::function(vec![],Type::from(TypeNode::Any),false));
            visitor.assign_str("glfwNewWindow",Type::function(vec![str,int.clone(),int.clone()],int.clone(),false));
            visitor.assign_str("glfwWindowShouldNotClose",Type::function(vec![int.clone()],Type::from(TypeNode::Bool),false));
            visitor.assign_str("glfwTerminate",Type::function(vec![],any.clone(),false));
            visitor.assign_str("glfwPollEvents",Type::function(vec![],any.clone(),false));
            visitor.assign_str("glfwSwapBuffers",Type::function(vec![int],any.clone(),false));
            match visitor.visit() {
                Ok(_) => (),
                _ => return,
            }

            use time::PreciseTime;

            let mut vm = VirtualMachine::new();
            let start = PreciseTime::now();
            let mut compiler = Compiler::new(&mut vm);
            glfw_bindings::register_funcs(&mut compiler);
            compiler.compile(ast.clone());

            let ins = compiler.finish();
            let ret = vm.run_instructions(ins);
            let end = PreciseTime::now();
            let result = start.to(end).num_milliseconds();

            println!("RESULT: {} in {} ms", ret.as_str(&mut vm),result);

        }

        _ => (),
    }
}
