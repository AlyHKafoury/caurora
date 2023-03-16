
#[repr(u16)]
#[derive(Debug,Clone, Copy,PartialEq, PartialOrd)]
pub enum OpCode {
    Add,
    Subtract,
    Multiply,
    Divide,
    Constant,
    Equal,
    Greater,
    Less,
    Nil,
    True,
    False,
    Not,
    Negate,
    Print,
    Pop,
    SetGlobalVar,
    GetGlobalVar,
    DefineGlobalVar,
    SetLocalVar,
    GetLocalVar,
    Jmp,
    JmpTrue,
    JmpFalse,
    Loop,
    Panic,
    Return
}

impl OpCode {
    pub fn repr(&self) -> u16 {
        // SAFETY: Because `Self` is marked `repr(u16)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `u16` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        unsafe { *<*const _>::from(self).cast::<u16>() }
    }
}