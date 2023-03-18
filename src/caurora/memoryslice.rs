use super::{opcodes::OpCode, values::Value};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct MemorySlice {
    memory: Vec<u16>,
    constants: Vec<Value>,
    lines: Vec<u16>,
}

impl MemorySlice {
    pub fn new() -> Self {
        return MemorySlice {
            memory: Vec::<u16>::new(),
            constants: Vec::<Value>::new(),
            lines: Vec::<u16>::new(),  
        };
    }

    pub fn push(&mut self, oc: OpCode) {
        self.memory.push(oc.repr())
    }

    pub fn push_raw(&mut self, oc: u16) {
        self.memory.push(oc)
    }

    pub fn line_end(&mut self) {
        let linelocation:isize = self.memory.len() as isize -1;
        if linelocation < 0 {
            return
        }
        self.lines.push((linelocation) as u16)
    }

    pub fn get_line(&mut self, op_location: u16) -> u16 {
        let mut line = 0;
        for new_line in &self.lines {
            if *new_line <= op_location as u16  {
                line += 1;
            } else {
                break;
            }
        }
        line
    }

    pub fn read_at_ip(&self, index: usize) -> Option<u16> {
        self.memory.get(index).copied()
    }
    
    pub fn replace_at_location(&mut self, index:usize, v: u16) {
        self.memory[index] = v
    }

    pub fn get_constant(&self, index: u16) -> Option<Value>{
        self.constants.get(index as usize).cloned()
    }

    pub fn get_memory_size(&mut self) -> usize {
        self.memory.len()
    }

    pub fn debug(&mut self, name: &str) {
        println!("== {} ==", name);
        println!("--------------------------------");
        println!("");
        println!("Memory:");
        println!("--------------------------------");
        let mut constant = false;
        for i in 0..self.memory.len() {
            if !constant {
                let opcode: OpCode = unsafe { std::mem::transmute(self.memory[i]) }; 
                println!("{:0>4} -- {:#?} -- {:#?}", i, self.memory[i], opcode);
                if opcode == OpCode::Constant || opcode == OpCode::DefineGlobalVar || opcode == OpCode::SetGlobalVar 
                || opcode == OpCode::GetGlobalVar || opcode == OpCode::SetLocalVar || opcode == OpCode::GetLocalVar ||
                opcode == OpCode::Jmp || opcode == OpCode::JmpFalse || opcode == OpCode::JmpTrue || opcode == OpCode::Loop {
                    constant = true;
                }
            }else {
                constant = false;
                let constant_value = self.get_constant(self.memory[i]).or_else(|| Some(Value::Raw)).unwrap();
                println!("{:0>4} -- {:#?} -- {:#?}", i, self.memory[i], constant_value);
            }
        }
        println!("");
        println!("Constants:");
        println!("--------------------------------");
        for i in 0..self.constants.len() {
            println!("{:0>4} -- {:#?}", i, self.constants[i])
        }
        println!("");
    }

    pub fn push_constant(&mut self, op: OpCode,v: Value) {
        let mut index = self.constants.len();
        let mut found = false;
        for i in 0..self.constants.len() {
            if self.constants[i] == v {
                index = i;
                found = true;
                break;
            }
        }
        if !found {
            self.constants.push(v);
        }
        self.push(op);
        self.memory.push(index as u16);
    }
}
