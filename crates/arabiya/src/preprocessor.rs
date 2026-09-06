//! A string/comment-aware translator that rewrites Arabic keywords to their
//! English Python equivalents.
//!
//! Only reserved words in code are translated; words inside string literals
//! and comments are preserved verbatim.

use crate::keywords::keyword;

fn is_word_char(c: char) -> bool {
    c == '_' || c.is_alphanumeric()
}

/// Translate an Arabic source string into valid Python source.
///
/// Walks the source character-by-character, translating Arabic keywords to
/// their English equivalents while leaving strings and comments untouched.
#[must_use]
pub fn translate(source: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::translate;

    // --- keyword translation ---

    #[test]
    fn single_keyword() {
        assert_eq!(translate("عرف"), "def");
    }

    #[test]
    fn all_keywords() {
        assert_eq!(translate("و"), "and");
        assert_eq!(translate("باسم"), "as");
        assert_eq!(translate("تحقق"), "assert");
        assert_eq!(translate("غير_متزامن"), "async");
        assert_eq!(translate("انتظر"), "await");
        assert_eq!(translate("توقف"), "break");
        assert_eq!(translate("حالة"), "case");
        assert_eq!(translate("صنف"), "class");
        assert_eq!(translate("تابع"), "continue");
        assert_eq!(translate("عرف"), "def");
        assert_eq!(translate("احذف"), "del");
        assert_eq!(translate("وإلا_إن"), "elif");
        assert_eq!(translate("وإلا"), "else");
        assert_eq!(translate("التقط"), "except");
        assert_eq!(translate("كاذب"), "False");
        assert_eq!(translate("ختاما"), "finally");
        assert_eq!(translate("لكل"), "for");
        assert_eq!(translate("من"), "from");
        assert_eq!(translate("عام"), "global");
        assert_eq!(translate("إن"), "if");
        assert_eq!(translate("استورد"), "import");
        assert_eq!(translate("ضمن"), "in");
        assert_eq!(translate("هو"), "is");
        assert_eq!(translate("لامبدا"), "lambda");
        assert_eq!(translate("عدم"), "None");
        assert_eq!(translate("غير_محلي"), "nonlocal");
        assert_eq!(translate("ليس"), "not");
        assert_eq!(translate("أو"), "or");
        assert_eq!(translate("تجاوز"), "pass");
        assert_eq!(translate("ارم"), "raise");
        assert_eq!(translate("أعد"), "return");
        assert_eq!(translate("صحيح"), "True");
        assert_eq!(translate("جرب"), "try");
        assert_eq!(translate("نوع"), "type");
        assert_eq!(translate("ما_دام"), "while");
        assert_eq!(translate("مع"), "with");
        assert_eq!(translate("انتج"), "yield");
        assert_eq!(translate("طابق"), "match");
    }

    // --- strings are NOT translated ---

    #[test]
    fn string_contents_not_translated() {
        // The word "عرف" inside a string literal is NOT translated
        assert_eq!(translate(r#""عرف""#), r#""عرف""#);
        assert_eq!(translate("'عرف'"), "'عرف'");
    }

    #[test]
    fn comment_not_translated() {
        assert_eq!(translate("# يعرف"), "# يعرف");
        assert_eq!(translate("# إن framework"), "# إن framework");
    }

    #[test]
    fn triple_quoted_string_not_translated() {
        let input = r#""""
عرف من🐂
"""""#;
        let output = translate(input);
        assert!(output.contains("عرف من🐂"));
    }

    // --- mixed content ---

    #[test]
    fn arabic_in_function_def() {
        assert_eq!(
            translate("عرف مجرب():\n    أعد ٤٢"),
            "def مجرب():\n    return ٤٢"
        );
    }

    #[test]
    fn if_else_block() {
        assert_eq!(
            translate("إن صحيح:\n    اطبع('أ')\nوإلا:\n    اطبع('ب')"),
            "if True:\n    اطبع('أ')\nelse:\n    اطبع('ب')"
        );
    }

    // --- identifiers pass through ---

    #[test]
    fn arabic_identifiers_untouched() {
        // Variable names with Arabic characters are valid Unicode identifiers
        // and should not be touched.
        assert_eq!(translate("م = ١٠"), "م = ١٠");
        assert_eq!(translate("فيبوناتشي(١٠)"), "فيبوناتشي(١٠)");
    }

    #[test]
    fn english_keywords_also_work() {
        // The translator should not break English keywords
        assert_eq!(translate("def foo():\n    pass"), "def foo():\n    pass");
    }

    // --- edge cases ---

    #[test]
    fn empty_string() {
        assert_eq!(translate(""), "");
    }

    #[test]
    fn newline_only() {
        assert_eq!(translate("\n\n"), "\n\n");
    }

    #[test]
    fn escape_in_string() {
        assert_eq!(translate(r#""hello\nworld""#), r#""hello\nworld""#);
    }

    #[test]
    fn escaped_quote_in_string() {
        assert_eq!(translate(r#""say \"hi\"""#), r#""say \"hi\"""#);
    }

    #[test]
    fn escaped_single_quote_in_string() {
        assert_eq!(translate(r"'it\'s'"), r"'it\'s'");
    }

    #[test]
    fn adjacent_keywords() {
        assert_eq!(translate("وإلا_إن"), "elif");
        // Ensure compound words are split correctly
        assert_eq!(translate("elif"), "elif");
    }

    // --- more complex programs ---

    #[test]
    fn for_loop() {
        // `نطاق` and `اطبع` are builtins, not keywords: only keywords translate.
        assert_eq!(
            translate("لكل x ضمن نطاق(١٠):\n    اطبع(x)"),
            "for x in نطاق(١٠):\n    اطبع(x)"
        );
    }

    #[test]
    fn try_except_finally() {
        // `خطأ_قيمة` and `اطبع` are builtins (runtime-bound), not keywords.
        assert_eq!(
            translate(
                "جرب:\n    ارم خطأ_قيمة()\nالتقط خطأ_قيمة:\n    اطبع('error')\nختاما:\n    اطبع('done')"
            ),
            "try:\n    raise خطأ_قيمة()\nexcept خطأ_قيمة:\n    اطبع('error')\nfinally:\n    اطبع('done')"
        );
    }

    #[test]
    fn string_with_keyword_like_content() {
        // Even when a string contains Arabic text that looks like keywords,
        // it should not be translated. Builtins (`اطبع`) are also untouched.
        assert_eq!(
            translate(r#"اطبع("البرنامج يعمل بنجاح")"#),
            r#"اطبع("البرنامج يعمل بنجاح")"#
        );
    }

    #[test]
    fn comment_with_english_and_arabic() {
        let input = "# this is a comment with إن在里面";
        let output = translate(input);
        assert_eq!(output, input);
    }
}
