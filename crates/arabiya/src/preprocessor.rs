//! A string/comment-aware translator that rewrites Arabic keywords and object
//! methods to their English Python equivalents.
//!
//! Reserved words in code are translated everywhere; method names are
//! translated only when they appear as an attribute access (right after a
//! `.`). Words inside string literals and comments are preserved verbatim.

use crate::keywords::keyword;
use crate::methods::method;
use crate::modules::{module, name};

fn is_word_char(c: char) -> bool {
    c == '_' || c.is_alphanumeric()
}

/// Map an Arabic-Indic (٠-٩) or Persian (۰-۹) digit to its ASCII value.
///
/// Returns `None` for any other character.
fn ascii_digit(c: char) -> Option<char> {
    match c {
        '٠'..='٩' => Some(char::from_u32(c as u32 - '٠' as u32 + 0x30).expect("0x30..=0x39")),
        '۰'..='۹' => Some(char::from_u32(c as u32 - '۰' as u32 + 0x30).expect("0x30..=0x39")),
        _ => None,
    }
}

/// Convert every Arabic-Indic / Persian digit in `s` to its ASCII form.
/// Used for numeric literals (RustPython's lexer requires ASCII digits).
fn translate_arabic_digits(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if let Some(digit) = ascii_digit(c) {
            out.push(digit);
        } else {
            out.push(c);
        }
    }
    out
}

/// Return `true` if `c` is an Arabic-Indic or Persian digit.
fn is_arabic_digit(c: char) -> bool {
    matches!(c, '٠'..='٩' | '۰'..='۹')
}

/// Map a Shifra escape letter (the Arabic first letter after a `\`, e.g. the
/// `ج` in `\ج`) to the Python escape letter it stands for (`n`, giving `\n`).
///
/// Returns `None` for letters without a Shifra spelling; the escape then
/// passes through untranslated.
fn shifra_escape(letter: char) -> Option<char> {
    Some(match letter {
        'ج' => 'n',  // جديد (new line) → \n
        'ط' => 't',  // تبويب (tab) → \t
        'ر' => 'r',  // رجوع (carriage return) → \r
        'م' => '\\', // مائل (backslash) → \\
        'ص' => '0',  // صفر (NUL) → \0
        'س' => 'b',  // سابق (backspace) → \b
        'ن' => 'f',  // نهاية (form feed) → \f
        'ع' => 'v',  // عمودي (vertical tab) → \v
        'ت' => 'a',  // تنبيه (bell) → \a
        _ => return None,
    })
}

/// Return `true` if `c` is a valid string-prefix letter (`b`, `r`, `f`, `u`
/// in either case) used to form prefixed string literals like `r"..."`.
fn is_string_prefix_letter(c: char) -> bool {
    matches!(c, 'b' | 'B' | 'r' | 'R' | 'f' | 'F' | 'u' | 'U')
}

/// Return `true` if the first non-whitespace character at or after `from` is
/// a `:`. Used to tell a statement-level `else:` from a ternary `else value`.
fn followed_by_colon(source: &str, from: usize) -> bool {
    source[from..].chars().find(|c| !c.is_whitespace()) == Some(':')
}

/// The Shifra f-string prefix, `مـ` (م + tatweel), which maps to Python's `f`.
const SHIFRA_F_PREFIX: &str = "مـ";

/// Skip a quoted string literal starting in `content` at `start` (the opening
/// quote). Single or triple quotes are handled, honoring backslash escapes.
/// Returns the index just past the closing quote.
fn skip_quoted(content: &str, start: usize, len: usize) -> usize {
    let quote = content[start..].chars().next().expect("valid UTF-8");
    let triple = content[start..].starts_with("'''") || content[start..].starts_with("\"\"\"");
    let mut i = start + if triple { 3 } else { 1 };
    while i < len {
        let c = content[i..].chars().next().expect("valid UTF-8");
        if c == '\\' && i + 1 < len {
            i += 1 + content[i + 1..].chars().next().expect("valid").len_utf8();
            continue;
        }
        if triple {
            if content[i..].starts_with(if quote == '\'' { "'''" } else { "\"\"\"" }) {
                return i + 3;
            }
        } else if c == quote {
            return i + 1;
        }
        i += c.len_utf8();
    }
    len
}

/// Copy a raw character from f-string `content` at position `i`, applying
/// Shifra escapes in non-raw literals. Advances `i` in place.
fn push_fstring_char(content: &str, len: usize, raw: bool, out: &mut String, i: &mut usize) {
    let ch = content[*i..].chars().next().expect("valid UTF-8");
    if ch == '\\' && *i + 1 < len {
        let escaped = content[*i + 1..].chars().next().expect("valid UTF-8");
        out.push('\\');
        if raw {
            out.push(escaped);
        } else if let Some(py) = shifra_escape(escaped) {
            out.push(py);
        } else {
            out.push(escaped);
        }
        *i += 1 + escaped.len_utf8();
        return;
    }
    out.push(ch);
    *i += ch.len_utf8();
}

/// Translate the *content* of an f-string (the text between its opening and
/// closing quotes). Literal runs are preserved verbatim (Shifra escapes are
/// applied); each `{…}` expression region is translated with the full Shifra
/// word rules so Arabic keywords inside an f-string work
/// (`…{ق['منتهية'] إذا … وإلا …}…`).
fn translate_fstring_content(content: &str, len: usize, raw: bool, out: &mut String) {
    let mut i = 0;
    while i < len {
        let ch = content[i..].chars().next().expect("valid UTF-8");
        if ch == '{' && !next_is_double_brace(content, len, i) {
            // Find the matching `}` for this expression, honoring nested
            // braces and nested string literals.
            let mut depth = 1usize;
            let mut j = i + ch.len_utf8();
            while j < len {
                let cj = content[j..].chars().next().expect("valid UTF-8");
                if cj == '\\' && j + 1 < len {
                    j += 1 + content[j + 1..].chars().next().expect("valid").len_utf8();
                    continue;
                }
                if cj == '\'' || cj == '"' {
                    j = skip_quoted(content, j, len);
                    continue;
                }
                if cj == '{' {
                    depth += 1;
                } else if cj == '}' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                j += cj.len_utf8();
            }
            let expr = &content[i + ch.len_utf8()..j];
            out.push('{');
            out.push_str(&translate(expr));
            out.push('}');
            i = if j < len {
                j + content[j..].chars().next().expect("valid").len_utf8()
            } else {
                j
            };
            continue;
        }
        if ch == '{' || ch == '}' {
            // `{{` / `}}` are literal brace escapes in f-strings: emit one.
            out.push(ch);
            if i + ch.len_utf8() < len && content[i + ch.len_utf8()..].starts_with(ch) {
                i += ch.len_utf8();
            }
            i += ch.len_utf8();
            continue;
        }
        push_fstring_char(content, len, raw, out, &mut i);
    }
}

/// Is the f-string at `content[i]` a doubled `{{` (a literal-brace escape)?
fn next_is_double_brace(content: &str, _len: usize, i: usize) -> bool {
    content[i..].starts_with("{{")
}

/// Find the closing quote of the string literal opened at `pos` in `source`.
/// Returns the index of the opening quote of the closing delimiter.
fn find_closing_quote(source: &str, pos: usize, quote: char, triple: bool) -> usize {
    let mut i = pos;
    while i < source.len() {
        let c = source[i..].chars().next().expect("valid UTF-8");
        if c == '\\' && i + 1 < source.len() {
            i += 1 + source[i + 1..].chars().next().expect("valid").len_utf8();
            continue;
        }
        if triple {
            if source[i..].starts_with(if quote == '\'' { "'''" } else { "\"\"\"" }) {
                return i;
            }
        } else if c == quote {
            return i;
        }
        i += c.len_utf8();
    }
    source.len()
}

/// Consume a full string literal whose opening quote is at `index` in
/// `source`, appending the translated literal to `out`. Returns the index just
/// past the closing quote.
///
/// Determines f-string (Latin `f`/`F` or Shifra `مـ`) and raw (`r`/`R`) status
/// from the characters immediately before the quote. Plain strings are
/// preserved verbatim (Shifra escapes applied); f-strings additionally
/// translate every `{…}` expression.
fn consume_quoted(source: &str, index: usize, out: &mut String) -> usize {
    let rest = &source[index..];
    let quote = rest.chars().next().expect("string must open with a quote");
    let triple = rest.starts_with("'''") || rest.starts_with("\"\"\"");
    let quote_len = if triple { 3 } else { 1 };
    let closing = find_closing_quote(source, index + quote_len, quote, triple);

    // Inspect the prefix letters just before the opening quote.
    let mut is_f = false;
    let mut is_raw = false;
    let mut j = index;
    while j > 0 {
        let p = source[..j].chars().next_back().expect("valid UTF-8");
        j -= p.len_utf8();
        if !is_string_prefix_letter(p) {
            break;
        }
        if p == 'f' || p == 'F' {
            is_f = true;
        }
        if p == 'r' || p == 'R' {
            is_raw = true;
        }
    }
    // A Latin prefix must not be the tail of an identifier (`bar"…"`). `j`
    // points at the first character that is not a prefix letter.
    let not_identifier = j == index || !is_word_char(source[j..].chars().next().expect("valid"));
    let latin_ok = !(is_f || is_raw) || not_identifier;
    // Shifra `مـ` f-prefix immediately before the quote.
    let shifra_f = index >= SHIFRA_F_PREFIX.len() && source[..index].ends_with(SHIFRA_F_PREFIX);
    let fstring = (is_f && latin_ok) || shifra_f;

    out.push_str(&source[index..index + quote_len]);
    if fstring {
        translate_fstring_content(
            &source[index + quote_len..closing],
            closing - index - quote_len,
            is_raw,
            out,
        );
    } else {
        let mut i = index + quote_len;
        while i < closing {
            push_fstring_char(source, closing, is_raw, out, &mut i);
        }
    }
    if closing < source.len() {
        out.push_str(&source[closing..closing + quote_len]);
    }
    closing + quote_len
}

/// Translate an Arabic source string into valid Python source.
///
/// Walks the source character-by-character, translating Arabic keywords to
/// their English equivalents while leaving strings and comments untouched.
#[must_use]
pub fn translate(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let mut index = 0;
    // Line state used to split one-line `إذا…وإلا` (a Python `if/else` can
    // never share a physical line) so the `else`/`elif` starts a new line at
    // the same indentation.
    let mut line_has_code = false;
    let mut line_indent = String::new();
    // Pending `from …` / `import …` module-name positions: the next bare word
    // after `من` (from) or `استورد` (import) is a module name and is looked up
    // in the module table (`معطيات_الصنوف` → `dataclasses`).
    let mut expect_from_module = false;
    let mut expect_import_module = false;

    while index < source.len() {
        let rest = &source[index..];
        let c = rest.chars().next().expect("index is a character boundary");

        if c == '#' {
            let end = rest
                .find('\n')
                .map_or(source.len(), |offset| index + offset);
            output.push_str(&source[index..end]);
            line_has_code = true;
            index = end;
            continue;
        }

        // A string literal carrying a Shifra `مـ` (f-string) prefix is handled
        // atomically here, before the generic word handler would otherwise
        // swallow the `م`.
        if c == 'م'
            && rest.starts_with(SHIFRA_F_PREFIX)
            && source[index + SHIFRA_F_PREFIX.len()..]
                .chars()
                .next()
                .is_some_and(|q| q == '\'' || q == '"')
        {
            index += SHIFRA_F_PREFIX.len();
            output.push('f');
            index = consume_quoted(source, index, &mut output);
            line_has_code = true;
            continue;
        }

        if matches!(c, '\'' | '"') {
            index = consume_quoted(source, index, &mut output);
            line_has_code = true;
            continue;
        }

        if is_word_char(c) {
            let start = index;
            let is_f_prefix = (source[index..].starts_with('f')
                || source[index..].starts_with('F'))
                && index + 1 < source.len()
                && source[index + 1..]
                    .chars()
                    .next()
                    .is_some_and(|q| q == '\'' || q == '"');
            while index < source.len() {
                let current = source[index..].chars().next().expect("valid UTF-8");
                if !is_word_char(current) {
                    break;
                }
                index += current.len_utf8();
            }
            let word = &source[start..index];
            // A Latin `f` immediately before a quote is an f-string prefix and
            // must be emitted before the (f-string-aware) quote handler. It is
            // not a real identifier word.
            if is_f_prefix && word == "f" {
                output.push('f');
                index = consume_quoted(source, index, &mut output);
                line_has_code = true;
                continue;
            }
            // A word that directly follows a `.` is an attribute/method access
            // and is translated from the methods table (e.g. `.اخرج` → `.pop`).
            let preceded_by_dot = start > 0 && source.as_bytes()[start - 1] == b'.';
            let matched_keyword = keyword(word);
            let replacement = if preceded_by_dot {
                method(word).unwrap_or(word)
            } else {
                // A bare word in the module-name position of a `from … import`
                // / `import` clause is looked up in the module table. Other
                // bare words fall back to library / dunder names so imports,
                // decorators, annotations, and assignment targets all agree
                // (`معطيات_الصنف` → `dataclass`, `الاسم__` → `__name__`).
                let module_name = if expect_from_module || expect_import_module {
                    module(word)
                } else {
                    None
                };
                match matched_keyword {
                    Some(python) => python,
                    None => module_name.or_else(|| name(word)).unwrap_or(word),
                }
            };
            // A bare word that is not itself a keyword consumes a pending module
            // name expectation; `from` / `import` keywords start a new one.
            if !preceded_by_dot && matched_keyword.is_none() {
                expect_from_module = false;
                expect_import_module = false;
            }
            if !preceded_by_dot && replacement == "from" {
                expect_from_module = true;
            }
            if !preceded_by_dot && replacement == "import" {
                expect_import_module = true;
            }
            // A mid-line `elif` (`وإذا`) always starts a new suite, and a
            // mid-line statement `else:` (`وإلا` followed by `:`) must too,
            // because Python rejects them after a finished simple suite on the
            // same physical line. A ternary `else` (`… value`) stays inline.
            let statement_else = replacement == "elif"
                || (replacement == "else" && followed_by_colon(source, index));
            if !preceded_by_dot && line_has_code && statement_else {
                // Drop any trailing whitespace so the split line is clean.
                while output.ends_with(char::is_whitespace) && !output.ends_with('\n') {
                    output.pop();
                }
                output.push('\n');
                output.push_str(&line_indent);
            }
            // Numeric literals written with Arabic-Indic / Persian digits are
            // rewritten to ASCII so the RustPython lexer accepts them
            // (`٤٢` → `42`, `٣.١` → `3.1`). Identifiers keep their digits.
            let out = if replacement.starts_with(is_arabic_digit) {
                translate_arabic_digits(replacement)
            } else {
                replacement.to_owned()
            };
            output.push_str(&out);
            line_has_code = true;
            continue;
        }

        if c == '\n' {
            output.push(c);
            index += c.len_utf8();
            line_has_code = false;
            line_indent.clear();
            expect_from_module = false;
            expect_import_module = false;
            continue;
        }

        if c.is_whitespace() {
            if !line_has_code {
                line_indent.push(c);
            }
            output.push(c);
            index += c.len_utf8();
            continue;
        }

        output.push(c);
        index += c.len_utf8();
        line_has_code = true;
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
        assert_eq!(translate("لازمني"), "async");
        assert_eq!(translate("انتظر"), "await");
        assert_eq!(translate("توقف"), "break");
        assert_eq!(translate("حالة"), "case");
        assert_eq!(translate("صنف"), "class");
        assert_eq!(translate("استمر"), "continue");
        assert_eq!(translate("عرف"), "def");
        assert_eq!(translate("احذف"), "del");
        assert_eq!(translate("وإذا"), "elif");
        assert_eq!(translate("وإلا"), "else");
        assert_eq!(translate("التقط"), "except");
        assert_eq!(translate("خاطئ"), "False");
        assert_eq!(translate("ختاما"), "finally");
        assert_eq!(translate("لكل"), "for");
        assert_eq!(translate("من"), "from");
        assert_eq!(translate("عام"), "global");
        assert_eq!(translate("إذا"), "if");
        assert_eq!(translate("استورد"), "import");
        assert_eq!(translate("ضمن"), "in");
        assert_eq!(translate("هو"), "is");
        assert_eq!(translate("لامبدا"), "lambda");
        assert_eq!(translate("عدم"), "None");
        assert_eq!(translate("لامحلي"), "nonlocal");
        assert_eq!(translate("ليس"), "not");
        assert_eq!(translate("أو"), "or");
        assert_eq!(translate("تجاوز"), "pass");
        assert_eq!(translate("ارم"), "raise");
        assert_eq!(translate("أعد"), "return");
        assert_eq!(translate("صحيح"), "True");
        assert_eq!(translate("جرب"), "try");
        assert_eq!(translate("نوع"), "type");
        assert_eq!(translate("طالما"), "while");
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
        assert_eq!(translate("# إذا framework"), "# إذا framework");
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
            "def مجرب():\n    return 42"
        );
    }

    #[test]
    fn if_else_block() {
        assert_eq!(
            translate("إذا صحيح:\n    اطبع('أ')\nوإلا:\n    اطبع('ب')"),
            "if True:\n    اطبع('أ')\nelse:\n    اطبع('ب')"
        );
    }

    // --- identifiers pass through ---

    #[test]
    fn arabic_identifiers_untouched() {
        // Variable names with Arabic characters are valid Unicode identifiers
        // and should not be touched.
        assert_eq!(translate("م = ١٠"), "م = 10");
        assert_eq!(translate("فيبوناتشي(١٠)"), "فيبوناتشي(10)");
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
        assert_eq!(translate("وإذا"), "elif");
        // Ensure compound words are split correctly
        assert_eq!(translate("elif"), "elif");
    }

    // --- more complex programs ---

    #[test]
    fn for_loop() {
        // `نطاق` and `اطبع` are builtins, not keywords: only keywords translate.
        assert_eq!(
            translate("لكل x ضمن نطاق(١٠):\n    اطبع(x)"),
            "for x in نطاق(10):\n    اطبع(x)"
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

    // --- method translation (words right after a dot) ---

    #[test]
    fn list_method_append() {
        assert_eq!(translate("م.أضف(١)"), "م.append(1)");
    }

    #[test]
    fn list_methods_pop_remove_sort() {
        assert_eq!(translate("م.اخرج()"), "م.pop()");
        assert_eq!(translate("م.احذف(س)"), "م.remove(س)");
        assert_eq!(translate("م.رتب()"), "م.sort()");
    }

    #[test]
    fn dict_methods() {
        assert_eq!(translate("ق.اجلب('مفتاح')"), "ق.get('مفتاح')");
        assert_eq!(translate("مفاتيح = ق.مفاتيح()"), "مفاتيح = ق.keys()");
    }

    #[test]
    fn string_methods() {
        assert_eq!(translate("ن.قسم('،')"), "ن.split('،')");
        assert_eq!(translate("ن.كبر()"), "ن.upper()");
        assert_eq!(translate("ن.صغر()"), "ن.lower()");
    }

    #[test]
    fn method_word_keyword_shadowing() {
        // `احذف` after a dot is `remove`; bare `احذف` is the `del` keyword.
        assert_eq!(translate("احذف م[0]"), "del م[0]");
        assert_eq!(translate("م.احذف(0)"), "م.remove(0)");
    }

    #[test]
    fn float_dot_is_not_a_method() {
        assert_eq!(translate("ن = 3.14"), "ن = 3.14");
        assert_eq!(translate("أ.04"), "أ.04");
    }

    #[test]
    fn english_methods_pass_through() {
        assert_eq!(translate("م.append(س).pop()"), "م.append(س).pop()");
    }

    #[test]
    fn methods_not_translated_inside_strings() {
        assert_eq!(translate(r#"اطبع("م.أضف")"#), r#"اطبع("م.أضف")"#);
    }

    // --- Arabic digit literals are converted to ASCII ---

    #[test]
    fn arabic_digit_integer() {
        assert_eq!(translate("ن = ٤٢"), "ن = 42");
        assert_eq!(translate("نطاق(١٠)"), "نطاق(10)");
    }

    #[test]
    fn arabic_digit_float() {
        assert_eq!(translate("٣.١"), "3.1");
        assert_eq!(translate("٠.٥"), "0.5");
    }

    #[test]
    fn persian_digits() {
        assert_eq!(translate("ن = ۴۲"), "ن = 42");
    }

    #[test]
    fn digits_in_identifiers_kept() {
        // Identifiers starting with a letter keep their Arabic digits.
        assert_eq!(translate("درجة٣ = ٥"), "درجة٣ = 5");
    }

    #[test]
    fn digits_in_strings_and_comments_kept() {
        assert_eq!(translate(r#"اطبع("٤٢")"#), r#"اطبع("٤٢")"#);
        assert_eq!(translate("# الرقم ٤٢"), "# الرقم ٤٢");
    }

    // --- one-line if/else statement ---

    #[test]
    fn one_line_if_else_splits() {
        // A statement `else:` after a finished simple suite must move to a new
        // line at the same indentation.
        assert_eq!(
            translate("إذا ن > ٣: اطبع(\"كبير\") وإلا: اطبع(\"صغير\")"),
            "if ن > 3: اطبع(\"كبير\")\nelse: اطبع(\"صغير\")"
        );
    }

    #[test]
    fn one_line_elif_chain_splits() {
        assert_eq!(
            translate("إذا أ: ف(أ) وإذا ب: ف(ب) وإلا: ف(ج)"),
            "if أ: ف(أ)\nelif ب: ف(ب)\nelse: ف(ج)"
        );
    }

    #[test]
    fn nested_one_line_if_else_keeps_indent() {
        assert_eq!(
            translate("لكل ن ضمن نطاق(٥):\n    إذا ن == ٢: اطبع(\"نعم\") وإلا: اطبع(\"لا\")"),
            "for ن in نطاق(5):\n    if ن == 2: اطبع(\"نعم\")\n    else: اطبع(\"لا\")"
        );
    }

    #[test]
    fn ternary_else_stays_inline() {
        // `وإلا` without a following `:` is a ternary `else`, not a statement.
        assert_eq!(
            translate("طابع = \"موجب\" إذا ن >= ٠ وإلا \"سالب\""),
            "طابع = \"موجب\" if ن >= 0 else \"سالب\""
        );
    }

    #[test]
    fn indented_block_else_unchanged() {
        // A normal indented `if/else` block is untouched (else already starts
        // its own line).
        assert_eq!(
            translate("إذا صحيح:\n    اطبع('أ')\nوإلا:\n    اطبع('ب')"),
            "if True:\n    اطبع('أ')\nelse:\n    اطبع('ب')"
        );
    }

    // --- Shifra escape sequences (inside strings) ---

    #[test]
    fn shifra_escape_newline() {
        assert_eq!(translate(r#"اطبع("سطر1\جسطر2")"#), r#"اطبع("سطر1\nسطر2")"#);
    }

    #[test]
    fn shifra_escape_tab() {
        assert_eq!(translate(r#"اطبع("م\طفصول")"#), r#"اطبع("م\tفصول")"#);
    }

    #[test]
    fn shifra_escape_backslash() {
        // `\م` → `\\` (a literal backslash in the generated Python).
        assert_eq!(translate(r#"اطبع("مسار\ملخط")"#), r#"اطبع("مسار\\لخط")"#);
    }

    #[test]
    fn shifra_escape_all_ctrl() {
        assert_eq!(translate(r#""رجوع\ر""#), r#""رجوع\r""#);
        assert_eq!(translate(r#""صفر\ص""#), r#""صفر\0""#);
        assert_eq!(translate(r#""سابق\س""#), r#""سابق\b""#);
        assert_eq!(translate(r#""نهاية\ن""#), r#""نهاية\f""#);
        assert_eq!(translate(r#""عمودي\ع""#), r#""عمودي\v""#);
        assert_eq!(translate(r#""تنبيه\ت""#), r#""تنبيه\a""#);
    }

    #[test]
    fn shifra_escape_unknown_letter_passes_through() {
        // An unlisted Shifra letter (here for `\x`-like use) is preserved.
        assert_eq!(translate(r#""abc\x41""#), r#""abc\x41""#);
    }

    #[test]
    fn shifra_escape_in_raw_string_kept() {
        // Raw strings must not apply Shifra escapes.
        assert_eq!(translate(r#"اطبع(r"سطر\جسطر")"#), r#"اطبع(r"سطر\جسطر")"#);
    }

    #[test]
    fn escaped_quote_and_backslash_still_work() {
        assert_eq!(translate(r#""say \"hi\"""#), r#""say \"hi\"""#);
        assert_eq!(translate(r"wo \' processor"), r"wo \' processor");
    }

    // --- f-strings (`مـ` or Latin `f` prefixes) ---

    #[test]
    fn fstring_shifra_prefix_maps_to_f() {
        assert_eq!(
            translate("اطبع(مـ\"المجموع = {ن + ١}\")"),
            "اطبع(f\"المجموع = {ن + 1}\")"
        );
    }

    #[test]
    fn fstring_latin_f_translates_expressions() {
        assert_eq!(translate("x=f\"{ن}\""), "x=f\"{ن}\"");
        assert_eq!(translate("x=F\"{ن}\""), "x=F\"{ن}\"");
        // A quoted string inside the expression is preserved verbatim.
        assert_eq!(
            translate("x=f\"{'إن' and 'نعم'}\""),
            "x=f\"{'إن' and 'نعم'}\""
        );
        // A bare `إذا` inside the expression is translated like any keyword.
        assert_eq!(translate("x=f\"{إذا X}\""), "x=f\"{if X}\"");
    }

    #[test]
    fn fstring_expression_translates_ternary() {
        assert_eq!(
            translate("z = مـ\"{'نعم' إذا ق['منتهية'] وإلا 'لا'}\""),
            "z = f\"{'نعم' if ق['منتهية'] else 'لا'}\""
        );
        assert_eq!(
            translate(r#"اطبع(f"المهمة :{ق['الاسم']} \ج {ن}")"#),
            r#"اطبع(f"المهمة :{ق['الاسم']} \n {ن}")"#
        );
    }

    #[test]
    fn fstring_literal_braces_and_text_preserved() {
        // `{{`/`}}` are f-string literal brace escapes; one is emitted.
        assert_eq!(translate("مـ\"{{قيمة}}\""), "f\"{قيمة}\"");
        assert_eq!(translate("مـ\"إن هذا نص إن\""), "f\"إن هذا نص إن\"");
    }

    #[test]
    fn fstring_latin_prefix_not_part_of_identifier() {
        // `ref"…"` ends in a prefix letter but is an identifier, not an f-string.
        assert_eq!(translate(r#"x = ref"hi""#), r#"x = ref"hi""#);
        assert_eq!(translate(r#"x = ter"hi""#), r#"x = ter"hi""#);
    }

    // --- module names and library / dunder names ---

    #[test]
    fn from_import_module_and_member() {
        assert_eq!(
            translate("من معطيات_الصنوف استورد معطيات_الصنف"),
            "from dataclasses import dataclass"
        );
        assert_eq!(
            translate("من تصنيف استورد أي_نوع"),
            "from typing import Any"
        );
    }

    #[test]
    fn english_module_names_still_pass_through() {
        // Existing English spellings keep working untouched.
        assert_eq!(
            translate("من dataclasses استورد dataclass"),
            "from dataclasses import dataclass"
        );
        assert_eq!(translate("من typing استورد Any"), "from typing import Any");
    }

    #[test]
    fn plain_import_module() {
        assert_eq!(translate("استورد معطيات_الصنوف"), "import dataclasses");
    }

    #[test]
    fn module_name_only_first_word() {
        // Qualified modules: only the leading name is translated.
        assert_eq!(
            translate("من معطيات_الصنوف.فرعية استورد معطيات_الصنف"),
            "from dataclasses.فرعية import dataclass"
        );
    }

    #[test]
    fn module_expectation_consumed_by_keyword() {
        // `from . import x`: the `.` holds the module position, so the `import`
        // keyword consumes it without translating anything.
        assert_eq!(translate("من . استورد سياق"), "from . import سياق");
    }

    #[test]
    fn module_expectation_resets_at_newline() {
        // The next line's first word must not be treated as a module name.
        assert_eq!(
            translate("من تصنيف استورد أي_نوع\nاطبع('مرحبا')"),
            "from typing import Any\nاطبع('مرحبا')"
        );
    }

    #[test]
    fn library_name_everywhere() {
        assert_eq!(translate("@معطيات_الصنف"), "@dataclass");
        assert_eq!(translate("الغرض: أي_نوع"), "الغرض: Any");
        assert_eq!(translate("عرف create() -> أي_نوع:"), "def create() -> Any:");
    }

    #[test]
    fn dunder_names() {
        assert_eq!(translate("الكل__ = [\"سياق\"]"), "__all__ = [\"سياق\"]");
        assert_eq!(
            translate("إذا الاسم__ == \"__main__\":"),
            "if __name__ == \"__main__\":"
        );
        assert_eq!(
            translate("عرف __التهيئة__(الذات, العدد):"),
            "def __init__(الذات, العدد):"
        );
        assert_eq!(translate("تم.__التهيئة__(العدد)"), "تم.__init__(العدد)");
    }

    #[test]
    fn document_words_in_strings_pass_through() {
        // `الاسم__` inside a string/comment must not become `__name__`.
        assert_eq!(translate(r#""الاسم__""#), r#""الاسم__""#);
        assert_eq!(translate("# الاسم__"), "# الاسم__");
    }
}
