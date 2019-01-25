use super::frame::Frame;
use super::opcodes::Opcode;
use super::parser::*;
use super::value::*;
use super::visitor::ty::Type;
use super::vm::VirtualMachine;

use fnv::FnvHashMap;

pub struct Compiler<'a> {
    pub ins: Vec<UOP>,
    pub func_def: FnvHashMap<String, usize>,
    pub vm: &'a mut VirtualMachine,
    pub labels: FnvHashMap<String, Option<usize>>,
    end_label: Option<String>,
}

#[derive(Clone, Debug)]
pub enum UOP {
    Op(Opcode),
    Goto(String),
}

impl<'a> Compiler<'a> {
    pub fn new(vm: &'a mut VirtualMachine) -> Compiler<'a> {
        Self {
            ins: vec![],
            func_def: FnvHashMap::default(),
            vm: vm,
            labels: FnvHashMap::default(),
            end_label: None,
        }
    }

    pub fn compile(&mut self, ast: Vec<Statement>) {
        self.vm.init_builtins();
        let print = self
            .vm
            .globals
            .get(&Value::Str("print".to_owned()))
            .expect("print not found");

        if let Value::FuncRef(id) = print {
            self.func_def.insert("print".into(), *id);
        }
        let print = self
            .vm
            .globals
            .get(&Value::Str("println".to_owned()))
            .expect("println not found");

        if let Value::FuncRef(id) = print {
            self.func_def.insert("println".into(), *id);
        }

        let obj = self
            .vm
            .globals
            .get(&Value::Str("new_obj".to_owned()))
            .expect("new_obj not found");

        if let Value::FuncRef(id) = obj {
            self.func_def.insert("new_obj".into(), *id);
        }

        let obj = self
            .vm
            .globals
            .get(&Value::Str("array_push".to_owned()))
            .expect("array_push not found");

        if let Value::FuncRef(id) = obj {
            self.func_def.insert("array_push".into(), *id);
        }

        let obj = self
            .vm
            .globals
            .get(&Value::Str("array_pop".to_owned()))
            .expect("array_pop not found");

        if let Value::FuncRef(id) = obj {
            self.func_def.insert("array_pop".into(), *id);
        }

        let obj = self
            .vm
            .globals
            .get(&Value::Str("array_len".to_owned()))
            .expect("array_len not found");

        if let Value::FuncRef(id) = obj {
            self.func_def.insert("array_len".into(), *id);
        }

        let obj = self
            .vm
            .globals
            .get(&Value::Str("getc".to_owned()))
            .expect("getc not found");

        if let Value::FuncRef(id) = obj {
            self.func_def.insert("getc".into(), *id);
        }

        let obj = self
            .vm
            .globals
            .get(&Value::Str("putc".to_owned()))
            .expect("putc not found");

        if let Value::FuncRef(id) = obj {
            self.func_def.insert("putc".into(), *id);
        }

        let obj = self
            .vm
            .globals
            .get(&Value::Str("chars".to_owned()))
            .expect("chars not found");

        if let Value::FuncRef(id) = obj {
            self.func_def.insert("chars".into(), *id);
        }
        for stmt in ast.iter() {
            self.stmt(stmt.node.clone());
        }
        self.emit(Opcode::PushNull);
        self.emit(Opcode::Ret);
    }

    pub fn new_empty_label(&mut self) -> String {
        let lab_name = self.labels.len().to_string();
        self.labels.insert(lab_name.clone(), None);
        lab_name
    }

    pub fn label_here(&mut self, label: &str) {
        *self.labels.get_mut(label).unwrap() = Some(self.ins.len());
    }

    pub fn new_label_here(&mut self, s: String) {
        self.labels.insert(s, Some(self.ins.len()));
    }

    pub fn emit(&mut self, op: Opcode) {
        self.ins.push(UOP::Op(op));
    }

    pub fn emit_goto(&mut self, lbl: &str) {
        self.ins.push(UOP::Goto(lbl.to_owned()));
    }

    pub fn finish(&mut self) -> Vec<Opcode> {
        self.ins
            .iter()
            .map(|e| match e {
                &UOP::Goto(ref lbl) => Opcode::Jmp(self.labels.get(lbl).unwrap().unwrap()),
                &UOP::Op(ref op) => op.clone(),
            })
            .collect::<Vec<Opcode>>()
    }

    pub fn stmt(&mut self, s: StatementNode) {
        match s {
            StatementNode::Expression(expr) => self.expr(expr.node),
            StatementNode::Return(r) => {
                if r.is_some() {
                    self.expr(r.unwrap().node.clone());
                    self.emit(Opcode::Ret);
                } else {
                    self.emit(Opcode::PushNull);
                    self.emit(Opcode::Ret);
                }
            }
            StatementNode::Variable(_, _, name, init) => {
                if init.is_some() {
                    let init = init.unwrap().clone();
                    if let ExpressionNode::Function(args, _, block, _) = init.node.clone() {
                        let mut func = Function {
                            args: args
                                .iter()
                                .map(|(name, _)| name.clone())
                                .collect::<Vec<String>>(),
                            nargs: args.len() as i32,
                            kind: FuncKind::Interpret(vec![]),
                        };

                        let mut cmpl = Compiler::new(&mut self.vm);
                        cmpl.func_def = self.func_def.clone();
                        cmpl.compile(vec![Statement::new(
                            StatementNode::Expression(block.as_ref().clone()),
                            block.pos.clone(),
                        )]);
                        let ins = cmpl.finish();
                        func.kind = FuncKind::Interpret(ins);

                        self.func_def = cmpl.func_def;
                        let id = self.vm.register_predefiend_func(func);
                        self.func_def.insert(name.to_owned(), id);

                        self.emit(Opcode::PushFunc(id));
                    } else {
                        self.expr(init.node);
                    }
                } else {
                    self.emit(Opcode::PushNull);
                }
                self.emit(Opcode::PushStr(name));
                self.emit(Opcode::StoreLocal);
            }
            StatementNode::Break => {
                let end = self.end_label.clone().unwrap().clone();

                self.emit_goto(&end);
            }
            StatementNode::Assignment(to, val) => {
                if let ExpressionNode::Index(a, b, is_arr) = to.node.clone() {
                    self.expr(val.node);
                    match b.node.clone() {
                        ExpressionNode::Identifier(ref name) => {
                            if !is_arr {
                                self.emit(Opcode::PushStr(name.to_owned()));
                            } else {
                                self.expr(ExpressionNode::Identifier(name.to_owned()));
                            }
                        }
                        ExpressionNode::Index(_, _, _) => self.expr(b.node.clone()),
                        ExpressionNode::Int(i) => self.emit(Opcode::PushInt(i as i64)),
                        v => panic!("{:?}", v),
                    };

                    self.expr(a.node.clone());
                    self.emit(Opcode::StoreField);
                } else {
                    self.expr(val.node);
                    if let ExpressionNode::Identifier(ref name) = to.node.clone() {
                        self.emit(Opcode::PushStr(name.to_owned()));
                    }
                    self.emit(Opcode::StoreLocal);
                }
            }
            _ => unimplemented!(),
        }
    }

    pub fn expr(&mut self, expr: ExpressionNode) {
        match expr {
            ExpressionNode::Bool(b) => self.emit(Opcode::PushBool(b)),
            ExpressionNode::Int(i) => self.emit(Opcode::PushInt(i as i64)),
            ExpressionNode::Float(f) => self.emit(Opcode::PushFloat(f as f64)),
            ExpressionNode::Str(s) => self.emit(Opcode::PushStr(s)),
            ExpressionNode::Char(c) => self.emit(Opcode::PushStr(c.to_string())),
            ExpressionNode::Identifier(name) => {
                if self.func_def.contains_key(&name) {
                    let id = self.func_def.get(&name).expect("not found");

                    self.emit(Opcode::PushFunc(*id));
                    return;
                }
                self.emit(Opcode::PushStr(name));
                self.emit(Opcode::LoadLocal);
            }

            ExpressionNode::Binary(lhs, op, rhs) => {
                self.expr(rhs.node.clone());
                self.expr(lhs.node.clone());

                match op {
                    Operator::Add => self.emit(Opcode::Add),
                    Operator::Sub => self.emit(Opcode::Sub),
                    Operator::Mul => self.emit(Opcode::Mul),
                    Operator::Mod => self.emit(Opcode::Rem),
                    Operator::Div => self.emit(Opcode::Div),
                    Operator::Lt => self.emit(Opcode::Lt),
                    Operator::Gt => self.emit(Opcode::Gt),
                    Operator::Eq => self.emit(Opcode::Eq),
                    Operator::NEq => self.emit(Opcode::Neq),
                    Operator::Or => self.emit(Opcode::Or),
                    _ => unimplemented!(),
                }
            }
            ExpressionNode::Empty => self.emit(Opcode::PushNull),
            ExpressionNode::EOF => {}
            ExpressionNode::Block(stmt) => {
                for stmt in stmt.iter() {
                    self.stmt(stmt.node.clone());
                }
            }

            ExpressionNode::Array(arr) => {
                for value in arr.iter().rev() {
                    self.expr(value.node.clone());
                }
                self.emit(Opcode::Amake(arr.len()));
            }

            ExpressionNode::Index(a, b, is_arr) => {
                
                match b.node.clone() {
                    ExpressionNode::Identifier(ref name) => {
                        if is_arr {
                            self.expr(b.node.clone());
                        } else {
                            self.emit(Opcode::PushStr(name.to_owned()));
                        }
                    }
                    ExpressionNode::Index(_, _, _) => self.expr(b.node.clone()),
                    ExpressionNode::Int(idx) => {
                        self.emit(Opcode::PushInt(idx as i64));
                    }
                    _ => unimplemented!(),
                };
                
                self.expr(a.node.clone());
                self.emit(Opcode::LoadField);
            }

            ExpressionNode::Call(target, args) => {
                for arg in args.iter().rev() {
                    self.expr(arg.node.clone());
                }
                self.expr(target.node.clone());
                self.emit(Opcode::Call(args.len()));
            }
            ExpressionNode::While(cond, block) => {
                let while_block = self.new_empty_label();
                let while_end = self.new_empty_label();
                let while_start = self.new_empty_label();
                self.end_label = Some(while_end.clone());
                self.emit_goto(&while_start);

                self.label_here(&while_block);
                self.expr(block.node.clone());
                self.label_here(&while_start);
                self.expr(cond.node.clone());

                let l = self.labels.clone();
                self.emit(Opcode::JmpT(l.get(&while_block.clone()).unwrap().unwrap()));
                self.label_here(&while_end);
            }

            ExpressionNode::If(cond, then, or) => {
                self.expr(cond.node.clone());
                let if_true = self.new_empty_label();
                let if_false = self.new_empty_label();
                let check = self.new_empty_label();
                let end = self.new_empty_label();

                self.emit_goto(&check);
                self.label_here(&if_true);
                self.expr(then.node.clone());
                self.emit_goto(&end);
                self.label_here(&if_false);
                if or.is_some() {
                    let r: Vec<_> = or.unwrap();
                    for (cond, block, _) in r.iter() {
                        if cond.is_none() {
                            self.expr(block.node.clone());
                            break;
                        } else {
                            let if_true2 = self.new_empty_label();
                            let check2 = self.new_empty_label();
                            self.emit_goto(&check2);
                            self.label_here(&if_true2);
                            self.expr(block.node.clone());
                            self.emit_goto(&end);

                            self.label_here(&check2);
                            self.expr(cond.clone().unwrap().node.clone());
                            let l = self.labels.clone();
                            self.emit(Opcode::JmpT(l.get(&if_true2).unwrap().unwrap()));
                        }
                    }
                } else {
                    self.emit(Opcode::Nop);
                }
                self.emit_goto(&end);

                let l = self.labels.clone();

                self.label_here(&check);

                self.emit(Opcode::JmpT(l.get(&if_true).unwrap().unwrap()));
                self.emit(Opcode::Jmp(l.get(&if_false).unwrap().unwrap()));
                self.label_here(&end);
            }

            ExpressionNode::Function(args, _, block, _) => {
                let mut func = Function {
                    args: args
                        .iter()
                        .map(|(name, _)| name.clone())
                        .collect::<Vec<String>>(),
                    nargs: args.len() as i32,
                    kind: FuncKind::Interpret(vec![]),
                };

                let mut cmpl = Compiler::new(&mut self.vm);
                cmpl.func_def = self.func_def.clone();
                cmpl.compile(vec![Statement::new(
                    StatementNode::Expression(block.as_ref().clone()),
                    block.pos.clone(),
                )]);
                let ins = cmpl.finish();
                func.kind = FuncKind::Interpret(ins);

                self.func_def = cmpl.func_def;
                let id = self.vm.register_predefiend_func(func);
                self.emit(Opcode::PushFunc(id));
            }
            ExpressionNode::Cast(e, _) => self.expr(e.node.clone()),

            v => panic!("{:?}", v),
        }
    }
}
