use std::collections::HashMap;

use crate::caurora::values::Object;

use super::{errorlogger, memoryslice::MemorySlice, opcodes::OpCode, values::Value};

pub enum InterpretResult {
    InterpretOk,
    InterpretCompileError,
    InterpretRuntimeError,
}

#[derive(Debug, Clone)]
pub struct VM<'a> {
    pub memory: &'a MemorySlice,
    pub ip: usize,
    pub stack: Vec<Value>,
    pub globals: HashMap<String, Value>,
}

impl VM<'_> {
    fn advance_and_read(&mut self) -> u16 {
        match self.memory.read_at_ip(self.ip) {
            Some(op) => {
                self.ip += 1;
                op
            }
            None => {
                errorlogger::log_error(&format!(
                    "Advance: Invalid instruction pointer, position: {:#?}",
                    self
                ));
                0
            }
        }
    }

    fn get_next_constant(&mut self) -> Value {
        let read_index = self.advance_and_read();
        match self.memory.get_constant(read_index) {
            Some(op) => op,
            None => {
                errorlogger::log_error(&format!(
                    "Constant: Invalid instruction pointer, position: {:#?}",
                    self
                ));
                Value::Number(0.0)
            }
        }
    }

    pub fn interpret(&mut self) -> InterpretResult {
        println!("== Commands ==");
        loop {
            let opcode = unsafe { std::mem::transmute(self.advance_and_read()) };
            match opcode {
                OpCode::Constant => {
                    let value = self.get_next_constant();
                    self.stack.push(value.clone());
                    println!("Setting Constant {:#?}", value);
                }
                OpCode::Negate => {
                    let value = match self.stack.pop().unwrap() {
                        Value::Number(x) => x,
                        _ => panic!("Wrong Stack value for negate {:#?}", opcode),
                    };
                    self.stack.push(Value::Number(-value));
                    println!("Setting Negate {:#?}", -value);
                }
                OpCode::Add => self.binary_op("+"),
                OpCode::Subtract => self.binary_op("-"),
                OpCode::Multiply => self.binary_op("*"),
                OpCode::Divide => self.binary_op("/"),
                OpCode::Nil => self.stack.push(Value::Nil),
                OpCode::True => self.stack.push(Value::Bool(true)),
                OpCode::False => self.stack.push(Value::Bool(false)),
                OpCode::Not => {
                    match self.stack.pop().unwrap() {
                        Value::Bool(x) => match x {
                            true => self.stack.push(Value::Bool(false)),
                            false => self.stack.push(Value::Bool(true)),
                        },
                        Value::Nil => self.stack.push(Value::Bool(true)),
                        _ => panic!("Wrong Stack value for Not operator {:#?}", opcode),
                    };
                }
                OpCode::Equal => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();

                    self.stack.push(Value::Bool(a == b));
                }
                OpCode::Greater => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();

                    self.stack.push(Value::Bool(a > b));
                }
                OpCode::Less => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();

                    self.stack.push(Value::Bool(a < b));
                }
                OpCode::Print => println!("Vm Print ! {:#?}", self.stack.pop().unwrap()),
                OpCode::Pop => {
                    self.stack.pop();
                }
                OpCode::DefineGlobalVar => {
                    let var_name = self.get_next_constant();
                    match var_name {
                        Value::Object(Object::String(var_name)) => {
                            self.globals.insert(var_name, self.stack.pop().unwrap());
                        }
                        _ => panic!("Invalid Identifier name at {:#?}", var_name),
                    }
                }
                OpCode::GetGlobalVar => {
                    let var_name = self.get_next_constant();
                    match var_name {
                        Value::Object(Object::String(var_name)) => {
                            self.stack.push(
                                self.globals
                                    .get(&var_name)
                                    .expect("Identifier not defined !")
                                    .clone(),
                            );
                        }
                        _ => panic!("Invalid Identifier name at {:#?}", var_name),
                    }
                }
                OpCode::SetGlobalVar => {
                    let var_name = self.get_next_constant();
                    match var_name {
                        Value::Object(Object::String(var_name)) => {
                            let key = self.globals.get(&var_name);
                            match key {
                                Some(_) => {self.globals.insert(var_name, self.stack.last().unwrap().clone());},
                                None => panic!("Identifier not defined ! {}", var_name)
                            }
                        }
                        _ => panic!("Invalid Identifier name at {:#?}", var_name),
                    }
                }
                OpCode::GetLocalVar => {
                    let local_location = match self.get_next_constant() {
                        Value::Number(x) => x as usize,
                        _ => panic!("Expected Number pointer for the local variable {:#?}", opcode)
                    };
                    self.stack.push(self.stack[local_location].clone())
                }
                OpCode::SetLocalVar => {
                    let local_location = match self.get_next_constant() {
                        Value::Number(x) => x as usize,
                        _ => panic!("Expected Number pointer for the local variable {:#?}", opcode)
                    };
                    self.stack[local_location] = self.stack.last().unwrap().clone()
                }
                OpCode::JmpFalse => {
                    let steps = self.advance_and_read();
                    match self.stack.last().unwrap().clone() {
                        Value::Bool(b) => {
                            if b == false {
                                self.ip += steps as usize;
                            }
                        }
                        _ => panic!("Top of the stack is not a bool for jmp: {:#?} \n  stack: {:#?}", opcode, self.stack)
                    }
                }
                OpCode::Jmp => {
                    let steps = self.advance_and_read();
                    self.ip += steps as usize;
                }
                OpCode::Return => {
                    println!("Return");
                    break;
                }
                _ => panic!("OpCode not implemented : {:#?}", opcode),
            }
        }
        println!("== Commands End ==");
        InterpretResult::InterpretOk
    }

    fn binary_op(&mut self, op: &str) {
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();

        if std::mem::discriminant(&a) != std::mem::discriminant(&b) {
            panic!(
                "left and right operands of {} not the same left : {:#?}, right {:#?}",
                op, a, b
            );
        }

        match (a.clone(), b.clone()) {
            (Value::Number(x), Value::Number(y)) => match op {
                "+" => self.stack.push(Value::Number(x + y)),
                "-" => self.stack.push(Value::Number(x - y)),
                "*" => self.stack.push(Value::Number(x * y)),
                "/" => self.stack.push(Value::Number(x / y)),
                _ => errorlogger::log_error(&format!("Invalid Binary Operation {:#?}", &self)),
            },
            (Value::Object(Object::String(mut x)), Value::Object(Object::String(y))) => match op {
                "+" => {
                    x.push_str(&y);
                    self.stack.push(Value::Object(Object::String(x)));
                }
                _ => errorlogger::log_error(&format!("Invalid Binary Operation {:#?}", &self)),
            },
            _ => panic!(
                "left and right operands of {} not the same left : {:#?}, right {:#?}",
                op, a, b
            ),
        }
    }

    pub fn debug(&self) {
        println!("");
        println!("");
        println!("== Stack ==");
        println!("--------------------------------");
        for i in 0..self.stack.len() {
            println!("{:0>4} -- {:#?}", i, self.stack[i])
        }
        println!("");
        println!("");
        println!("== Globals ==");
        println!("--------------------------------");
        println!("{:#?}", self.globals);
    }
}
