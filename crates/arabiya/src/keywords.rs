//! Arabic keyword spellings mapped to their English Python equivalents.

/// Return the Python spelling for an Arabic keyword, or `None` if the word is
/// not a known Arabic keyword.
#[must_use]
pub fn keyword(word: &str) -> Option<&'static str> {
    Some(match word {
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

/// Return `true` if `word` is a known Arabic keyword.
#[must_use]
pub fn is_keyword(word: &str) -> bool {
    keyword(word).is_some()
}
