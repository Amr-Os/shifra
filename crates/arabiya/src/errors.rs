//! Arabic rendering of uncaught exceptions for Shifra runs.
//!
//! Shifra programs are executed by the RustPython VM, whose exception
//! messages and traceback text are English. This module installs a Shifra
//! `sys.excepthook` (defined in `res/errors_hook.py`) that re-renders
//! uncaught exceptions in Arabic: the traceback header, frame lines, the
//! exception class name (mapped back through [`crate::builtins::BUILTINS`])
//! and common English message fragments.

/// Install the Arabic exception reporter as `sys.excepthook`.
///
/// This must run before a Shifra file is compiled or executed so that both
/// compile-time (syntax) errors and runtime errors are reported in Arabic.
/// It is best-effort: the hook degrades to a minimal textual report (and
/// finally to RustPython's own rendering) if anything goes wrong.
#[cfg(feature = "vm")]
pub fn install_error_hook(vm: &rustpython_vm::VirtualMachine) -> rustpython_vm::PyResult<()> {
    vm.run_simple_string(include_str!("../res/errors_hook.py"))
        .map(|_| ())
}
