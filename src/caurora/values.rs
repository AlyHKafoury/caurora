#[derive(Debug,Clone, PartialEq, PartialOrd)]
pub enum Object {
    String(String),
    Function{
        name: String,
        address: usize,
        arity: usize,
    }
}

#[derive(Debug,Clone, PartialEq, PartialOrd)]
pub enum Value {
    Number(f64),
    Nil,
    Raw,
    Bool(bool),
    Object(Object),
}

