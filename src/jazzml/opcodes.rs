#[derive(Clone, Debug)]
pub enum Opcode {
    PushInt(i64),
    PushFloat(f64),
    PushObject(usize),
    PushFunc(usize),
    PushStr(String),
    PushNull,
    PushBool(bool),
    Pop,

    Amake(usize),
    Aget,
    /// Call field of object
    CallObj(usize),
    /// Call function
    Call(usize),
    /// Tail call
    TailCall(usize),
    Add,
    Sub,
    Div,
    Mul,
    Rem,
    Bxor,
    Bor,
    Band,
    And,
    Nop,
    Or,
    Shr,
    Shl,
    Lt,
    Gt,
    Eq,
    Neq,

    JmpF(usize),
    JmpT(usize),
    Jmp(usize),

    StoreField,
    LoadField,

    LoadGlobal,
    StoreGlobal,
    StoreLocal,
    LoadLocal,
    Ret,
}
