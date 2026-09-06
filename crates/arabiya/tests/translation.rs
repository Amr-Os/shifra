//! Integration tests for the شِفرة (Shifra) translation pipeline.

#[cfg(test)]
mod tests {
    use rustpython_arabiya::builtins::builtin;
    use rustpython_arabiya::keywords::{is_keyword, keyword};
    use rustpython_arabiya::preprocessor::translate;

    #[test]
    fn keyword_table_pairs_are_sane() {
        // Every known Arabic keyword should translate to a valid Python keyword.
        for arabic in [
            "و",
            "باسم",
            "تحقق",
            "غير_متزامن",
            "انتظر",
            "توقف",
            "حالة",
            "صنف",
            "تابع",
            "عرف",
            "احذف",
            "وإلا_إن",
            "وإلا",
            "التقط",
            "كاذب",
            "ختاما",
            "لكل",
            "من",
            "عام",
            "إن",
            "استورد",
            "ضمن",
            "هو",
            "لامبدا",
            "عدم",
            "غير_محلي",
            "ليس",
            "أو",
            "تجاوز",
            "ارم",
            "أعد",
            "صحيح",
            "جرب",
            "نوع",
            "ما_دام",
            "مع",
            "انتج",
            "طابق",
        ] {
            assert!(is_keyword(arabic), "{arabic} should be a keyword");
            assert!(keyword(arabic).is_some(), "{arabic} should map");
        }
    }

    #[test]
    fn keyword_table_no_unknown() {
        for word in ["xy", "", "לא", "हिन्दी", "déf", "foo", "RLO"] {
            assert!(!is_keyword(word), "{word} should not be a keyword");
        }
    }

    #[test]
    fn builtin_table_pairs_are_sane() {
        for &(arabic, python) in rustpython_arabiya::builtins::BUILTINS {
            assert_eq!(builtin(arabic), Some(python));
            assert!(!arabic.is_empty());
            assert!(!python.is_empty());
        }
    }

    #[test]
    fn python_keyword_round_trip() {
        // A pure-English snippet must survive translation unchanged.
        let source = "def fib(n):\n    if n < 2:\n        return n\n    return fib(n-1) + fib(n-2)";
        assert_eq!(translate(source), source);
    }

    #[test]
    fn full_arabic_program() {
        let source = r##"عرف فيبوناتشي(عدد):
    إن عدد <= ١:
        أعد عدد
    أعد فيبوناتشي(عدد - ١) + فيبوناتشي(عدد - ٢)

لكل i ضمن نطاق(١٠):
    اطبع(فيبوناتشي(i))"##;
        let expected = r##"def فيبوناتشي(عدد):
    if عدد <= ١:
        return عدد
    return فيبوناتشي(عدد - ١) + فيبوناتشي(عدد - ٢)

for i in نطاق(١٠):
    اطبع(فيبوناتشي(i))"##;
        assert_eq!(translate(source), expected);
    }

    #[test]
    fn class_and_method_translation() {
        let source = r#"صنف شخص:
    عرف __init__(الذات, الاسم):
        الذات.الاسم = الاسم
    عرف عرف_النفس(الذات):
        أعد "أنا " + الذات.الاسم"#;
        let expected = r#"class شخص:
    def __init__(الذات, الاسم):
        الذات.الاسم = الاسم
    def عرف_النفس(الذات):
        return "أنا " + الذات.الاسم"#;
        assert_eq!(translate(source), expected);
    }

    #[test]
    fn string_and_comment_preserved_in_program() {
        let source = r#"# يشرح هذا البرنامج الاستثناءات
عرف قسمة(المقسوم, المقسوم_عليه):
    جرب:
        أعد المقسوم / المقسوم_عليه
    التقط خطأ_قيمة:
        اطبع("لا يمكن القسمة على صفر")"#;
        let output = translate(source);
        assert!(output.contains(r#"# يشرح هذا البرنامج الاستثناءات"#));
        assert!(output.contains(r#"لا يمكن القسمة على صفر"#));
        assert!(output.contains("def قسمة(المقسوم, المقسوم_عليه):"));
        assert!(output.contains("try:"));
        assert!(output.contains("except خطأ_قيمة:"));
    }

    #[test]
    fn is_requires_boolean_identifiers() {
        // Arabic identifiers and keywords interleaved in a condition
        let source = "إن نشط هو صحيح:\n    اطبع(\"يعمل\")";
        let expected = "if نشط is True:\n    اطبع(\"يعمل\")";
        assert_eq!(translate(source), expected);
    }

    #[test]
    fn empty_and_whitespace() {
        assert_eq!(translate("\n  \n\t\n"), "\n  \n\t\n");
        assert_eq!(translate("   "), "   ");
    }
}
