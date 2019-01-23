use super::frame::Frame;
use super::opcodes::Opcode;
use super::value::{FuncKind, FuncRef, Function, Object, ObjectRef, Value};

use fnv::FnvHashMap;
use std::cell::RefCell;

pub struct VirtualMachine {
    pub functions: FnvHashMap<usize, FuncRef>,
    pub globals: FnvHashMap<Value, Value>,
    pub pool: FnvHashMap<usize, ObjectRef>,
    fid: usize,
    oid: usize,
}

impl VirtualMachine {
    pub fn new() -> VirtualMachine {
        VirtualMachine {
            functions: FnvHashMap::default(),
            globals: FnvHashMap::default(),
            pool: FnvHashMap::default(),
            fid: 0,
            oid: 0,
        }
    }

    pub fn init_builtins(&mut self) {
        use super::builtins::*;

        self.register_native_func(format!("concat"),&concat,-1);
        self.register_native_func(format!("print"), &print, -1);
        self.register_native_func(format!("new_obj"),&new_obj,-1);
        macro_rules! register_native {
            ($($fname: ident: $argc: expr),+) => {
                $(
                self.register_native_func(format!("__{}__",stringify!($fname)), &$fname, $argc);
                )+
            };
        }
        register_native! {
            add:2,
            sub:2,
            div:2,
            mul:2,
            rem:2,
            or:2,
            bor:2,
            and:2,
            band:2,
            bxor:2,
            shr:2,
            shl:2,
            eq:2,
            neq:2,
            gt:2,
            lt:2
        }
    }

    pub fn run_instructions(&mut self, ins: Vec<Opcode>) -> Value {
        let mut locals = FnvHashMap::default();
        let mut frame = Frame::new_with_ins(self,&mut locals,ins);


        frame.run_frame()
    }

    pub fn run_func(&mut self, func: usize, args: Vec<Value>) -> Value {
        let func: &FuncRef = &self
            .functions
            .get(&func)
            .expect("function not defined")
            .clone();
        let func: &Function = &func.borrow();

        match &func.kind {
            FuncKind::Interpret(ins) => {
                let mut args = {
                    let mut temp = FnvHashMap::default();

                    for (arg, arg_name) in args.iter().zip(&func.args) {
                        temp.insert(arg_name.to_owned(), arg.clone());
                    }
                    temp
                };
                let mut frame = Frame::new(self,&mut args);

                frame.code = ins.clone();
                frame.run_frame()
            }
            FuncKind::Native(f) => f(self, args),
        }
    }

    pub fn register_predefiend_func(&mut self, f: Function) -> usize {
        let func_id = self.fid;
        self.functions
            .insert(self.fid, FuncRef::new(RefCell::new(f)));
        self.fid += 1;
        func_id
    }

    pub fn new_func(&mut self, nargs: i32, args: Vec<String>) -> usize {
        let func = Function {
            kind: FuncKind::Interpret(vec![]),
            nargs,
            args,
        };
        let func_id = self.fid;
        self.functions
            .insert(self.fid, FuncRef::new(RefCell::new(func)));
        self.fid += 1;
        func_id
    }

    pub fn register_func(
        &mut self,
        name: String,
        ins: Vec<Opcode>,
        nargs: i32,
        args: Vec<String>,
    ) -> usize {
        let func = Function {
            kind: FuncKind::Interpret(ins),
            nargs,
            args,
        };
        self.globals
            .insert(Value::Str(name), Value::FuncRef(self.fid));
        let id = self.fid;
        self.functions
            .insert(self.fid, FuncRef::new(RefCell::new(func)));
        self.fid += 1;
        id
    }

    pub fn register_native_func<T: Fn(&mut VirtualMachine, Vec<Value>) -> Value>(
        &mut self,
        name: String,
        f: &'static T,
        nargs: i32,
    ) -> usize {
        let func = Function {
            kind: FuncKind::Native(f),
            nargs,
            args: vec![],
        };
        let id = self.fid;
        self.globals
            .insert(Value::Str(name), Value::FuncRef(self.fid));
        self.functions
            .insert(self.fid, FuncRef::new(RefCell::new(func)));
        self.fid += 1;
        id
    }

    pub fn get_object(&self, s: &usize) -> &ObjectRef {
        self.pool.get(s).unwrap()
    }

    pub fn get_func(&self, s: &usize) -> &FuncRef {
        self.functions.get(s).unwrap()
    }

    pub fn new_object(&mut self) -> usize {
        let id = self.oid;
        self.pool
            .insert(self.oid, ObjectRef::new(RefCell::new(Object::new())));
        self.oid += 1;
        id
    }

    pub fn register_object(&mut self, obj: Object) -> usize {
        let id = self.oid;

        self.pool.insert(id, ObjectRef::new(RefCell::new(obj)));
        self.oid += 1;
        id
    }
}
