//! Arabic spellings for standard library module names and single library /
//! dunder names. Canonical mapping: `SHIFRA.md` (section 8).

/// Arabic module names rewritten only in the module-name position of a
/// `from … import` / `import` statement (`من معطيات_الصنوف استورد …` →
/// `from dataclasses import …`). English module names still pass through.
pub const MODULES: &[(&str, &str)] = &[("معطيات_الصنوف", "dataclasses"), ("تصنيف", "typing")];

/// Library names that are always rewritten, wherever they appear: an import
/// member list, a decorator, a type annotation, or an assignment target
/// (`الكل__ = …` → `__all__ = …`).
pub const NAMES: &[(&str, &str)] = &[
    ("معطيات_الصنف", "dataclass"),
    ("أي_نوع", "Any"),
    ("الكل__", "__all__"),
    ("__الكل__", "__all__"),
    ("الاسم__", "__name__"),
    ("__الاسم__", "__name__"),
    ("__التهيئة__", "__init__"),
];

/// Return the Python module name for an Arabic module name, or `None`.
#[must_use]
pub fn module(arabic: &str) -> Option<&'static str> {
    MODULES
        .iter()
        .find_map(|&(a, b)| (a == arabic).then_some(b))
}

/// Return the Python name for an Arabic library / dunder name, or `None`.
#[must_use]
pub fn name(arabic: &str) -> Option<&'static str> {
    NAMES.iter().find_map(|&(a, b)| (a == arabic).then_some(b))
}
