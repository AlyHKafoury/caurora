#[derive(Debug,Clone, PartialEq, PartialOrd)]
pub enum Object {
    String(String)
}

#[derive(Debug,Clone, PartialEq, PartialOrd)]
pub enum Value {
    Number(f64),
    Nil,
    Bool(bool),
    Object(Object),
}

