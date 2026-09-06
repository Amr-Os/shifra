//! A bootstrap runner for `.ar` files.
//!
//! The production implementation belongs in the RustPython Ruff lexer. This
//! example is intentionally a small vertical slice: it translates reserved
//! Arabic words only in code, never in strings or comments, before compiling.

use std::{env, fs, process::ExitCode};

use rustpython::{InterpreterBuilder, InterpreterBuilderExt};
use rustpython_vm as vm;

fn keyword(word: &str) -> Option<&'static str> {
    Some(match word {
        // "ادخل"=>"input",
        "و" => "and",
        "باسم" => "as",
        "تحقق" => "assert",
        "غير_متزامن" => "async",
        "انتظر" => "await",
        "توقف" => "break",
        "حالة" => "case",
        "صنف" => "class",
        "تابع" => "continue",
        "عرف" => "def",
        "احذف" => "del",
        "وإلا_إن" => "elif",
        "وإلا" => "else",
        "التقط" => "except",
        "كاذب" => "False",
        "ختاما" => "finally",
        "لكل" => "for",
        "من" => "from",
        "عام" => "global",
        "إن" => "if",
        "استورد" => "import",
        "ضمن" => "in",
        "هو" => "is",
        "لامبدا" => "lambda",
        "عدم" => "None",
        "غير_محلي" => "nonlocal",
        "ليس" => "not",
        "أو" => "or",
        "تجاوز" => "pass",
        "ارم" => "raise",
        "أعد" => "return",
        "صحيح" => "True",
        "جرب" => "try",
        "نوع" => "type",
        "ما_دام" => "while",
        "مع" => "with",
        "انتج" => "yield",
        "طابق" => "match",
        _ => return None,
    })
}

fn is_word_char(c: char) -> bool {
    c == '_' || c.is_alphanumeric()
}

fn translate(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let mut index = 0;

    while index < source.len() {
        let rest = &source[index..];
        let c = rest.chars().next().expect("index is a character boundary");

        if c == '#' {
            let end = rest
                .find('\n')
                .map_or(source.len(), |offset| index + offset);
            output.push_str(&source[index..end]);
            index = end;
            continue;
        }

        if matches!(c, '\'' | '"') {
            let quote = c;
            let triple = rest.starts_with("'''") || rest.starts_with("\"\"\"");
            let quote_len = if triple { 3 } else { 1 };
            output.push_str(&source[index..index + quote_len]);
            index += quote_len;

            while index < source.len() {
                let string_rest = &source[index..];
                if triple && string_rest.starts_with(if quote == '\'' { "'''" } else { "\"\"\"" }) {
                    output.push_str(&string_rest[..3]);
                    index += 3;
                    break;
                }
                let string_char = string_rest.chars().next().expect("valid UTF-8");
                output.push(string_char);
                index += string_char.len_utf8();
                if !triple && string_char == quote {
                    break;
                }
                if string_char == '\\' && index < source.len() {
                    let escaped = source[index..].chars().next().expect("valid UTF-8");
                    output.push(escaped);
                    index += escaped.len_utf8();
                }
            }
            continue;
        }

        if is_word_char(c) {
            let start = index;
            while index < source.len() {
                let current = source[index..].chars().next().expect("valid UTF-8");
                if !is_word_char(current) {
                    break;
                }
                index += current.len_utf8();
            }
            let word = &source[start..index];
            output.push_str(keyword(word).unwrap_or(word));
            continue;
        }

        output.push(c);
        index += c.len_utf8();
    }

    output
}

fn install_arabic_builtins(scope: &vm::scope::Scope, vm: &vm::VirtualMachine) -> vm::PyResult<()> {
    for (arabic, python) in [
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
    ] {
        let value = vm.builtins.get_attr(python, vm)?;
        scope.globals.set_item(arabic, value, vm)?;
    }
    Ok(())
}

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
        install_arabic_builtins(&scope, vm)?;
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
