#[derive(Clone,Debug)]
pub enum Opcode {
    PushInt(i64),
    PushFloat(f64),
    PushStr(String),
    PushNull,
    PushBool(bool),
    Pop,

    Amake(usize),
    Call(usize),
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
    Or,
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