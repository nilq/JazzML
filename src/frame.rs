use crate::opcodes::Opcode;
use crate::value::*;
use fnv::FnvHashMap;
use crate::vm::VirtualMachine;
use std::cell::RefCell;
use crate::builtins;


pub struct Frame<'a> {
    pub code: Vec<Opcode>,
    pub locals: FnvHashMap<String,ValueRef>,
    pub pc: usize,
    pub stack: Vec<ValueRef>,
    pub vm: &'a mut VirtualMachine
}

pub fn value(v: Value) -> ValueRef {
    Ref::new(RefCell::new(v))
}

impl<'a> Frame<'a> {
    pub fn new(vm: &mut VirtualMachine) -> Frame {
        Frame {
            code: vec![],
            locals: FnvHashMap::default(),
            pc: 0,
            stack: vec![],
            vm,
        }
    }

    pub fn fetch_opcode(&mut self) -> Opcode {
        let ins = self.code[self.pc].clone();
        self.pc += 1;
        ins
    }

    pub fn pop(&mut self) -> ValueRef {
        self.stack.pop().unwrap()
    }

    pub fn push(&mut self,v: ValueRef) {
        self.stack.push(v);
    }

    pub fn execute_call(&mut self,v: ValueRef,argc: usize,obj_call: bool) -> ValueRef {
        let v: &Value = &v.borrow();

        match v {
            &Value::ObjectRef(id) => {
                let obj = self.vm.get_object(&id).clone();
                let obj: &Object = &obj.borrow();
                let reference = obj.load(&Value::Str("__call__".into()));
                self.stack.push(value(Value::ObjectRef(id)));
                self.execute_call(reference.clone(), argc,true)
            }
            &Value::FuncRef(id) => {
                let func: &Function = &self.vm.get_func(&id).borrow().clone();
                match func.nargs {
                    -1 /* VAR ARGS */ => {
                        let mut temp = vec![];
                        for _ in 0..argc {
                            temp.push(self.pop());
                        }
                        let ret = match func.kind.clone() {
                            FuncKind::Native(f) => f(self.vm,temp),
                            FuncKind::Interpret(v) => {
                                let mut frame = Frame::new(self.vm);
                                frame.code = v;
                                frame.stack = temp;
                                frame.run_frame()
                            }
                        };

                        return ret;
                    }
                    nargs => {
                        if nargs != argc as i32 {
                            panic!("Expected {} argument(s) found {}",nargs,argc);
                        }

                        let mut temp = vec![];
                        if obj_call {
                            temp.push(self.pop());
                        }
                        
                        for _ in 0..nargs {
                            temp.push(self.pop());
                        }

                        let ret = match &func.kind {
                            FuncKind::Native(f) => f(self.vm,temp),
                            FuncKind::Interpret(v) => {
                                let mut frame = Frame::new(self.vm);
                                frame.code = v.clone();
                                for (arg,arg_name) in temp.iter().zip(&func.args) {
                                    frame.locals.insert(arg_name.clone(), arg.clone());
                                }
                                frame.run_frame()
                            }
                        };
                        return ret;
                    }
                }

                
            }
            _ => panic!("Can't call value `{:?}`",v)
        }
    }

    pub fn run_frame(&mut self) -> ValueRef {
        let value = loop 
        {
            let result = self.execute_op();
            match result {
                None => (),
                Some(res) => break Some(res),
            }
        };

        value.unwrap().clone()
    }

    pub fn execute_op(&mut self) -> Option<ValueRef> {
        let ins = self.fetch_opcode();

        match ins {
            Opcode::PushNull => {
                self.push(value(Value::Null));
                None
            }
            Opcode::PushBool(b) => {
                self.push(value(Value::Bool(b)));
                None
            }
            Opcode::PushInt(int) => {
                self.stack.push(Ref::new(RefCell::new(Value::Int(int))));
                None
            }
            Opcode::PushFloat(float) => {
                self.stack.push(Ref::new(RefCell::new(Value::Float(float.to_bits()))));
                None
            }

            Opcode::PushStr(str) => {
                self.stack.push(Ref::new(RefCell::new(Value::Str(str))));
                None
            }
            Opcode::Amake(arr_len) => {
                let mut temp = vec![];
                for _ in 0..arr_len {
                    temp.push(self.pop());
                }

                self.push(Ref::new(RefCell::new(Value::Array(temp))));
                None
            }

            Opcode::Add => {
                let x = self.pop();
                let y = self.pop();

                let z = builtins::add(self.vm,vec![x,y]);
                self.push(z);
                None
            }

            Opcode::Sub => {
                let x = self.pop();
                let y = self.pop();

                let z = builtins::sub(self.vm,vec![x,y]);
                self.push(z);
                None
            }
            Opcode::Mul => {
                let x = self.pop();
                let y = self.pop();

                let z = builtins::mul(self.vm,vec![x,y]);
                self.push(z);
                None
            }

            Opcode::Div => {
                let x = self.pop();
                let y = self.pop();

                let z = builtins::div(self.vm,vec![x,y]);
                self.push(z);
                None
            }

            Opcode::Rem => {
                let x = self.pop();
                let y = self.pop();

                let z = builtins::rem(self.vm,vec![x,y]);
                self.push(z);
                None
            }
            

            Opcode::LoadLocal => {
                let name: ValueRef = self.pop();
                let name_str = name.borrow().as_str(self.vm);

                let val = self.locals.get(&name_str).expect(&format!("Local `{}` doesn't exists",name_str));

                self.push(val.clone());
                None
            }

            Opcode::StoreLocal => {
                let name: ValueRef = self.pop();
                let name_str = name.borrow().as_str(self.vm);
                let val = self.pop();
                self.locals.insert(name_str,val);
                None
            }

            Opcode::StoreField => {
                let target = self.pop();
                let key = self.pop();
                let val = self.pop();
                let key: &Value = &key.borrow();
                let target: &Value = &target.borrow();
                match target {
                    Value::ObjectRef(id) => {
                        let object: &mut Object = &mut self.vm.get_object(id).borrow_mut();
                        object.store(key.clone(), val);
                    }
                    _ => panic!("Can't load field on `{:?}`",target)
                }
                None
            }
            Opcode::LoadField => {
                let target = self.pop();
                let key = self.pop();
                let target: &Value = &target.borrow();
                let key: &Value = &key.borrow();
                let result = match target {
                    Value::ObjectRef(id) => 
                    {
                        let object = self.vm.get_object(id).borrow();
                        object.load(key).clone()
                    }
                    _ => panic!("Can't load field on `{:?}`",target),
                };
                self.push(result);
                None
            }

            Opcode::LoadGlobal => {
                let key = self.pop();
                let val = self.vm.globals.get(&key.borrow()).unwrap();

                self.stack.push(val.clone());
                None
            }

            Opcode::Call(nargs) => {
                let target = self.pop();
                let result = self.execute_call(target, nargs,false);
                self.push(result);
                None
            }

            Opcode::Ret => {
                let ret = self.pop();
                return Some(ret);
            }

            _ => unimplemented!()
        }
    }
}