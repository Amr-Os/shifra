//! A bootstrap runner for `.ar` files.
//!
//! The production implementation belongs in the RustPython Ruff lexer. This
//! example is intentionally a small vertical slice: it translates reserved
//! Arabic words only in code, never in strings or comments, before compiling.

use std::{env, fs, process::ExitCode};

use rustpython::{InterpreterBuilder, InterpreterBuilderExt};
use rustpython_arabiya::preprocessor::translate;
use rustpython_vm as vm;

fn main() -> ExitCode {
    let Some(path) = env::args().nth(1) else {
        eprintln!("usage: cargo run --example arabiya -- <program.ar>");
        return ExitCode::FAILURE;
    };
    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("cannot read {path}: {error}");
            return ExitCode::FAILURE;
        }
    };
    let translated = translate(&source);

    let interpreter = InterpreterBuilder::new().init_stdlib().interpreter();
    match interpreter.enter(|vm| -> vm::PyResult<_> {
        let scope = vm.new_scope_with_builtins();
        rustpython_arabiya::install_builtins(&scope, vm)?;
        let code = vm
            .compile(&translated, vm::compiler::Mode::Exec, &path)
            .map_err(|error| error.into_pyexception(vm, Some(&source)))?;
        let result = vm.run_code_obj(code, scope)?;
        let stdout = vm.sys_module.get_attr("stdout", vm)?;
        vm.call_method(&stdout, "flush", ())?;
        Ok(result)
    }) {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error:?}");
            ExitCode::FAILURE
        }
    }
}
