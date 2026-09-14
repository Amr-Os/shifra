use rustpython_arabiya::translate;
use shifra::InterpreterBuilderExt;
use shifra::vm::{
    PyObject, PyPayload, PyResult, TryFromBorrowedObject, VirtualMachine, pyclass, pymodule,
};

/// Load a Shifra `.sf` file, translate it to AST-level Python, and execute it
/// into a module registered under `name` so `vm.import(name)` finds it.
fn import_shifra_module(vm: &VirtualMachine, name: &str, sf_path: &str) -> PyResult<()> {
    let source =
        std::fs::read_to_string(sf_path).map_err(|err| vm.new_os_error(err.to_string()))?;
    let translated = translate(&source);

    let scope = vm.new_scope_with_builtins();
    rustpython_arabiya::install_builtins(&scope, vm)?;

    let code = vm
        .compile(&translated, shifra::vm::compiler::Mode::Exec, sf_path)
        .map_err(|err| err.into_pyexception(vm, Some(&source)))?;
    vm.run_code_obj(code, scope.clone())?;

    let dict = scope.globals.clone();
    let module = vm.new_module(name, dict.clone(), None);
    let modules = vm.sys_module.get_attr("modules", vm)?;
    modules.set_item(name, module.into(), vm)?;
    Ok(())
}

pub fn main() {
    let builder = shifra::Interpreter::builder(Default::default());
    let def = rust_py_module::module_def(&builder.ctx);
    let interp = builder.init_stdlib().add_native_module(def).build();

    // `Interpreter::run` (instead of `enter`) calls `finalize`, which flushes
    // the VM's stdout/stderr so Python-level `print`/`اطبع` output is not lost.
    let exit_code = interp.run(|vm| {
        vm.insert_sys_path(vm.new_pyobj("examples"))
            .expect("add path");

        let module_name = "call_between_rust_and_python";

        // Shifra source (`.sf`) lives next to this example; if it is present,
        // translate + compile it. Otherwise fall back to plain Python module.
        let sf_path = "examples/call_between_rust_and_python.sf";
        if std::path::Path::new(sf_path).exists() {
            import_shifra_module(vm, module_name, sf_path).map_err(|err| {
                let mut message = String::new();
                let _ = vm.write_exception(&mut message, &err);
                vm.new_runtime_error(format!("load Shifra module: {message}"))
            })?;
        } else {
            vm.import(module_name, 0)?;
        }

        let module = vm.import(module_name, 0)?;
        let init_fn = module.get_attr("python_callback", vm)?;
        init_fn.call((), vm)?;

        let take_string_fn = module.get_attr("take_string", vm)?;
        take_string_fn.call((String::from("Rust string sent to python"),), vm)?;
        Ok(())
    });
    std::process::exit(exit_code as i32);
}

#[pymodule]
mod rust_py_module {
    use super::*;
    use shifra::vm::{PyObjectRef, convert::ToPyObject};

    #[pyfunction]
    fn rust_function(
        num: i32,
        s: String,
        python_person: PythonPerson,
        _vm: &VirtualMachine,
    ) -> PyResult<RustStruct> {
        println!(
            "Calling standalone rust function from python passing args:
num: {},
string: {},
python_person.name: {}",
            num, s, python_person.name
        );
        Ok(RustStruct {
            numbers: NumVec(vec![1, 2, 3, 4]),
        })
    }

    #[derive(Debug, Clone)]
    struct NumVec(Vec<i32>);

    impl ToPyObject for NumVec {
        fn to_pyobject(self, vm: &VirtualMachine) -> PyObjectRef {
            let list = self.0.into_iter().map(|e| vm.new_pyobj(e)).collect();
            vm.ctx.new_list(list).to_pyobject(vm)
        }
    }

    #[pyattr]
    #[pyclass(module = "rust_py_module", name = "RustStruct")]
    #[derive(Debug, PyPayload)]
    struct RustStruct {
        numbers: NumVec,
    }

    #[pyclass]
    impl RustStruct {
        #[pygetset]
        fn numbers(&self) -> NumVec {
            self.numbers.clone()
        }

        #[pymethod]
        fn print_in_rust_from_python(&self) {
            println!("Calling a rust method from python");
        }
    }

    struct PythonPerson {
        name: String,
    }

    impl<'a> TryFromBorrowedObject<'a> for PythonPerson {
        fn try_from_borrowed_object(vm: &VirtualMachine, obj: &'a PyObject) -> PyResult<Self> {
            let name = obj.get_attr("name", vm)?.try_into_value::<String>(vm)?;
            Ok(Self { name })
        }
    }
}
