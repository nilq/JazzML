use std::collections::HashMap;
use std::fmt::{self, Display, Formatter, Write};
use std::rc::Rc;

use super::super::error::Response::*;

use super::*;

#[derive(Debug, Clone)]
pub enum TypeNode {
    Int,
    Float,
    Bool,
    Str,
    Any,
    Char,
    Nil,
    Id(Rc<Expression>),
    Array(Rc<Type>, Option<usize>),
    Func(Vec<Type>, Rc<Type>, Option<Rc<ExpressionNode>>, bool),
    Module(HashMap<String, Type>),
    Struct(HashMap<String, Type>, String),
    This,
}

impl TypeNode {
    pub fn check_expression(&self, other: &ExpressionNode) -> bool {
        use self::TypeNode::*;

        match *other {
            ExpressionNode::Int(_) => match *self {
                Int | Float => true,
                _ => false,
            },

            ExpressionNode::Array(ref content) => {
                let array_content = if let &Array(ref array_content, ref len) = self {
                    if let Some(len) = len {
                        if *len != content.len() {
                            return false;
                        }
                    }

                    array_content
                } else {
                    return false;
                };

                for element in content {
                    if !array_content.node.check_expression(&element.node) {
                        return false;
                    }
                }

                true
            }

            _ => false,
        }
    }

    pub fn strong_cmp(&self, other: &TypeNode) -> bool {
        use self::TypeNode::*;

        match (self, other) {
            (&Int, &Int) => true,
            (&Float, &Float) => true,
            (&Bool, &Bool) => true,
            (&Str, &Str) => true,
            (&Any, &Any) => true,
            (&Char, &Char) => true,
            (&This, &This) => true,
            (&Id(ref a), &Id(ref b)) => a == b,
            (&Array(ref a, ref la), &Array(ref b, ref lb)) => a == b && (la == &None || la == lb),
            (&Func(ref a_params, ref a_retty, .., a), &Func(ref b_params, ref b_retty, .., b)) => {
                a_params == b_params && a_retty == b_retty && a == b
            }
            (&Struct(_, ref content), &Struct(_, ref content_b)) => content == content_b,
            _ => false,
        }
    }
}

impl PartialEq for TypeNode {
    fn eq(&self, other: &Self) -> bool {
        use self::TypeNode::*;

        match (self, other) {
            (&Int, &Int) => true,
            (&Str, &Str) => true,
            (&Float, &Float) => true,
            (&Char, &Char) => true,
            (&Bool, &Bool) => true,
            (&Nil, &Nil) => true,
            (&This, &This) => true,
            (&Array(ref a, ref la), &Array(ref b, ref lb)) => a == b && (la == &None || la == lb),
            (&Id(ref a), &Id(ref b)) => a == b,
            (&Func(ref a_params, ref a_retty, .., a), &Func(ref b_params, ref b_retty, .., b)) => {
                a_params == b_params && a_retty == b_retty && a == b
            }

            (&Struct(_, ref content), &Struct(_, ref content_b)) => content == content_b,

            (&Any, _) => true,
            (_, &Any) => true,

            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TypeMode {
    Undeclared,
    Immutable,
    Optional,
    Implemented,
    Regular,
    Splat(Option<usize>),
    Unwrap(usize),
}

impl TypeMode {
    pub fn strong_cmp(&self, other: &TypeMode) -> bool {
        use self::TypeMode::*;

        match (self, other) {
            (&Regular, &Regular) => true,
            (&Immutable, &Immutable) => true,
            (&Optional, &Optional) => true,
            (&Implemented, &Implemented) => true,
            (&Undeclared, &Undeclared) => true,
            (&Splat(a), &Splat(b)) => &a == &b,
            (&Unwrap(_), &Unwrap(_)) => true,
            _ => false,
        }
    }
}

impl Display for TypeNode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use self::TypeNode::*;

        match *self {
            Int => write!(f, "int"),
            Float => write!(f, "float"),
            Bool => write!(f, "bool"),
            Str => write!(f, "string"),
            Char => write!(f, "char"),
            Nil => write!(f, "nil"),
            This => write!(f, "self"),
            Any => write!(f, "any"),
            Array(ref n, l) => {
                if let Some(len) = l {
                    write!(f, "[{}; {}]", n, len)
                } else {
                    write!(f, "[{}]", n)
                }
            }

            Id(ref n) => write!(f, "deid({})", n.pos.get_lexeme()),

            Module(_) => write!(f, "module"),
            Struct(..) => write!(f, "<struct>"),

            Func(ref params, ref return_type, ..) => {
                write!(f, "fun(")?;

                for (index, element) in params.iter().enumerate() {
                    if index < params.len() - 1 {
                        write!(f, "{}, ", element)?
                    } else {
                        write!(f, "{}", element)?
                    }
                }

                write!(f, ") -> {}", return_type)
            }
        }
    }
}

impl PartialEq for TypeMode {
    fn eq(&self, other: &TypeMode) -> bool {
        use self::TypeMode::*;

        match (self, other) {
            (&Regular, &Regular) => true,
            (&Regular, &Immutable) => true,
            (&Immutable, &Immutable) => true,
            (&Immutable, &Regular) => true,
            (_, &Optional) => true,
            (&Optional, _) => true,
            (&Undeclared, _) => false,
            (_, &Undeclared) => false,
            (&Splat(a), &Splat(b)) => &a == &b,
            (&Unwrap(_), _) => true,
            (_, &Unwrap(_)) => true,
            _ => false,
        }
    }
}

impl Display for TypeMode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use self::TypeMode::*;

        match *self {
            Regular => Ok(()),
            Immutable => write!(f, "constant "),
            Undeclared => write!(f, "undeclared "),
            Optional => write!(f, "optional "),
            Implemented => Ok(()),
            Splat(_) => write!(f, "..."),
            Unwrap(_) => write!(f, "*"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Type {
    pub node: TypeNode,
    pub mode: TypeMode,
}

impl Type {
    pub fn new(node: TypeNode, mode: TypeMode) -> Self {
        Self { node, mode }
    }

    pub fn is_method(&self) -> bool {
        if let TypeNode::Func(.., is_method) = self.node {
            return is_method;
        }

        false
    }

    pub fn id(id: Rc<Expression>) -> Self {
        Type::new(TypeNode::Id(id), TypeMode::Regular)
    }

    pub fn from(node: TypeNode) -> Type {
        Type::new(node, TypeMode::Regular)
    }

    pub fn array(t: Type, len: Option<usize>) -> Type {
        Type::new(TypeNode::Array(Rc::new(t), len), TypeMode::Regular)
    }

    pub fn function(params: Vec<Type>, return_type: Type, is_method: bool) -> Self {
        Type::new(
            TypeNode::Func(params, Rc::new(return_type), None, is_method),
            TypeMode::Regular,
        )
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}{}", self.mode, self.node)
    }
}
