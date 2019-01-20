use super::frame::Frame;
use super::opcodes::Opcode;
use super::parser::*;
use super::value::*;
use super::visitor::ty::Type;
use super::vm::VirtualMachine;

use fnv::{FnvHashMap, FnvHashSet};


pub struct Compiler<'a> {
    pub ins: Vec<Opcode>,
    pub func_def: FnvHashMap<String, usize>,
    pub vm: &'a mut VirtualMachine,
}

impl<'a> Compiler<'a> {
    pub fn new(vm: &'a mut VirtualMachine) -> Compiler<'a> {
        Self {
            ins: vec![],
            func_def: FnvHashMap::default(),
            vm: vm,
        }
    }

    pub fn compile(&mut self, ast: Vec<Statement>) {
        for stmt in ast.iter() {
            self.stmt(stmt.node.clone());
        }
    }

    pub fn stmt(&mut self, s: StatementNode) {
        match s {
            StatementNode::Expression(expr) => self.expr(expr.node),
            StatementNode::Return(r) => {
                if r.is_some() {
                    self.expr(r.unwrap().node.clone());
                    self.ins.push(Opcode::Ret);
                } else {
                    self.ins.push(Opcode::PushNull);
                    self.ins.push(Opcode::Ret);
                }
            }
            StatementNode::Variable(_, _, name, init) => {
                if init.is_some() {
                    let init = init.unwrap().clone();
                    if let ExpressionNode::Function(args, _, block, _) = init.node.clone()
                    {
                        let mut func = Function {
                            args: args.iter().map(|(name,_)| name.clone()).collect::<Vec<String>>(),
                            nargs: args.len() as i32,
                            kind: FuncKind::Interpret(vec![]),
                        };

                        let mut cmpl = Compiler::new(&mut self.vm);
                        cmpl.func_def = self.func_def.clone();
                        cmpl.compile(vec![Statement::new(StatementNode::Expression(block.as_ref().clone()),block.pos.clone())]);
                        func.kind = FuncKind::Interpret(cmpl.ins);
                        

                        self.func_def = cmpl.func_def;
                        let id = self.vm.register_predefiend_func(func);
                        self.func_def.insert(name.to_owned(),id);

                        self.ins.push(Opcode::PushFunc(id));

                    } else {
                        self.expr(init.node);
                    }
                } else {
                    self.ins.push(Opcode::PushNull);
                }
                self.ins.push(Opcode::PushStr(name));
                self.ins.push(Opcode::StoreLocal);
            }
            StatementNode::Assignment(to, val) => {
                self.expr(val.node);
                self.expr(to.node);

                self.ins.push(Opcode::StoreLocal);
            }
            _ => unimplemented!(),
        }
    }

    pub fn expr(&mut self, expr: ExpressionNode) {
        match expr {
            ExpressionNode::Bool(b) => self.ins.push(Opcode::PushBool(b)),
            ExpressionNode::Int(i) => self.ins.push(Opcode::PushInt(i as i64)),
            ExpressionNode::Float(f) => self.ins.push(Opcode::PushFloat(f as f64)),
            ExpressionNode::Str(s) => self.ins.push(Opcode::PushStr(s)),
            ExpressionNode::Char(c) => self.ins.push(Opcode::PushStr(c.to_string())),
            ExpressionNode::Identifier(name) => {
                self.ins.push(Opcode::PushStr(name));
                self.ins.push(Opcode::LoadLocal);
            }
            ExpressionNode::Binary(lhs, op, rhs) => {
                self.expr(lhs.node.clone());
                self.expr(rhs.node.clone());
                match op {
                    Operator::Add => self.ins.push(Opcode::Add),
                    Operator::Sub => self.ins.push(Opcode::Sub),
                    Operator::Mul => self.ins.push(Opcode::Mul),
                    Operator::Mod => self.ins.push(Opcode::Rem),
                    Operator::Div => self.ins.push(Opcode::Div),
                    _ => unimplemented!(),
                }
            }
            ExpressionNode::Empty => {}
            ExpressionNode::EOF => {}
            ExpressionNode::Block(stmt) => {
                for stmt in stmt.iter() {
                    self.stmt(stmt.node.clone());
                }
            }

            ExpressionNode::Call(target,args) => {
                for arg in args.iter().rev() {
                    self.expr(arg.node.clone());
                }
                self.expr(target.node.clone());
                self.ins.push(Opcode::Call(args.len()));

            }

            _ => unimplemented!(),
        }
    }
}
