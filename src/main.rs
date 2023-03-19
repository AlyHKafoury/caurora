use std::{env, fs, io, process::exit, collections::HashMap};

use caurora::{memoryslice::MemorySlice, values::{Value}, compiler::Compiler};

use crate::caurora::virtualmachine::VM;

use chrono::prelude::*;

mod caurora;

fn main() {
    const N: usize = 1_000_000;
    let start = Local::now().timestamp() as f64;
    std::thread::Builder::new()
        .stack_size(1024 * N)
        .spawn(||{
    match env::args().len() {
        2 => run_file(env::args().nth(1).unwrap()).unwrap(),
        _ => {
            println!("Usage: aurora [script]");
            exit(1);
        }
    }
    }).unwrap().join().unwrap();
    let end = Local::now().timestamp() as f64;
    println!("Time: {}", end - start);
}

fn run_file(path: String) -> Result<(), io::Error> {
    let mut script = Box::leak(fs::read_to_string(path)?.into_boxed_str());
    run(&script[..]);
    return Ok(());
}

fn run(script: & 'static str) -> () {
    let mut main_memory = MemorySlice::new();

    let mut scanner = caurora::scanner::Scanner::new(script);

    let mut cmplr = Compiler::new(&script, main_memory, scanner);
    main_memory = cmplr.compile();

    //main_memory.debug("Main");
    let mut vm = VM {
        memory: &main_memory,
        ip: 0,
        stack: Vec::<Value>::new(),
        globals: HashMap::<String, Value>::new(),
        ip_stack: Vec::<usize>::new(),
        sp: 0,
        temp_val: Value::Nil,
    };
    vm.interpret();
    // vm.debug();
}
