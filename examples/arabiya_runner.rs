//! A bootstrap runner for Shifra (`.sf`, `.شفـ`, `.ar`) files.
//!
//! The production implementation belongs in the RustPython Ruff lexer. This
//! example is intentionally a small vertical slice: it translates reserved
//! Arabic words only in code, never in strings or comments, before compiling.

use std::{env, fs, process::ExitCode};

use rustpython_arabiya::preprocessor::translate;
use rustpython_vm as vm;
use rustpython_vm::AsObject;
use rustpython_vm::object::PyObjectRef;
use shifra::{InterpreterBuilder, InterpreterBuilderExt};

fn main() -> ExitCode {
    let Some(path) = env::args().nth(1) else {
        eprintln!("usage: cargo run --example arabiya -- <program.sf>");
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
    let outcome: Result<(), String> = interpreter.enter(|vm| {
        let run = (|| -> vm::PyResult<()> {
            let scope = vm.new_scope_with_builtins();
            rustpython_arabiya::install_builtins(&scope, vm)?;
            let sys_module = vm.sys_module.to_owned();
            let argv: PyObjectRef = vm
                .ctx
                .new_list(vec![PyObjectRef::from(vm.ctx.new_str(path.clone()))])
                .into();
            sys_module.set_attr("argv", argv, vm)?;
            let code = vm
                .compile(&translated, vm::compiler::Mode::Exec, &path)
                .map_err(|error| error.into_pyexception(vm, Some(&source)))?;
            vm.run_code_obj(code, scope)?;
            let stdout = vm.sys_module.get_attr("stdout", vm)?;
            vm.call_method(&stdout, "flush", ())?;
            Ok(())
        })();
        match run {
            Ok(()) => Ok(()),
            Err(error) => {
                let msg = (|| {
                    let tb = vm.import("traceback", 0).ok()?;
                    let lines = vm
                        .call_method(&tb, "format_exception", (error.clone(),))
                        .ok()?;
                    let list = lines.downcast_ref::<vm::builtins::PyList>()?;
                    Some(
                        list.borrow_vec()
                            .iter()
                            .filter_map(|l| {
                                l.downcast_ref::<vm::builtins::PyStr>()
                                    .map(|s| s.to_string_lossy().into_owned())
                            })
                            .collect::<Vec<_>>()
                            .join(""),
                    )
                })()
                .unwrap_or_else(|| {
                    error
                        .as_object()
                        .repr(vm)
                        .map(|r| r.to_string())
                        .unwrap_or_default()
                });
                Err(msg)
            }
        }
    });
    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("{msg}");
            ExitCode::FAILURE
        }
    }
}
