use std::cell::RefCell;
use std::rc::Rc;

use crate::compiler::context::KaramelCompilerContext;
use crate::file::compute_path_and_read_file;
use crate::{types::Token, vm::interpreter::run_vm};
use crate::parser::*;
use crate::compiler::*;
use crate::syntax::SyntaxParser;
use crate::logger::{CONSOLE_LOGGER};
use crate::error::generate_error_message;

use log;
use crate::types::VmObject;


pub enum ExecutionSource {
    Code(String),
    File(String)
}

pub struct ExecutionParameters {
    pub source: ExecutionSource,
    pub return_opcode: bool,
    pub return_output: bool
}

#[derive(Default)]
pub struct ExecutionStatus {
    pub compiled: bool,
    pub executed: bool,
    pub memory_output: Option<Vec<VmObject>>,
    pub stdout: Option<RefCell<String>>,
    pub stderr: Option<RefCell<String>>,
    pub opcodes: Option<Vec<Token>>
}

fn get_execution_path() -> String {
    match std::env::current_exe() {
        Ok(path) => match path.parent() {
            Some(parent_path) => parent_path.to_str().unwrap().to_string(),
            _ => String::from(".")
        },
        _ => String::from(".")
    }
}

pub fn code_executer(parameters: ExecutionParameters) -> ExecutionStatus {
    let mut status = ExecutionStatus::default();
    match log::set_logger(&CONSOLE_LOGGER) {
        Ok(_) => {
            if cfg!(debug_assertions) {
                log::set_max_level(log::LevelFilter::Debug)
            } else {
                log::set_max_level(log::LevelFilter::Info)
            }
        },
        _ => ()
    };

    let mut context: KaramelCompilerContext = KaramelCompilerContext::new();
    context.script_path = get_execution_path();
    log::debug!("Execution path: {}", context.script_path);

    let data = match parameters.source {
        ExecutionSource::Code(code) => code,
        ExecutionSource::File(filename) => {
            match compute_path_and_read_file(filename, &context) {
                Ok(content) => content,
                Err(error) => {
                    log::error!("Program hata ile sonlandırıldı: {}", error);
                    status.executed = false;
                    return status
                }
            }
        }
    };

    let mut parser = Parser::new(&data);
    match parser.parse() {
        Err(error) => {
            log::error!("{}", generate_error_message(&data, &error));
            return status;
        },
        _ => ()
    };

    let syntax = SyntaxParser::new(parser.tokens().to_vec());
    let ast = match syntax.parse() {
        Ok(ast) => ast,
        Err(error) => {
            log::error!("{}", generate_error_message(&data, &error));
            return status;
        }
    };

    let opcode_compiler = InterpreterCompiler {};
    if parameters.return_output {
        context.stdout = Some(RefCell::new(String::new()));
        context.stderr = Some(RefCell::new(String::new()));
    }

    let execution_status = match opcode_compiler.compile(ast.clone(), &mut context) {
        Ok(_) => unsafe { run_vm(&mut context) },
        Err(message) => {
            log::error!("Program hata ile sonlandırıldı: {}", message);
            return status;
        }
    };

    match execution_status {
        Ok(memory) => {
            status.compiled = true;
            status.executed = true;
            status.memory_output = Some(memory)
        },
        Err(error) => {
            log::error!("Program hata ile sonlandırıldı: {}", error);
            return status;
        }
    };

    log::info!("Program başarıyla çalıştırıldı");
    if parameters.return_opcode {
        status.opcodes = Some(parser.tokens());
    }

    status.stdout = context.stdout;
    status.stderr = context.stderr;

    status
}
