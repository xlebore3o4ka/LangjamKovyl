extern crate core;

use crate::syntax::lexer::Lexer;
use crate::syntax::parser::Parser;
use crate::vm::module::VMModuleManager;
use crate::vm::{VMRef, VM};

pub mod syntax;
pub mod vm;
pub mod std;
pub mod napi;
pub mod utils;

fn main() {
    let args = ::std::env::args().collect::<Vec<String>>();
    let file =
        if args.len() == 1 {
            println!("Входной файл не указан - запускаем чат (examples/chat.yg)");
            "examples/chat.yg"
        } else {
            &args.get(1).unwrap().parse::<String>().unwrap()
        };
    run_vm(file);
}

pub fn run_vm(file: &str) {
    let mut vm = VM::new().unwrap();
    let mut vm = VMRef::from(&mut vm);

    let input = ::std::fs::read_to_string(file).unwrap();
    let lexer = Lexer::new("test.yg".to_owned(), input);
    let mut parser = Parser::new(lexer);
    let module = parser.parse_module().unwrap();
    VMModuleManager::load_ast_module(vm, &module).unwrap();

    vm.stop(false).unwrap();
}

pub(crate) fn ownership_hack_mut<'a, 'b, T>(value: &'a mut T) -> &'b mut T {
    // SAFETY: Обход владения.
    unsafe { ::std::mem::transmute(value) }
}

pub(crate) fn ownership_hack<'a, 'b, T>(value: &'a T) -> &'b T {
    // SAFETY: Обход владения.
    unsafe { ::std::mem::transmute(value) }
}