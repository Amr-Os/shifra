<img src="./logo.png" width="125" height="125" align="left" />

# شِفرة · Shifra

**لغة برمجة عربية كاملة مبنية على بايثون ٣** — مترجم بايثون ٣٫١٤ مكتوب بلغة
Rust يقرأ أكوادًا عربية (`عرف` = `def`، `إذا` = `if`، `اطبع` = `print`…) ويحوّلها
إلى بايثون قبل التنفيذ، فتعمل كل دلالات Python 3.14 دون تغيير.

البرنامج التالي بلغة شِفرة:

```shifra
عرف أكبر(أ, ب):
    إذا أ > ب:
        أعد أ
    وإلا:
        أعد ب

طابع = "موجب" إذا ن >= ٠ وإلا "سالب"
```

---

## يتضمّن المشروع

| الجزء | الموقع | ما هو |
| --- | --- | --- |
| المترجم | `crates/` + `src/` | مترجم Python 3.14 مكتوب بـ Rust يقبل ملفات شِفرة (`.sf`، `.ar`، `.شفـ`) |
| طبقة اللغة | `crates/arabiya/` | قاموس المترجم الذي يحوّل النص العربي إلى نصّ بايثون |
| مواصفة اللغة | `SHIFRA.md` | المعجم العربي ↔ بايثون (المصدر المرجعي) |
| محرّر VS Code | `editors/vscode-shifra/` | إضافة تتضمّن تلوين الصياغة ومحرّرًا عربيًا RTL وتشخيصًا حيًا وتشغيل الملفات |
| محرّر Neovim | `editors/nvim-shifra/` | إضافة Neovim مع عرض RTL وصياغة وتشخيص ومقاطع جاهزة |
| الخطوط | `editors/fonts/` | خطّا حاسوب عربيان (Thanyah Sans Term + Shifra Naskh Term) |
| تطبيق أندرويد | `android/` | تطبيق «شِفرة» على الهاتف بدون Gradle، يضم المترجم داخل `libshifra.so` |
| أمثلة GUI | `example_projects/shifra_tk/` | برامج Tkinter مكتوبة بشِفرة (سلحفاة، عدّاد، نافذة ترحيب) |
| التوثيق والعمليات | `README.md`، `CONTRIBUTING.md`، `AGENTS.md`، `scripts/` | تعليمات البناء والاختبار والعمل |

---

## بدء سريع

```bash
# تشغيل ملف بامتداد .sf (أو .ar / .شفـ)
cargo run --release -- prog.sf

# تحقّق من الصياغة دون تنفيذ (تستخدمه المحرّرات للتشخيص)
cargo run --release -- --check prog.sf

# واجهة تفاعلية (REPL)
cargo run --release
```

> ملاحظة: لا تتم ترجمة المدخل القياسي (stdin)؛ يجب تمرير مسار الملف كوسيط.

### المتطلبات

- Rust أحدث نسخة مستقرة (الإصدار المطلوب في `rust-toolchain.toml`)
- لمرور ملفات `.sf` يجب تمريرها كوسيط تشغيل

---

## البناء والاختبار

```bash
# اختبار طبقة اللغة (58 وحدة + 12 اختبار تكامل)
cargo test -p rustpython-arabiya

# اختبار المترجم
cargo test --workspace --exclude rustpython_wasm --exclude rustpython-venvlauncher --exclude rustpython-capi
(cd crates/capi && cargo test)

# اختبارات بايثون
cd extra_tests && pytest -v

# الفحص بالـ linter
cargo clippy --workspace --all-targets --exclude rustpython_wasm --exclude rustpython-venvlauncher --exclude rustpython-capi
```

### تطبيق أندرويد

```bash
VER=0.3.8 ./android/build.sh
```

### إضافة VS Code

```bash
cd editors/vscode-shifra
node_modules/.bin/vsce package -o shifra-language-0.3.8.vsix
code --install-extension shifra-language-0.3.8.vsix
```

---

## الترخيص (Licence)

مشروع شِفرة هو فرع (fork) من
[RustPython](https://github.com/RustPython/RustPython)، ويحافظ على الترخيصين
الأصليين:

- الشيفرة البرمجية مرخّصة بموجب رخصة MIT — انظر [LICENSE](LICENSE).
- الشعار (logo) مرخّص بموجب CC-BY-4.0 — انظر [LICENSE-logo](LICENSE-logo).
- المكتبة القياسية (مجلد `Lib/`) من CPython وتخضع لرخصة PSF — انظر
  `Lib/PSF-LICENSE`.

بإمكانك القراءة عن المساهمة في [CONTRIBUTING.md](CONTRIBUTING.md).

---

# English

**Shifra** is a complete **Arabic programming language** built on top of a fork
of [RustPython](https://github.com/RustPython/RustPython) — a Python **3.14.0**
interpreter written in Rust. Shifra code uses Arabic spellings for Python's
keywords, builtins and methods (`عرف` = `def`, `إذا` = `if`, `اطبع` = `print`,
...). A preprocessor rewrites the Arabic source into ordinary Python before it
reaches the RustPython compiler, so **all Python 3.14 semantics** work
unchanged. Files use the `.sf` (canonical), `.شفـ` or `.ar` (legacy) extensions.

## Quick start

```bash
cargo run --release -- prog.sf   # run a Shifra file
cargo run --release -- --check prog.sf  # syntax check (used by editors)
cargo run --release             # interactive REPL
```

## What's in the repo

| Part | Where | What it is |
| --- | --- | --- |
| Interpreter | `crates/` + `src/` | Fork of RustPython wired to accept Shifra files |
| Language layer | `crates/arabiya/` | Arabic ↔ Python translator (keywords, builtins, methods, modules) |
| Language spec | `SHIFRA.md` | The canonical Arabic ↔ Python dictionary |
| VS Code extension | `editors/vscode-shifra/` | Syntax highlighting, RTL custom editor, live diagnostics, running files |
| Neovim plugin | `editors/nvim-shifra/` | RTL rendering, syntax, diagnostics, snippets |
| Fonts | `editors/fonts/` | Two monospace Arabic terminal fonts |
| Android app | `android/` | Gradle-free app embedding the interpreter as `libshifra.so` |
| Tk GUI demos | `example_projects/shifra_tk/` | Tkinter programs written in Shifra (RTL-aware) |

## Testing

```bash
cargo test -p rustpython-arabiya
cargo test --workspace --exclude rustpython_wasm --exclude rustpython-venvlauncher --exclude rustpython-capi
(cd crates/capi && cargo test)
cd extra_tests && pytest -v
```

## Licenses

Shifra is a fork of [RustPython](https://github.com/RustPython/RustPython) and
keeps the original two licenses:

- **MIT** for the code — see [LICENSE](LICENSE).
- **CC-BY-4.0** for the logo — see [LICENSE-logo](LICENSE-logo).
- The standard library (`Lib/`) is CPython's and is under the PSF license — see
  `Lib/PSF-LICENSE`.