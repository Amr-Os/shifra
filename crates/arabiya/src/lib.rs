//! شِفرة (Shifra): an Arabic programming language built on top of RustPython.
//!
//! This crate translates reserved Arabic keywords to their English Python
//! equivalents before compilation. Arabic identifiers, strings, and comments
//! are preserved verbatim.
//!
//! The production implementation belongs in the RustPython Ruff lexer; this
//! crate provides the preprocessor fallback used until then.

pub mod builtins;
pub mod errors;
pub mod keywords;
pub mod methods;
pub mod modules;
pub mod preprocessor;

pub use preprocessor::translate;

#[cfg(feature = "vm")]
pub use builtins::install_builtins;
#[cfg(feature = "vm")]
pub use errors::install_error_hook;
