use super::opcodes::Opcode;
use super::vm::VirtualMachine;
use std::cell::Ref as SRef;
use std::cell::RefCell;
use std::f64;
use std::rc::Rc;

pub type Ref<T> = Rc<RefCell<T>>;
pub type ObjectRef = Ref<Object>;
pub type FuncRef = Ref<Function>;

pub const VAR_ARGS: i32 = -1;

#[derive(Clone)]
pub enum FuncKind {
    Native(&'static Fn(&mut VirtualMachine, Vec<Value>) -> Value),
    Interpret(Vec<Opcode>),
}

use std::fmt;

impl fmt::Debug for FuncKind {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FuncKind::Interpret(code) => write!(fmt, "<interpret> {:#?}", code),
            _ => write!(fmt, "<native>"),
        }
    }
}

#[derive(Clone)]
pub struct Function {
    pub nargs: i32,
    pub kind: FuncKind,
    pub args: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Int(i64),
    Float(u64),
    Bool(bool),
    Str(String),
    Array(usize),
    ObjectRef(usize),
    FuncRef(usize),
    Null,
}

use std::hash::{Hash, Hasher};

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        use self::Value::*;
        match self {
            Int(i) => i.hash(state),
            Float(bits) => bits.hash(state),
            Str(s) => s.hash(state),
            ObjectRef(id) => id.hash(state),
            FuncRef(id) => id.hash(state),
            Bool(b) => b.hash(state),
            Null => {
                let u: u32 = rand::random();
                u.hash(state);
            }
            Array(arr) => arr.hash(state),
        }
    }
}

impl Value {
    pub fn as_f64(&self, _vm: &VirtualMachine) -> f64 {
        match self {
            Value::Float(bits) => f64::from_bits(*bits),
            Value::Int(i) => *i as f64,
            Value::Str(s) => s.parse().unwrap(),
            Value::Null => 0.0,
            _ => unimplemented!(),
        }
    }
    pub fn as_int(&self, _vm: &VirtualMachine) -> i64 {
        match self {
            Value::Float(bits) => f64::from_bits(*bits) as i64,
            Value::Int(i) => *i,
            Value::Str(s) => s.parse().unwrap(),
            Value::Null => 0,
            _ => unimplemented!(),
        }
    }
    pub fn as_str(&self, _vm: &VirtualMachine) -> String {
        match self {
            Value::Str(s) => s.to_string(),
            Value::Int(i) => i.to_string(),
            Value::Float(bits) => f64::from_bits(*bits).to_string(),
            Value::Array(_arr) => format!("array"),
            Value::Null => "null".into(),
            Value::ObjectRef(id) => {
                let obj = _vm.get_object(&id).borrow().clone();
                return format!("{:?}", obj.map);
            }
            Value::Bool(b) => format!("{}", b),
            _ => unimplemented!(),
        }
    }
    pub fn as_object_id(&self) -> usize {
        match self {
            Value::ObjectRef(id) => *id,
            _ => unimplemented!(),
        }
    }

    pub fn as_func_id(&mut self) -> usize {
        match self {
            Value::FuncRef(id) => *id,
            _ => unimplemented!(),
        }
    }
}

use fnv::FnvHashMap;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Object {
    pub name: Option<String>,
    pub map: FnvHashMap<Value, Value>,
}

impl Object {
    pub fn new() -> Object {
        Object {
            name: None,
            map: FnvHashMap::default(),
        }
    }

    pub fn load(&self, key: &Value) -> &Value {
        self.map.get(key).unwrap_or(&Value::Null)
    }

    pub fn store(&mut self, key: Value, obj: Value) {
        self.map.insert(key, obj);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArrayRef {
    pub vec: Ref<Vec<Value>>,
}

impl ArrayRef {
    pub fn new() -> ArrayRef {
        Self {
            vec: Ref::new(RefCell::new(vec![])),
        }
    }
}
