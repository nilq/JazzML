use std::collections::HashMap;
use std::fmt::{self, Display, Formatter, Write};
use std::rc::Rc;

use super::super::error::Response::*;

use super::ty::*;
use super::*;

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[derive(Debug, Clone)]
pub enum FlagContext {
    Block(Option<Type>),
    Nothing,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Inside {
    Loop,
    Calling(Pos),
    Splat(Option<usize>),
    Implement(Type),
    Function,
    Nothing,
}

#[derive(Clone)]
pub struct ImplementationFrame {
    pub implementations: HashMap<String, HashMap<String, Type>>,
}

impl ImplementationFrame {
    pub fn new() -> Self {
        Self {
            implementations: HashMap::new(),
        }
    }
}

pub struct Visitor<'v> {
    pub symtab: SymTab,

    pub source: &'v Source,
    pub ast: &'v Vec<Statement>,

    pub flag: Option<FlagContext>,
    pub inside: Vec<Inside>,

    pub method_calls: HashMap<Pos, bool>,
}

impl<'v> Visitor<'v> {
    pub fn visit(&mut self) -> Result<(), ()> {
        self.visit_block(self.ast, false)?;

        Ok(())
    }

    pub fn new(ast: &'v Vec<Statement>, source: &'v Source) -> Self {
        Visitor {
            symtab: SymTab::new(),

            source,
            ast,

            flag: None,
            inside: Vec::new(),

            method_calls: HashMap::new(),
        }
    }

    fn visit_statement(&mut self, statement: &Statement) -> Result<(), ()> {
        use self::StatementNode::*;

        match statement.node {
            Expression(ref expr) => self.visit_expression(expr),
            Variable(..) => self.visit_variable(&statement.node, &statement.pos),

            Return(ref value) => {
                if self.inside.contains(&Inside::Function) {
                    if let Some(ref expression) = *value {
                        self.visit_expression(expression)
                    } else {
                        Ok(())
                    }
                } else {
                    return Err(response!(
                        Wrong("can't return outside of function"),
                        self.source.file,
                        statement.pos
                    ));
                }
            }

            Break => {
                if self.inside.contains(&Inside::Loop) {
                    Ok(())
                } else {
                    return Err(response!(
                        Wrong("can't break outside loop"),
                        self.source.file,
                        statement.pos
                    ));
                }
            }

            Continue => {
                if self.inside.contains(&Inside::Loop) {
                    Ok(())
                } else {
                    return Err(response!(
                        Wrong("can't continue outside loop"),
                        self.source.file,
                        statement.pos
                    ));
                }
            }

            Assignment(ref left, ref right) => {
                self.visit_expression(left)?;
                self.visit_expression(right)?;

                let a = self.type_expression(left)?;
                let b = self.type_expression(right)?;

                self.assert_types(a, b, &left.pos)?;

                Ok(())
            }

            _ => Ok(()),
        }
    }

    fn visit_expression(&mut self, expression: &Expression) -> Result<(), ()> {
        use self::ExpressionNode::*;

        match expression.node {
            Identifier(ref name) => {
                self.fetch(name, &expression.pos)?;

                Ok(())
            }

            Struct(ref params, _) => {
                let mut name_buffer = Vec::new();

                for &(ref name, _) in params.iter() {
                    if name_buffer.contains(&name) {
                        return Err(response!(
                            Wrong(format!("field `{}` defined more than once", name)),
                            self.source.file,
                            expression.pos
                        ));
                    }

                    name_buffer.push(&name)
                }

                Ok(())
            }

            Neg(ref expr) => {
                let expr_type = self.type_expression(expr)?;

                match expr_type.node {
                    TypeNode::Float | TypeNode::Int => Ok(()),

                    _ => Err(response!(
                        Wrong(format!("can't negate type `{}`", expr_type)),
                        self.source.file,
                        expression.pos
                    )),
                }
            }

            Not(ref expr) => {
                let expr_type = self.type_expression(expr)?;

                if expr_type.node.strong_cmp(&TypeNode::Bool) {
                    Ok(())
                } else {
                    Err(response!(
                        Wrong(format!("can't negate type `{}`", expr_type)),
                        self.source.file,
                        expression.pos
                    ))
                }
            }

            Initialization(ref left, ref args) => {
                let struct_type = self.type_expression(&*left)?;

                if struct_type.node != TypeNode::Any {
                    if let TypeNode::Struct(ref content, _) = struct_type.node {
                        if struct_type.mode.strong_cmp(&TypeMode::Undeclared) {
                            for arg in args.iter() {
                                self.visit_expression(&arg.1)?;

                                let arg_type = self.type_expression(&arg.1)?;

                                if let Some(ref content_type) = content.get(&arg.0) {
                                    if !content_type
                                        .node
                                        .check_expression(&Parser::fold_expression(&arg.1)?.node)
                                        && arg_type != **content_type
                                    {
                                        return Err(response!(
                                            Wrong(format!(
                                                "mismatched types, expected `{}` got `{}`",
                                                content_type, arg_type
                                            )),
                                            self.source.file,
                                            expression.pos
                                        ));
                                    }
                                } else {
                                    return Err(response!(
                                        Wrong(format!("no such member `{}` in struct", arg.0)),
                                        self.source.file,
                                        arg.1.pos
                                    ));
                                }
                            }
                        } else {
                            return Err(response!(
                                Wrong(format!(
                                    "can't initialize non-struct: `{}`",
                                    struct_type.node
                                )),
                                self.source.file,
                                expression.pos
                            ));
                        }
                    } else {
                        return Err(response!(
                            Wrong(format!(
                                "can't initialize non-struct: `{}`",
                                struct_type.node
                            )),
                            self.source.file,
                            expression.pos
                        ));
                    }
                }

                Ok(())
            }

            Block(ref statements) => {
                self.push_scope();

                self.visit_block(statements, true)?;

                self.pop_scope();

                Ok(())
            }

            If(ref condition, ref body, ref elses) => {
                self.visit_expression(&*condition)?;

                let condition_type = self.type_expression(&*condition)?.node;

                if condition_type == TypeNode::Bool {
                    self.push_scope();

                    self.visit_expression(body)?;
                    let body_type = self.type_expression(body)?;

                    self.pop_scope();

                    if let &Some(ref elses) = elses {
                        for &(ref maybe_condition, ref body, _) in elses {
                            if let Some(ref condition) = *maybe_condition {
                                let condition_type = self.type_expression(condition)?.node;

                                if condition_type != TypeNode::Bool {
                                    return Err(response!(
                                        Wrong(format!(
                                            "mismatched condition, must be `bool` got `{}`",
                                            condition_type
                                        )),
                                        self.source.file,
                                        condition.pos
                                    ));
                                }
                            }

                            self.push_scope();

                            self.visit_expression(body)?;
                            let else_body_type = self.type_expression(body)?;

                            self.pop_scope();

                            if body_type != else_body_type {
                                return Err(response!(
                                    Wrong(format!(
                                        "mismatched types, expected `{}` got `{}`",
                                        body_type, else_body_type
                                    )),
                                    self.source.file,
                                    body.pos
                                ));
                            }
                        }
                    }

                    Ok(())
                } else {
                    return Err(response!(
                        Wrong(format!(
                            "mismatched condition, must be `bool` got `{}`",
                            condition_type
                        )),
                        self.source.file,
                        expression.pos
                    ));
                }
            }

            While(ref condition, ref body) => {
                self.visit_expression(&*condition)?;

                let condition_type = self.type_expression(&*condition)?.node;

                if condition_type == TypeNode::Bool {
                    self.inside.push(Inside::Loop);

                    self.push_scope();

                    self.visit_expression(body)?;

                    let body_type = self.type_expression(body)?;

                    if body_type.node != TypeNode::Nil {
                        let body_pos = if let Block(ref content) = body.node {
                            content.last().unwrap().pos.clone()
                        } else {
                            unreachable!()
                        };

                        return Err(response!(
                            Wrong("mismatched types, expected `()`"),
                            self.source.file,
                            body_pos
                        ));
                    }

                    self.pop_scope();

                    self.inside.pop();

                    Ok(())
                } else {
                    return Err(response!(
                        Wrong(format!(
                            "mismatched condition, must be `bool` got `{}`",
                            condition_type
                        )),
                        self.source.file,
                        expression.pos
                    ));
                }
            }

            Array(ref content) => {
                let t = self.type_expression(content.first().unwrap())?;

                for element in content {
                    let element_type = self.type_expression(element)?;

                    if !t
                        .node
                        .check_expression(&Parser::fold_expression(element)?.node)
                        && t.node != element_type.node
                    {
                        return Err(response!(
                            Wrong(format!(
                                "mismatched types in array, expected `{}` got `{}`",
                                t, element_type
                            )),
                            self.source.file,
                            element.pos
                        ));
                    }
                }

                Ok(())
            }

            Call(ref expr, ref args) => {
                self.visit_expression(expr)?;

                self.inside.push(Inside::Calling(expr.pos.clone()));

                let expression_type = self.type_expression(expr)?;

                if let TypeNode::Func(ref params, _, ref func, .., is_method) = expression_type.node
                {
                    // this is where we visit the func, no diggity
                    if let Some(func) = func {
                        self.visit_expression(&Expression::new(
                            (**func).clone(),
                            expression.pos.clone(),
                        ))?;
                    }

                    if is_method {
                        self.method_calls.insert(expression.pos.clone(), true);
                    }

                    let mut actual_arg_len = args.len();
                    let mut type_buffer: Option<Type> = None;

                    for (i, param_type) in params.iter().enumerate() {
                        let param_type = self.deid(param_type.clone())?;
                        let arg_type = self.type_expression(&args[i])?;

                        if !param_type
                            .node
                            .check_expression(&Parser::fold_expression(&args[i])?.node)
                            && arg_type.node != param_type.node
                        {
                            return Err(response!(
                                Wrong(format!(
                                    "mismatched types, expected type `{}` got `{}`",
                                    param_type.node, arg_type
                                )),
                                self.source.file,
                                args[i].pos
                            ));
                        }

                        let arg_type = if i < args.len() {
                            self.type_expression(&args[i])?
                        } else {
                            type_buffer.as_ref().unwrap().clone()
                        };

                        let mode = arg_type.mode.clone();

                        if let TypeMode::Unwrap(ref len) = mode {
                            type_buffer = Some(arg_type.clone());

                            actual_arg_len += len
                        }
                    }

                    if actual_arg_len > params.len() {
                        let last = self.deid(params.last().unwrap().clone())?;

                        if let TypeMode::Splat(_) = last.mode {
                            for splat in &args[params.len()..] {
                                let splat_type = self.type_expression(&splat)?;

                                if !last.node.check_expression(&splat.node)
                                    && last.node != splat_type.node
                                {
                                    return Err(response!(
                                        Wrong(format!(
                                            "mismatched splat argument, expected `{}` got `{}`",
                                            last, splat_type
                                        )),
                                        self.source.file,
                                        splat.pos
                                    ));
                                }
                            }
                        }

                        self.inside
                            .push(Inside::Splat(Some(actual_arg_len - params.len())))
                    }

                    self.visit_expression(expr)?;

                    self.inside.pop();

                    if actual_arg_len != params.len() {
                        match params.last().unwrap().mode {
                            TypeMode::Splat(_) => (),
                            _ => {
                                return Err(response!(
                                    Wrong(format!(
                                        "expected {} argument{} got {}",
                                        params.len(),
                                        if params.len() > 1 { "s" } else { "" },
                                        actual_arg_len
                                    )),
                                    self.source.file,
                                    args.last().unwrap_or(expression).pos
                                ));
                            }
                        }
                    }
                }

                Ok(())
            }

            Function(ref params, ref retty, ref body, ref is_method) => {
                let mut frame_hash = HashMap::new();

                let mut return_type = self.deid(retty.clone())?;

                if let TypeNode::Id(ref ident) = retty.node {
                    self.visit_expression(&ident)?;

                    let ident_type = self.type_expression(&ident)?;

                    if let TypeNode::Struct(..) = ident_type.node {
                        return_type = Type::from(ident_type.node)
                    } else {
                        return Err(response!(
                            Wrong(format!("can't use `{}` as type", ident_type)),
                            self.source.file,
                            ident.pos
                        ));
                    }
                }

                return_type = Type::from(return_type.node.clone());

                let mut found_splat = false;

                for param in params.iter() {
                    if let TypeMode::Splat(_) = param.1.mode {
                        if found_splat {
                            return Err(response!(
                                Wrong("can't have multiple splat parameters in function"),
                                self.source.file,
                                expression.pos
                            ));
                        }

                        found_splat = true
                    }

                    frame_hash.insert(param.0.clone(), param.1.clone());
                }

                if *is_method {
                    let mut found = false;

                    for inside in self.inside.iter().rev() {
                        // ffs
                        if let Inside::Implement(ref t) = inside {
                            found = true;
                        }
                    }

                    if !found {
                        return Err(response!(
                            Wrong("can't define method outside implementation"),
                            self.source.file,
                            expression.pos
                        ));
                    }
                }

                self.symtab
                    .put_frame(Frame::from(frame_hash, self.symtab.stack.len()));

                self.inside.push(Inside::Function);

                self.visit_expression(body)?;

                self.symtab.revert_frame(); // we'll need those

                let body_type = self.type_expression(body)?;

                self.pop_scope(); // we don't need those anymore

                self.inside.pop();

                self.pop_scope();

                if return_type.node != body_type.node {
                    Err(response!(
                        Wrong(format!(
                            "mismatched return type, expected `{}` got `{}`",
                            return_type, body_type
                        )),
                        self.source.file,
                        body.pos
                    ))
                } else {
                    Ok(())
                }
            }

            Index(ref left, ref index) => {
                let mut left_type = self.type_expression(left)?;

                if let TypeMode::Splat(_) = left_type.mode {
                    left_type = Type::from(TypeNode::Array(Rc::new(left_type.clone()), None))
                }

                match left_type.node {
                    TypeNode::Array(_, ref len) => {
                        self.inside.push(Inside::Nothing);

                        self.visit_expression(index)?;

                        let index_type = self.type_expression(index)?;

                        match index_type.node {
                            TypeNode::Int => {
                                if let Int(ref a) = Parser::fold_expression(index)?.node {
                                    if let Some(len) = len {
                                        if *a as usize > *len {
                                            return Err(response!(
                                                Wrong(format!(
                                                    "index out of bounds, len is {} got {}",
                                                    len, a
                                                )),
                                                self.source.file,
                                                left.pos
                                            ));
                                        }
                                    }
                                }
                            }

                            _ => {
                                return Err(response!(
                                    Wrong(format!(
                                        "can't index with `{}`, must be unsigned integer",
                                        index_type
                                    )),
                                    self.source.file,
                                    left.pos
                                ));
                            }
                        }
                    }

                    TypeNode::Module(ref content) => {
                        self.inside.push(Inside::Nothing);

                        if let Identifier(ref name) = index.node {
                            if !content.contains_key(name) {
                                return Err(response!(
                                    Wrong(format!("no such module member `{}`", name)),
                                    self.source.file,
                                    index.pos
                                ));
                            }
                        } else {
                            let index_type = self.type_expression(index)?;

                            return Err(response!(
                                Wrong(format!("can't index module with `{}`", index_type)),
                                self.source.file,
                                index.pos
                            ));
                        }
                    }

                    TypeNode::Struct(ref content, ref id) => {
                        self.inside.push(Inside::Implement(left_type.clone()));

                        if let Identifier(ref name) = index.node {
                            if !content.contains_key(name) && !self.is_implemented(id, name) {
                                return Err(response!(
                                    Wrong(format!("no such struct member `{}`", name)),
                                    self.source.file,
                                    index.pos
                                ));
                            }
                        } else {
                            let index_type = self.type_expression(index)?;

                            return Err(response!(
                                Wrong(format!("can't index struct with `{}`", index_type)),
                                self.source.file,
                                index.pos
                            ));
                        }
                    }

                    TypeNode::Any => (),

                    _ => {
                        return Err(response!(
                            Wrong(format!("can't index type `{}`", left_type)),
                            self.source.file,
                            left.pos
                        ));
                    }
                }

                Ok(())
            }

            _ => Ok(()),
        }
    }

    fn visit_variable(&mut self, variable: &StatementNode, pos: &Pos) -> Result<(), ()> {
        use self::ExpressionNode::*;

        if let &StatementNode::Variable(ref is_mutable, ref var_type, ref name, ref right) =
            variable
        {
            let mut variable_type = var_type.clone();

            if let TypeNode::Id(ref ident) = var_type.node {
                let ident_type = self.type_expression(&ident)?;

                if let TypeNode::Struct(..) = ident_type.node {
                    variable_type = Type::from(ident_type.node)
                } else {
                    return Err(response!(
                        Wrong(format!("can't use `{}` as type", ident_type)),
                        self.source.file,
                        ident.pos
                    ));
                }
            }

            variable_type = Type::from(variable_type.node.clone());

            if let &Some(ref right) = right {
                match right.node {
                    Function(..) | Block(_) | If(..) | While(..) => (),
                    _ => self.visit_expression(right)?,
                }

                let right_type = self.type_expression(&right)?;

                if variable_type.node != TypeNode::Nil {
                    if !variable_type
                        .node
                        .check_expression(&Parser::fold_expression(right)?.node)
                        && variable_type.node != right_type.node
                    {
                        return Err(response!(
                            Wrong(format!(
                                "mismatched types, expe cted type `{}` got `{}`",
                                variable_type.node, right_type.node
                            )),
                            self.source.file,
                            right.pos
                        ));
                    } else {
                        self.assign(name.to_owned(), variable_type.to_owned())
                    }
                } else {
                    self.assign(name.to_owned(), right_type)
                }

                match right.node {
                    Function(..) | Block(_) | If(..) | While(..) => self.visit_expression(right)?,
                    _ => (),
                }
            } else {
                self.assign(name.to_owned(), variable_type.to_owned())
            }

            Ok(())
        } else {
            unreachable!()
        }
    }

    pub fn type_statement(&mut self, statement: &Statement) -> Result<Type, ()> {
        use self::StatementNode::*;

        let t = match statement.node {
            Expression(ref expression) => self.type_expression(expression)?,
            Return(ref expression) => {
                if let Some(ref expression) = *expression {
                    self.type_expression(expression)?
                } else {
                    Type::from(TypeNode::Nil)
                }
            }
            _ => Type::from(TypeNode::Nil),
        };

        Ok(t)
    }

    fn type_expression(&mut self, expression: &Expression) -> Result<Type, ()> {
        use self::ExpressionNode::*;

        let t = match expression.node {
            Identifier(ref name) => {
                let t = self.fetch(name, &expression.pos)?;

                self.deid(t)?
            }

            Struct(ref params, ref id) => {
                let mut param_hash = HashMap::new();

                for param in params {
                    param_hash.insert(
                        param.0.clone(),
                        Type::from(self.deid(param.1.clone())?.node),
                    );
                }

                Type::new(
                    TypeNode::Struct(param_hash, id.to_string()),
                    TypeMode::Undeclared,
                )
            }

            Str(_) => Type::from(TypeNode::Str),
            Char(_) => Type::from(TypeNode::Char),
            Bool(_) => Type::from(TypeNode::Bool),
            Int(_) => Type::from(TypeNode::Int),
            Float(_) => Type::from(TypeNode::Float),

            Array(ref content) => Type::array(
                self.type_expression(content.first().unwrap())?,
                Some(content.len()),
            ),

            Initialization(ref name, ref content) => {
                let struct_type = Type::from(self.type_expression(name)?.node);

                if struct_type.node == TypeNode::Any {
                    let mut new_content = HashMap::new();

                    for (name, ty) in content {
                        new_content.insert(name.clone(), self.type_expression(ty)?);
                    }

                    Type::from(TypeNode::Struct(new_content, String::new()))
                } else {
                    struct_type
                }
            }

            If(_, ref body, ..) => self.type_expression(body)?,

            Index(ref array, ref index) => {
                let mut kind = self.type_expression(array)?;

                if let TypeMode::Splat(_) = kind.mode {
                    kind = Type::from(TypeNode::Array(Rc::new(kind.clone()), None))
                }

                match kind.node {
                    TypeNode::Array(ref t, _) => (**t).clone(),
                    TypeNode::Any => Type::new(TypeNode::Any, kind.mode),

                    TypeNode::Module(ref content) => {
                        if let Identifier(ref name) = index.node {
                            if let Some(kind) = content.get(name) {
                                kind.clone()
                            } else {
                                return Err(response!(
                                    Wrong(format!("no such module member `{}`", name)),
                                    self.source.file,
                                    index.pos
                                ));
                            }
                        } else {
                            unreachable!()
                        }
                    }

                    TypeNode::Struct(ref content, ref struct_id) => {
                        if let Identifier(ref name) = index.node {
                            if !self.is_implemented(struct_id, name) {
                                if let Some(kind2) = content.get(name) {
                                    if kind.mode.strong_cmp(&TypeMode::Undeclared) {
                                        if kind2.is_method() {
                                            return Err(
                        response!(
                          Wrong(format!("can't access non-static method `{}` on undeclared struct", name)),
                          self.source.file,
                          index.pos
                        )
                      );
                                        } else if !kind2.mode.strong_cmp(&TypeMode::Implemented) {
                                            return Err(
                        response!(
                          Wrong(format!("can't access uninitialized value `{}` on undeclared struct", name)),
                          self.source.file,
                          index.pos
                        )
                      );
                                        }
                                    }

                                    kind2.clone()
                                } else {
                                    return Err(response!(
                                        Wrong(format!("no such struct member `{}`", name)),
                                        self.source.file,
                                        index.pos
                                    ));
                                }
                            } else {
                                self.symtab.get_implementation_force(struct_id, name)
                            }
                        } else {
                            unreachable!()
                        }
                    }

                    _ => {
                        return Err(response!(
                            Wrong(format!("can't index type `{}`", kind)),
                            self.source.file,
                            expression.pos
                        ));
                    }
                }
            }

            Call(ref expression, _) => {
                if let TypeNode::Func(_, ref return_type, ..) =
                    self.type_expression(expression)?.node
                {
                    (**return_type).clone()
                } else {
                    panic!(
                        "BAM! (please submit an issue): called {:#?}",
                        self.type_expression(expression)?.node
                    )
                }
            }

            Function(ref params, ref return_type, _, is_method) => {
                let mut param_types = Vec::new();

                for param in params {
                    param_types.push(self.deid(param.1.clone())?)
                }

                let return_type = self.deid(return_type.clone())?;

                Type::from(TypeNode::Func(
                    param_types,
                    Rc::new(return_type),
                    Some(Rc::new(expression.node.clone())),
                    is_method,
                ))
            }

            Block(ref statements) => {
                let flag_backup = self.flag.clone();

                if self.flag.is_none() {
                    self.flag = Some(FlagContext::Block(None))
                }

                self.push_scope();

                let block_type = if statements.len() > 0 {
                    for element in statements {
                        match element.node {
                            StatementNode::Expression(ref expression) => match expression.node {
                                Block(_) | If(..) | While(..) => {
                                    self.type_expression(expression)?;
                                }

                                _ => (),
                            },

                            StatementNode::Return(ref return_type) => {
                                let flag = self.flag.clone();

                                if let Some(ref flag) = flag {
                                    if let &FlagContext::Block(ref consistent) = flag {
                                        let return_type =
                                            if let Some(ref return_type) = *return_type {
                                                self.type_expression(&return_type)?
                                            } else {
                                                Type::from(TypeNode::Nil)
                                            };

                                        if let Some(ref consistent) = *consistent {
                                            if return_type != *consistent {
                                                return Err(
                          response!(
                            Wrong(format!("mismatched types, expected `{}` found `{}`", consistent, return_type)),
                            self.source.file,
                            expression.pos
                          )
                        );
                                            }
                                        } else {
                                            self.flag =
                                                Some(FlagContext::Block(Some(return_type.clone())))
                                        }
                                    }
                                }
                            }

                            _ => (),
                        }
                    }

                    self.visit_expression(&expression)?;

                    self.symtab.revert_frame();

                    let last = statements.last().unwrap();
                    let implicit_type = self.type_statement(last)?;

                    self.pop_scope();

                    if let Some(flag) = self.flag.clone() {
                        if let FlagContext::Block(ref consistent) = flag {
                            if let Some(ref consistent) = *consistent {
                                if implicit_type.node != consistent.node {
                                    return Err(response!(
                                        Wrong(format!(
                                            "mismatched types, expected `{}` found `{}`",
                                            consistent, implicit_type
                                        )),
                                        self.source.file,
                                        last.pos
                                    ));
                                }
                            } else {
                                self.flag = Some(FlagContext::Block(Some(implicit_type.clone())))
                            }
                        }
                    }

                    implicit_type
                } else {
                    Type::from(TypeNode::Nil)
                };

                self.flag = flag_backup;

                block_type
            }

            Cast(_, ref t) => t.to_owned(),

            Binary(ref left, ref op, ref right) => {
                use self::Operator::*;

                match (
                    self.type_expression(left)?.node,
                    op,
                    self.type_expression(right)?.node,
                ) {
                    (ref a, ref op, ref b) => match **op {
                        Add | Sub | Mul | Div | Mod => {
                            if [a, b] != [&TypeNode::Nil, &TypeNode::Nil] {
                                // real hack here
                                if a == b {
                                    match a {
                                        TypeNode::Float | TypeNode::Int => match b {
                                            TypeNode::Float | TypeNode::Int => {
                                                Type::from(a.clone())
                                            }

                                            _ => {
                                                return Err(response!(
                                                    Wrong(format!(
                                                        "can't perform operation `{} {} {}`",
                                                        a, op, b
                                                    )),
                                                    self.source.file,
                                                    expression.pos
                                                ));
                                            }
                                        },

                                        _ => {
                                            return Err(response!(
                                                Wrong(format!(
                                                    "can't perform operation `{} {} {}`",
                                                    a, op, b
                                                )),
                                                self.source.file,
                                                expression.pos
                                            ));
                                        }
                                    }
                                } else {
                                    return Err(response!(
                                        Wrong(format!(
                                            "can't perform operation `{} {} {}`",
                                            a, op, b
                                        )),
                                        self.source.file,
                                        expression.pos
                                    ));
                                }
                            } else {
                                return Err(response!(
                                    Wrong(format!("can't perform operation `{} {} {}`", a, op, b)),
                                    self.source.file,
                                    expression.pos
                                ));
                            }
                        }

                        Pow => match a {
                            TypeNode::Float | TypeNode::Int => match b {
                                TypeNode::Float | TypeNode::Int => Type::from(a.clone()),

                                _ => {
                                    return Err(response!(
                                        Wrong(format!(
                                            "can't perform operation `{} {} {}`",
                                            a, op, b
                                        )),
                                        self.source.file,
                                        expression.pos
                                    ));
                                }
                            },

                            _ => {
                                return Err(response!(
                                    Wrong(format!("can't perform operation `{} {} {}`", a, op, b)),
                                    self.source.file,
                                    expression.pos
                                ));
                            }
                        },

                        And | Or => {
                            if a == b && *a == TypeNode::Bool {
                                Type::from(TypeNode::Bool)
                            } else {
                                return Err(response!(
                                    Wrong(format!("can't perform operation `{} {} {}`", a, op, b)),
                                    self.source.file,
                                    expression.pos
                                ));
                            }
                        }

                        Concat => {
                            if *a == TypeNode::Str {
                                match *b {
                                    TypeNode::Func(..) | TypeNode::Array(..) => {
                                        return Err(response!(
                                            Wrong(format!(
                                                "can't perform operation `{} {} {}`",
                                                a, op, b
                                            )),
                                            self.source.file,
                                            expression.pos
                                        ));
                                    }

                                    _ => Type::from(TypeNode::Str),
                                }
                            } else {
                                return Err(response!(
                                    Wrong(format!("can't perform operation `{} {} {}`", a, op, b)),
                                    self.source.file,
                                    expression.pos
                                ));
                            }
                        }

                        Eq | Lt | Gt | NEq | LtEq | GtEq => {
                            if a == b {
                                Type::from(TypeNode::Bool)
                            } else {
                                return Err(response!(
                                    Wrong(format!("can't perform operation `{} {} {}`", a, op, b)),
                                    self.source.file,
                                    expression.pos
                                ));
                            }
                        }

                        _ => {
                            return Err(response!(
                                Wrong(format!("can't perform operation `{} {} {}`", a, op, b)),
                                self.source.file,
                                expression.pos
                            ));
                        }
                    },
                }
            }

            Neg(ref expr) => self.type_expression(expr)?,
            Not(_) => Type::from(TypeNode::Bool),

            _ => Type::from(TypeNode::Nil),
        };

        self.deid(t)
    }

    // `ensure_implicit` gets mad at wannabe implicit returns
    fn visit_block(&mut self, content: &Vec<Statement>, ensure_implicits: bool) -> Result<(), ()> {
        for (i, statement) in content.iter().enumerate() {
            // ommiting functions, for that extra user-feel
            if let StatementNode::Variable(.., ref name, ref value) = statement.node {
                if let Some(ref right) = *value {
                    if let ExpressionNode::Function(ref params, ref retty, .., is_method) =
                        right.node
                    {
                        let mut types = params.iter().map(|x| x.1.clone()).collect::<Vec<Type>>();

                        let t = Type::from(TypeNode::Func(
                            types,
                            Rc::new(retty.clone()),
                            Some(Rc::new(right.node.clone())),
                            is_method,
                        ));

                        self.assign(name.to_owned(), t);

                        continue;
                    }
                }
            }

            if ensure_implicits {
                if i < content.len() - 1 {
                    if let StatementNode::Expression(ref expression) = statement.node {
                        self.ensure_no_implicit(expression)?
                    }
                }
            }

            self.visit_statement(&statement)?
        }

        for statement in content.iter() {
            if let StatementNode::Variable(.., ref right) = statement.node {
                if let Some(ref right) = *right {
                    if let ExpressionNode::Function(..) = right.node {
                        self.visit_statement(statement)?
                    }
                }
            }
        }

        Ok(())
    }

    fn ensure_no_implicit(&self, expression: &Expression) -> Result<(), ()> {
        use self::ExpressionNode::*;

        match expression.node {
            Block(ref statements) => {
                if let Some(statement) = statements.last() {
                    if let StatementNode::Expression(ref expression) = statement.node {
                        match expression.node {
                            Call(..) => (),
                            Block(..) => {
                                self.ensure_no_implicit(expression)?;
                            }

                            If(_, ref expr, _) | While(_, ref expr) => {
                                self.ensure_no_implicit(&*expr)?
                            }

                            _ => {
                                return Err(response!(
                                    Wrong("unexpected expression without context"),
                                    self.source.file,
                                    expression.pos
                                ));
                            }
                        }
                    }

                    ()
                } else {
                    ()
                }
            }

            Call(..) => (),

            If(_, ref expr, _) | While(_, ref expr) => self.ensure_no_implicit(&*expr)?,

            _ => {
                return Err(response!(
                    Wrong("unexpected expression without context"),
                    self.source.file,
                    expression.pos
                ));
            }
        }

        Ok(())
    }

    fn assert_types(&self, a: Type, b: Type, pos: &Pos) -> Result<bool, ()> {
        if a != b {
            Err(response!(
                Wrong(format!("mismatched types, expected `{}` got `{}`", a, b)),
                self.source.file,
                pos
            ))
        } else {
            Ok(true)
        }
    }

    fn fetch(&self, name: &String, pos: &Pos) -> Result<Type, ()> {
        if let Some(t) = self.symtab.fetch(name) {
            Ok(t)
        } else {
            Err(response!(
                Wrong(format!("can't seem to find `{}`", name)),
                self.source.file,
                pos
            ))
        }
    }

    fn fetch_str(&self, name: &str, pos: &Pos) -> Result<Type, ()> {
        if let Some(t) = self.symtab.fetch_str(name) {
            Ok(t)
        } else {
            Err(response!(
                Wrong(format!("can't seem to find `{}`", name)),
                self.source.file,
                pos
            ))
        }
    }

    fn assign_str(&mut self, name: &str, t: Type) {
        self.symtab.assign_str(name, t)
    }

    fn assign(&mut self, name: String, t: Type) {
        self.symtab.assign(name, t)
    }

    fn push_scope(&mut self) {
        self.symtab.push()
    }

    fn pop_scope(&mut self) {
        self.symtab.pop()
    }

    pub fn deid(&mut self, t: Type) -> Result<Type, ()> {
        if let TypeNode::Id(ref expr) = t.node {
            let mut new_t = self.type_expression(expr)?;

            new_t.mode = t.mode.clone();

            Ok(new_t)
        } else {
            Ok(t)
        }
    }

    pub fn is_implemented(&mut self, struct_id: &String, method_name: &String) -> bool {
        if let Some(ref content) = self.symtab.get_implementations(struct_id) {
            return content.contains_key(method_name);
        }

        false
    }
}
