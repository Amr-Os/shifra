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
            "لازمني",
            "انتظر",
            "توقف",
            "حالة",
            "صنف",
            "استمر",
            "عرف",
            "احذف",
            "وإذا",
            "وإلا",
            "التقط",
            "خاطئ",
            "ختاما",
            "لكل",
            "من",
            "عام",
            "إذا",
            "استورد",
            "ضمن",
            "هو",
            "لامبدا",
            "عدم",
            "لامحلي",
            "ليس",
            "أو",
            "تجاوز",
            "ارم",
            "أعد",
            "صحيح",
            "جرب",
            "نوع",
            "طالما",
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
    إذا عدد <= ١:
        أعد عدد
    أعد فيبوناتشي(عدد - ١) + فيبوناتشي(عدد - ٢)

لكل i ضمن نطاق(١٠):
    اطبع(فيبوناتشي(i))"##;
        let expected = r##"def فيبوناتشي(عدد):
    if عدد <= 1:
        return عدد
    return فيبوناتشي(عدد - 1) + فيبوناتشي(عدد - 2)

for i in نطاق(10):
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
        let source = "إذا نشط هو صحيح:\n    اطبع(\"يعمل\")";
        let expected = "if نشط is True:\n    اطبع(\"يعمل\")";
        assert_eq!(translate(source), expected);
    }

    #[test]
    fn empty_and_whitespace() {
        assert_eq!(translate("\n  \n\t\n"), "\n  \n\t\n");
        assert_eq!(translate("   "), "   ");
    }

    #[test]
    fn module_and_name_table_pairs_are_sane() {
        for &(arabic, python) in rustpython_arabiya::modules::MODULES {
            assert_eq!(rustpython_arabiya::modules::module(arabic), Some(python));
            assert!(!arabic.is_empty());
            assert!(!python.is_empty());
        }
        for &(arabic, python) in rustpython_arabiya::modules::NAMES {
            assert_eq!(rustpython_arabiya::modules::name(arabic), Some(python));
            assert!(!arabic.is_empty());
            assert!(!python.is_empty());
        }
    }

    #[test]
    fn shifra_package_module_embeds_like_english() {
        // `package_embed.sf` in either spelling must translate identically.
        let shifra = r##"من معطيات_الصنوف استورد معطيات_الصنف
من تصنيف استورد أي_نوع

الكل__ = ["سياق"]


@معطيات_الصنف
صنف السياق:
    الاسم: نص
    الشيء: أي_نوع


سياق_المخزن = السياق(
    الاسم="test name",
    الشيء=عدم,
)


عرف سياق() -> السياق:
    أعد سياق_المخزن


إذا الاسم__ == "__main__":
    اطبع(سياق().الاسم)"##;
        let python = r##"from dataclasses import dataclass
from typing import Any

__all__ = ["سياق"]


@dataclass
class السياق:
    الاسم: نص
    الشيء: Any


سياق_المخزن = السياق(
    الاسم="test name",
    الشيء=None,
)


def سياق() -> السياق:
    return سياق_المخزن


if __name__ == "__main__":
    اطبع(سياق().الاسم)"##;
        assert_eq!(translate(shifra), python);
    }

    #[test]
    fn fstring_expressions_translated_inline() {
        let source = r##"مهام = [{'الاسم': 'م', 'منتهية': صحيح}]
لكل ق ضمن مهام:
    اطبع(مـ"المهمة :{ق['الاسم']} \ج مكتملة؟: {'نعم' إذا ق['منتهية'] وإلا 'لا'} \ج")"##;
        let expected = r##"مهام = [{'الاسم': 'م', 'منتهية': True}]
for ق in مهام:
    اطبع(f"المهمة :{ق['الاسم']} \n مكتملة؟: {'نعم' if ق['منتهية'] else 'لا'} \n")"##;
        assert_eq!(translate(source), expected);
    }
}
