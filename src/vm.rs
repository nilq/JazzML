use crate::value::{ObjectRef,ValueRef,Value,FuncRef,FuncKind,Function,Object};
use fnv::FnvHashMap;
use crate::opcodes::Opcode;
use crate::frame::Frame;
use std::cell::RefCell;

pub struct VirtualMachine {
    pub functions: FnvHashMap<usize,FuncRef>,
    pub globals: FnvHashMap<Value,ValueRef>,
    pub pool: FnvHashMap<usize,ObjectRef>,
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

    pub fn run_instructions(&mut self,ins: Vec<Opcode>) -> ValueRef {
        let mut frame = Frame::new(self);
        frame.code = ins;

        frame.run_frame()
    }

    pub fn register_func(&mut self,name: String,ins: Vec<Opcode>,nargs: i32,args: Vec<String>) {
        let func = Function {
            kind: FuncKind::Interpret(ins),
            nargs,
            args,
        };
        self.globals.insert(Value::Str(name),ValueRef::new(RefCell::new(Value::FuncRef(self.fid))));
        self.functions.insert(self.fid,FuncRef::new(RefCell::new(func)));
        self.fid += 1;
    }

    pub fn register_native_func<T: Fn(&mut VirtualMachine,Vec<ValueRef>) -> ValueRef>(&mut self,name: String,f: &'static T,nargs: i32) {
        let func = Function {
            kind: FuncKind::Native(f),
            nargs,
            args: vec![],
        };
        
        self.globals.insert(Value::Str(name),ValueRef::new(RefCell::new(Value::FuncRef(self.fid))));
        self.functions.insert(self.fid,FuncRef::new(RefCell::new(func)));
        self.fid += 1;
    }

    pub fn get_object(&self,s: &usize) -> &ObjectRef {
        self.pool.get(s).unwrap()
    }

    pub fn get_func(&self,s: &usize) -> &FuncRef {
        self.functions.get(s).unwrap()
    }

    pub fn register_object(&mut self,name: String,obj: Object) {
        self.pool.insert(self.oid,ObjectRef::new(RefCell::new(obj)));
        self.globals.insert(Value::Str(name),ValueRef::new(RefCell::new(Value::ObjectRef(self.oid))));
        self.oid += 1;
    }
}