//! Arabic spellings for Python builtin functions, types, constants, and
//! exceptions. Canonical mapping: `SHIFRA.md` (section 3 and 4).

/// The set of `(arabic, python)` builtin name mappings installed at runtime.
///
/// `نوع`/`type` is intentionally absent: it is a keyword and is rewritten by
/// the preprocessor (`type` works as both the builtin and the type statement).
pub const BUILTINS: &[(&str, &str)] = &[
    // --- functions & types ---
    ("اطبع", "print"),
    ("ادخل", "input"),
    ("طول", "len"),
    ("نطاق", "range"),
    ("قائمة", "list"),
    ("قاموس", "dict"),
    ("مجموعة", "set"),
    ("متسلسلة", "tuple"),
    ("مجموع", "sum"),
    ("اصغر", "min"),
    ("اكبر", "max"),
    ("عدد", "int"),
    ("كسري", "float"),
    ("مركب", "complex"),
    ("منطقي", "bool"),
    ("نص", "str"),
    ("بايت", "bytes"),
    ("بيتات", "bytearray"),
    ("مطلق", "abs"),
    ("الكل", "all"),
    ("أي", "any"),
    ("كرر", "iter"),
    ("التالي", "next"),
    ("مكرر_غير_متزامن", "aiter"),
    ("التالي_غير_متزامن", "anext"),
    ("ترقيم", "enumerate"),
    ("رشح", "filter"),
    ("طبق", "map"),
    ("دمج", "zip"),
    ("كائن", "object"),
    ("خاصية", "property"),
    ("دالة_صنف", "classmethod"),
    ("دالة_ثابتة", "staticmethod"),
    ("دور", "round"),
    ("قوة", "pow"),
    ("قسمة_باقية", "divmod"),
    ("معكوس", "reversed"),
    ("مرتب", "sorted"),
    ("نسق", "format"),
    ("تمثيل", "repr"),
    ("هوية", "id"),
    ("قيم", "eval"),
    ("نفذ", "exec"),
    ("هاش", "hash"),
    ("مساعدة", "help"),
    ("افتح", "open"),
    ("حرف", "chr"),
    ("رمز_الحرف", "ord"),
    ("ثنائي", "bin"),
    ("ثماني", "oct"),
    ("ست_عشري", "hex"),
    ("أسكي", "ascii"),
    ("شريحة", "slice"),
    ("دليل", "dir"),
    ("متغيرات", "vars"),
    ("عالميات", "globals"),
    ("محليات", "locals"),
    ("اجمع", "compile"),
    ("نقطة_توقف", "breakpoint"),
    ("عرض_الذاكرة", "memoryview"),
    ("علوي", "super"),
    ("اجلب_سمة", "getattr"),
    ("عين_سمة", "setattr"),
    ("احذف_سمة", "delattr"),
    ("هل_له_سمة", "hasattr"),
    ("مثيل", "isinstance"),
    ("مشتق", "issubclass"),
    ("استدعائي", "callable"),
    // --- constants ---
    ("غير_منفذ", "NotImplemented"),
    ("ثلاث_نقاط", "Ellipsis"),
    // --- exceptions ---
    ("خطأ", "Exception"),
    ("خطأ_أساسي", "BaseException"),
    ("خطأ_حسابي", "ArithmeticError"),
    ("خطأ_تحقق", "AssertionError"),
    ("خطأ_سمة", "AttributeError"),
    ("خطأ_نظام_التشغيل", "OSError"),
    ("خطأ_نهاية_الملف", "EOFError"),
    ("خطأ_استيراد", "ImportError"),
    ("حيد", "IndexError"),
    ("خطأ_مفتاح", "KeyError"),
    ("خطأ_بحث", "LookupError"),
    ("خطأ_ذاكرة", "MemoryError"),
    ("خطأ_اسم", "NameError"),
    ("خطأ_غير_منفذ", "NotImplementedError"),
    ("خطأ_فيضان", "OverflowError"),
    ("خطأ_مرجع", "ReferenceError"),
    ("خطأ_وقت_التشغيل", "RuntimeError"),
    ("خطأ_إزاحة", "IndentationError"),
    ("خطأ_صياغة", "SyntaxError"),
    ("خطأ_تبويب", "TabError"),
    ("خطأ_نوع", "TypeError"),
    ("خطأ_قيمة", "ValueError"),
    ("خطأ_القسمة_على_صفر", "ZeroDivisionError"),
    ("خطأ_متغير_محلي", "UnboundLocalError"),
    ("خطأ_مخزن", "BufferError"),
    ("خطأ_استدعاء_ذاتي", "RecursionError"),
    ("خطأ_وحدة_غير_موجودة", "ModuleNotFoundError"),
    ("خطأ_ملف_غير_موجود", "FileNotFoundError"),
    ("خطأ_ملف_موجود", "FileExistsError"),
    ("خطأ_صلاحية", "PermissionError"),
    ("خطأ_اتصال", "ConnectionError"),
    ("خطأ_اتصال_مرفوض", "ConnectionRefusedError"),
    ("خطأ_اتصال_مقاطع", "ConnectionAbortedError"),
    ("خطأ_اتصال_معاد", "ConnectionResetError"),
    ("خطأ_أنبوب_مكسور", "BrokenPipeError"),
    ("خطأ_إدخال_محجوب", "BlockingIOError"),
    ("خطأ_إدخال_إخراج", "IOError"),
    ("خطأ_بيئة", "EnvironmentError"),
    ("خطأ_نقطة_كسرية", "FloatingPointError"),
    ("خطأ_مقاطع", "InterruptedError"),
    ("خطأ_مسار_دليل", "IsADirectoryError"),
    ("خطأ_ليس_دليلا", "NotADirectoryError"),
    ("خطأ_عملية_غير_موجودة", "ProcessLookupError"),
    ("خطأ_عملية_طفل", "ChildProcessError"),
    ("خطأ_يونيكود", "UnicodeError"),
    ("خطأ_فك_يونيكود", "UnicodeDecodeError"),
    ("خطأ_ترميز_يونيكود", "UnicodeEncodeError"),
    ("خطأ_ترجمة_يونيكود", "UnicodeTranslateError"),
    ("خطأ_نظام", "SystemError"),
    ("مجموعة_أخطاء", "ExceptionGroup"),
    ("مجموعة_أخطاء_أساسية", "BaseExceptionGroup"),
    ("توقف_التكرار", "StopIteration"),
    ("توقف_التكرار_غير_متزامن", "StopAsyncIteration"),
    ("خروج_النظام", "SystemExit"),
    ("خروج_المولد", "GeneratorExit"),
    ("انقطاع_لوحة_المفاتيح", "KeyboardInterrupt"),
    ("تحذير", "Warning"),
    ("تحذير_تقادم", "DeprecationWarning"),
    ("تحذير_ترميز", "EncodingWarning"),
    ("تحذير_مستقبلي", "FutureWarning"),
    ("تحذير_استيراد", "ImportWarning"),
    ("تحذير_صياغة", "SyntaxWarning"),
    ("تحذير_وقت_التشغيل", "RuntimeWarning"),
    ("تحذير_مستخدم", "UserWarning"),
    ("تحذير_بايت", "BytesWarning"),
    ("تحذير_موارد", "ResourceWarning"),
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
        if let Ok(value) = vm.builtins.get_attr(python, vm) {
            scope.globals.set_item(arabic, value, vm)?;
        }
    }
    Ok(())
}
