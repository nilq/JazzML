use crate::frame::Frame;
use crate::opcodes::Opcode;
use crate::value::{FuncKind, FuncRef, Function, Object, ObjectRef, Value, ValueRef};
use fnv::FnvHashMap;
use std::cell::RefCell;

pub struct VirtualMachine {
    pub functions: FnvHashMap<usize, FuncRef>,
    pub globals: FnvHashMap<Value, ValueRef>,
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
        use crate::builtins::*;

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
            new_obj:-1,
            gt:2,
            lt:2
        }
    }

    pub fn run_instructions(&mut self, ins: Vec<Opcode>) -> ValueRef {
        let mut frame = Frame::new(self);
        frame.code = ins;

        frame.run_frame()
    }

    pub fn register_func(&mut self, name: String, ins: Vec<Opcode>, nargs: i32, args: Vec<String>) {
        let func = Function {
            kind: FuncKind::Interpret(ins),
            nargs,
            args,
        };
        self.globals.insert(
            Value::Str(name),
            ValueRef::new(RefCell::new(Value::FuncRef(self.fid))),
        );
        self.functions
            .insert(self.fid, FuncRef::new(RefCell::new(func)));
        self.fid += 1;
    }

    pub fn register_native_func<T: Fn(&mut VirtualMachine, Vec<ValueRef>) -> ValueRef>(
        &mut self,
        name: String,
        f: &'static T,
        nargs: i32,
    ) {
        let func = Function {
            kind: FuncKind::Native(f),
            nargs,
            args: vec![],
        };

        self.globals.insert(
            Value::Str(name),
            ValueRef::new(RefCell::new(Value::FuncRef(self.fid))),
        );
        self.functions
            .insert(self.fid, FuncRef::new(RefCell::new(func)));
        self.fid += 1;
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
}
