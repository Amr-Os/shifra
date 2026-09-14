use rustpython_arabiya::translate;
use shifra::vm;
use shifra::vm::PyResult;
use shifra::vm::builtins::PyStrRef;
use shifra::vm::{Interpreter, VirtualMachine};
use std::process::ExitCode;

/// Load a Shifra `.sf` file, translate it to Python, and execute it into a
/// module registered under `name` so `vm.import(name)` finds it.
fn import_shifra_module(vm: &VirtualMachine, name: &str, sf_path: &str) -> PyResult<()> {
    let source =
        std::fs::read_to_string(sf_path).map_err(|err| vm.new_os_error(err.to_string()))?;
    let translated = translate(&source);

    let scope = vm.new_scope_with_builtins();
    rustpython_arabiya::install_builtins(&scope, vm)?;

    let code = vm
        .compile(&translated, vm::compiler::Mode::Exec, sf_path)
        .map_err(|err| err.into_pyexception(vm, Some(&source)))?;
    vm.run_code_obj(code, scope.clone())?;

    let module = vm.new_module(name, scope.globals.clone(), None);
    let modules = vm.sys_module.get_attr("modules", vm)?;
    modules.set_item(name, module.into(), vm)?;
    Ok(())
}

fn py_main(interp: &Interpreter) -> vm::PyResult<PyStrRef> {
    interp.enter(|vm| {
        // Add local library path
        vm.insert_sys_path(vm.new_pyobj("examples"))
            .expect("add examples to sys.path failed");
        // Shifra source (`.sf`) lives next to this example; Shifra packages
        // are loaded by translating the Arabic source, not by `vm.import`.
        import_shifra_module(vm, "package_embed", "examples/package_embed_arabic.sf")
            .expect("load Shifra module");
        let module = vm.import("package_embed", 0)?;
        let name_func = module.get_attr("سياق", vm)?;
        let result = name_func.call((), vm)?;
        let result: PyStrRef = result.get_attr("الاسم", vm)?.try_into_value(vm)?;
        vm::PyResult::Ok(result)
    })
}

fn main() -> ExitCode {
    // Add standard library path
    let mut settings = vm::Settings::default();
    settings.path_list.push("Lib".to_owned());
    let builder = vm::Interpreter::builder(settings);
    let defs = rustpython_stdlib::stdlib_module_defs(&builder.ctx);
    let interp = builder.add_native_modules(&defs).build();
    let result = py_main(&interp);
    let result = result.map(|result| {
        println!("name: {result}");
    });
    vm::host_env::os::exit_code(interp.run(|_vm| result))
}
