//! Arabic spellings for Python builtin functions, types, and exceptions.

/// The set of `(arabic, python)` builtin name mappings installed at runtime.
pub const BUILTINS: &[(&str, &str)] = &[
    ("اطبع", "print"),
    ("ادخل", "input"),
    ("طول", "len"),
    ("نطاق", "range"),
    ("قائمة", "list"),
    ("قاموس", "dict"),
    ("مجموع", "sum"),
    ("اصغر", "min"),
    ("اكبر", "max"),
    ("عدد", "int"),
    ("نص", "str"),
    ("خطأ_قيمة", "ValueError"),
    ("خطأ_مفتاح", "KeyError"),
];

/// Return the Python name for an Arabic builtin name, or `None`.
#[must_use]
pub fn builtin(arabic: &str) -> Option<&'static str> {
    BUILTINS
        .iter()
        .find_map(|&(a, b)| (a == arabic).then_some(b))
}

/// Install Arabic spellings of Python builtins into `scope`'s globals.
///
/// This makes functions like `اطبع` (`print`) and `طول` (`len`) available to
/// Shifra programs without translation.
#[cfg(feature = "vm")]
pub fn install_builtins(
    scope: &rustpython_vm::scope::Scope,
    vm: &rustpython_vm::VirtualMachine,
) -> rustpython_vm::PyResult<()> {
    for &(arabic, python) in BUILTINS {
        let value = vm.builtins.get_attr(python, vm)?;
        scope.globals.set_item(arabic, value, vm)?;
    }
    Ok(())
}
