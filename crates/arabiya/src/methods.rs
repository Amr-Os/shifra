//! Arabic spellings for object methods (attributes accessed after a dot).
//! Canonical mapping: `SHIFRA.md` (section 5).

/// The set of `(arabic, python)` method name mappings. A name is translated
/// only when it appears as an attribute access — immediately after a `.` —
/// so `قائمة.اخرج()` becomes `list.pop()` while the same word used elsewhere
/// keeps its normal meaning (e.g. `احذف م[0]` remains the `del` statement).
pub const METHODS: &[(&str, &str)] = &[
    // --- list / tuple ---
    ("أضف", "append"),
    ("امسح", "clear"),
    ("انسخ", "copy"),
    ("عد", "count"),
    ("مدد", "extend"),
    ("موقع", "index"),
    ("ادرج", "insert"),
    ("اخرج", "pop"),
    ("احذف", "remove"),
    ("عكس", "reverse"),
    ("رتب", "sort"),
    // --- dict ---
    ("من_مفاتيح", "fromkeys"),
    ("اجلب", "get"),
    ("عناصر", "items"),
    ("مفاتيح", "keys"),
    ("قيم", "values"),
    ("اخرج_آخر", "popitem"),
    ("عين_مبدئي", "setdefault"),
    ("دمج", "update"),
    // --- set ---
    ("ضم", "add"),
    ("فرق", "difference"),
    ("فرق_حدث", "difference_update"),
    ("تجاهل", "discard"),
    ("تقاطع", "intersection"),
    ("تقاطع_حدث", "intersection_update"),
    ("منفصل", "isdisjoint"),
    ("مجموعة_فرعية", "issubset"),
    ("مجموعة_فوق", "issuperset"),
    ("فرق_متماثل", "symmetric_difference"),
    ("فرق_متماثل_حدث", "symmetric_difference_update"),
    ("اتحاد", "union"),
    // --- str ---
    ("كبر", "upper"),
    ("صغر", "lower"),
    ("حرف_الأول", "capitalize"),
    ("طوى", "casefold"),
    ("وسط", "center"),
    ("رمز", "encode"),
    ("ينتهي_ب", "endswith"),
    ("يبدأ_ب", "startswith"),
    ("وسع_الجداول", "expandtabs"),
    ("ابحث", "find"),
    ("ابحث_من_آخر", "rfind"),
    ("موقع_من_آخر", "rindex"),
    ("نسق", "format"),
    ("نسق_من_قاموس", "format_map"),
    ("هل_حروف", "isalpha"),
    ("هل_حروف_ورقام", "isalnum"),
    ("هل_أسكي", "isascii"),
    ("هل_عشري", "isdecimal"),
    ("هل_رقم", "isdigit"),
    ("هل_رقمي", "isnumeric"),
    ("هل_معرف", "isidentifier"),
    ("هل_صغار", "islower"),
    ("هل_كبار", "isupper"),
    ("هل_قابل_للطباعة", "isprintable"),
    ("هل_مسافة", "isspace"),
    ("هل_عناوين", "istitle"),
    ("اربط", "join"),
    ("برر_يسار", "ljust"),
    ("برر_يمين", "rjust"),
    ("جرد", "strip"),
    ("جرد_يسار", "lstrip"),
    ("جرد_يمين", "rstrip"),
    ("جهز_ترجمة", "maketrans"),
    ("اقتسم", "partition"),
    ("اقتسم_من_آخر", "rpartition"),
    ("احذف_بادئة", "removeprefix"),
    ("احذف_لاحقة", "removesuffix"),
    ("استبدل", "replace"),
    ("قسم", "split"),
    ("قسم_من_آخر", "rsplit"),
    ("قسم_الأسطر", "splitlines"),
    ("بدل_الحالة", "swapcase"),
    ("بعناوين", "title"),
    ("ترجم", "translate"),
    ("املأ_اصفار", "zfill"),
    // --- bytes ---
    ("فك_رمز", "decode"),
    ("من_ست_عشري", "fromhex"),
    ("ست_عشري", "hex"),
    // --- dunders ---
    ("__التهيئة__", "__init__"),
];

/// Return the Python spelling for an Arabic method name, or `None`.
#[must_use]
pub fn method(arabic: &str) -> Option<&'static str> {
    METHODS
        .iter()
        .find_map(|&(a, b)| (a == arabic).then_some(b))
}
