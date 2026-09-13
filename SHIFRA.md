# شِفرة · Shifra — Arabic ↔ Python dictionary

This file is the **canonical source of truth** for the Shifra language: every Arabic word below is mapped to the Python symbol it stands for. The interpreter (`crates/arabiya`) is generated from this file, so to change the language you edit this file, then ask the agent to re-sync the Rust tables (`keywords.rs`, `builtins.rs`, `methods.rs`) and the editor tokenizer.

Rules that always hold:

- **Keywords** are translated by the compiler before parsing (they shape code structure).
- **Builtins, constants, and exceptions** are installed at runtime as Arabic names (they behave exactly like the Python ones).
- **Objects' methods** are translated when the word appears as an attribute — right after a dot (`.أضف(...)` → `.append(...)`). Method words used anywhere else keep their usual meaning (e.g. `احذف م[1]` is the `del` statement, while `قائمة.احذف(س)` is `list.remove`).
- English still works everywhere; Arabic is an addition, never a break.

---

## 1. Keywords (الكلمات الأساسية)

| Arabic | Python | Notes |
| --- | --- | --- |
| `و` | `and` | |
| `باسم` | `as` | |
| `تحقق` | `assert` | |
| `لازمني` | `async` | |
| `انتظر` | `await` | |
| `توقف` | `break` | |
| `حالة` | `case` | |
| `صنف` | `class` | |
| `استمر` | `continue` | |
| `عرف` | `def` | |
| `احذف` | `del` | statement form |
| `وإذا` | `elif` | |
| `وإلا` | `else` | |
| `التقط` | `except` | |
| `خاطئ` | `False` | |
| `ختاما` | `finally` | |
| `لكل` | `for` | |
| `من` | `from` | |
| `عام` | `global` | |
| `إذا` | `if` | |
| `استورد` | `import` | |
| `ضمن` | `in` | |
| `هو` | `is` | |
| `لامبدا` | `lambda` | |
| `عدم` | `None` | |
| `لامحلي` | `nonlocal` | |
| `ليس` | `not` | |
| `أو` | `or` | |
| `تجاوز` | `pass` | |
| `ارم` | `raise` | |
| `أعد` | `return` | |
| `صحيح` | `True` | |
| `جرب` | `try` | |
| `نوع` | `type` | keyword + builtin |
| `طالما` | `while` | |
| `مع` | `with` | |
| `انتج` | `yield` | |
| `طابق` | `match` | |

## 2. Literals & constants (القيم الثابتة)

| Arabic | Python | Notes |
| --- | --- | --- |
| `صحيح` | `True` | |
| `خاطئ` | `False` | |
| `عدم` | `None` | |
| `غير_منفذ` | `NotImplemented` | |
| `ثلاث_نقاط` | `Ellipsis` | `...` |

## 3. Builtin functions & types (الدوال الأساسية وأنواعها)

| Arabic | Python | Notes |
| --- | --- | --- |
| `اطبع` | `print` | |
| `ادخل` | `input` | |
| `طول` | `len` | |
| `نطاق` | `range` | |
| `قائمة` | `list` | |
| `قاموس` | `dict` | |
| `مجموعة` | `set` | |
| `متسلسلة` | `tuple` | |
| `مجموع` | `sum` | |
| `اصغر` | `min` | |
| `اكبر` | `max` | |
| `عدد` | `int` | |
| `كسري` | `float` | |
| `مركب` | `complex` | |
| `منطقي` | `bool` | |
| `نص` | `str` | |
| `بايت` | `bytes` | |
| `بيتات` | `bytearray` | |
| `مطلق` | `abs` | |
| `الكل` | `all` | |
| `أي` | `any` | |
| `كرر` | `iter` | |
| `التالي` | `next` | |
| `مكرر_غير_متزامن` | `aiter` | |
| `التالي_غير_متزامن` | `anext` | |
| `ترقيم` | `enumerate` | |
| `رشح` | `filter` | |
| `طبق` | `map` | |
| `دمج` | `zip` | |
| `كائن` | `object` | |
| `نوع` | `type` | via keyword |
| `خاصية` | `property` | |
| `دالة_صنف` | `classmethod` | |
| `دالة_ثابتة` | `staticmethod` | |
| `دور` | `round` | |
| `قوة` | `pow` | |
| `قسمة_باقية` | `divmod` | |
| `معكوس` | `reversed` | `reversed(seq)`; note `عكس` is the `.reverse()` method |
| `مرتب` | `sorted` | |
| `نسق` | `format` | |
| `تمثيل` | `repr` | |
| `هوية` | `id` | |
| `قيم` | `eval` | |
| `نفذ` | `exec` | |
| `هاش` | `hash` | |
| `مساعدة` | `help` | |
| `افتح` | `open` | |
| `حرف` | `chr` | |
| `رمز_الحرف` | `ord` | |
| `ثنائي` | `bin` | |
| `ثماني` | `oct` | |
| `ست_عشري` | `hex` | |
| `أسكي` | `ascii` | |
| `شريحة` | `slice` | |
| `دليل` | `dir` | |
| `متغيرات` | `vars` | |
| `عالميات` | `globals` | |
| `محليات` | `locals` | |
| `اجمع` | `compile` | |
| `نقطة_توقف` | `breakpoint` | |
| `عرض_الذاكرة` | `memoryview` | |
| `علوي` | `super` | `super()` |
| `اجلب_سمة` | `getattr` | |
| `عين_سمة` | `setattr` | |
| `احذف_سمة` | `delattr` | |
| `هل_له_سمة` | `hasattr` | |
| `مثيل` | `isinstance` | |
| `مشتق` | `issubclass` | |
| `استدعائي` | `callable` | |

## 4. Exceptions (الأخطاء)

| Arabic | Python | Notes |
| --- | --- | --- |
| `خطأ` | `Exception` | |
| `خطأ_أساسي` | `BaseException` | |
| `خطأ_حسابي` | `ArithmeticError` | |
| `خطأ_تحقق` | `AssertionError` | matches `تحقق` |
| `خطأ_سمة` | `AttributeError` | |
| `خطأ_نظام_التشغيل` | `OSError` | |
| `خطأ_نهاية_الملف` | `EOFError` | |
| `خطأ_استيراد` | `ImportError` | |
| `حيد` | `IndexError` | |
| `خطأ_مفتاح` | `KeyError` | |
| `خطأ_بحث` | `LookupError` | |
| `خطأ_ذاكرة` | `MemoryError` | |
| `خطأ_اسم` | `NameError` | |
| `خطأ_غير_منفذ` | `NotImplementedError` | |
| `خطأ_فيضان` | `OverflowError` | |
| `خطأ_مرجع` | `ReferenceError` | |
| `خطأ_وقت_التشغيل` | `RuntimeError` | |
| `خطأ_إزاحة` | `IndentationError` | |
| `خطأ_صياغة` | `SyntaxError` | |
| `خطأ_تبويب` | `TabError` | |
| `خطأ_نوع` | `TypeError` | |
| `خطأ_قيمة` | `ValueError` | |
| `خطأ_القسمة_على_صفر` | `ZeroDivisionError` | |
| `خطأ_متغير_محلي` | `UnboundLocalError` | |
| `خطأ_مخزن` | `BufferError` | |
| `خطأ_استدعاء_ذاتي` | `RecursionError` | |
| `خطأ_وحدة_غير_موجودة` | `ModuleNotFoundError` | |
| `خطأ_ملف_غير_موجود` | `FileNotFoundError` | |
| `خطأ_ملف_موجود` | `FileExistsError` | |
| `خطأ_صلاحية` | `PermissionError` | |
| `خطأ_اتصال` | `ConnectionError` | |
| `خطأ_اتصال_مرفوض` | `ConnectionRefusedError` | |
| `خطأ_اتصال_مقاطع` | `ConnectionAbortedError` | |
| `خطأ_اتصال_معاد` | `ConnectionResetError` | |
| `خطأ_أنبوب_مكسور` | `BrokenPipeError` | |
| `خطأ_إدخال_محجوب` | `BlockingIOError` | |
| `خطأ_إدخال_إخراج` | `IOError` | alias of OSError |
| `خطأ_بيئة` | `EnvironmentError` | alias of OSError |
| `خطأ_نقطة_كسرية` | `FloatingPointError` | |
| `خطأ_مقاطع` | `InterruptedError` | |
| `خطأ_مسار_دليل` | `IsADirectoryError` | |
| `خطأ_ليس_دليلا` | `NotADirectoryError` | |
| `خطأ_عملية_غير_موجودة` | `ProcessLookupError` | |
| `خطأ_عملية_طفل` | `ChildProcessError` | |
| `خطأ_يونيكود` | `UnicodeError` | |
| `خطأ_فك_يونيكود` | `UnicodeDecodeError` | |
| `خطأ_ترميز_يونيكود` | `UnicodeEncodeError` | |
| `خطأ_ترجمة_يونيكود` | `UnicodeTranslateError` | |
| `خطأ_نظام` | `SystemError` | |
| `مجموعة_أخطاء` | `ExceptionGroup` | |
| `مجموعة_أخطاء_أساسية` | `BaseExceptionGroup` | |
| `توقف_التكرار` | `StopIteration` | |
| `توقف_التكرار_غير_متزامن` | `StopAsyncIteration` | |
| `خروج_النظام` | `SystemExit` | |
| `خروج_المولد` | `GeneratorExit` | |
| `انقطاع_لوحة_المفاتيح` | `KeyboardInterrupt` | |
| `تحذير` | `Warning` | |
| `تحذير_تقادم` | `DeprecationWarning` | |
| `تحذير_ترميز` | `EncodingWarning` | |
| `تحذير_مستقبلي` | `FutureWarning` | |
| `تحذير_استيراد` | `ImportWarning` | |
| `تحذير_صياغة` | `SyntaxWarning` | |
| `تحذير_وقت_التشغيل` | `RuntimeWarning` | |
| `تحذير_مستخدم` | `UserWarning` | |
| `تحذير_بايت` | `BytesWarning` | |
| `تحذير_موارد` | `ResourceWarning` | |

## 5. Object methods (دوال الكائنات)

Written after a dot: `قائمة.أضف(١)` → `قائمة.append(١)`. Names are shared across types where the concept matches (e.g. `احذف` is both `del` and `.remove`).

### `list` / `tuple`

| Arabic | Python | |
| --- | --- | --- |
| `أضف` | `append` (list) | |
| `امسح` | `clear` (list) | |
| `انسخ` | `copy` | |
| `عد` | `count` | |
| `مدد` | `extend` (list) | |
| `موقع` | `index` | |
| `ادرج` | `insert` (list) | |
| `اخرج` | `pop` | |
| `احذف` | `remove` | |
| `عكس` | `reverse` (list) | |
| `رتب` | `sort` (list) | |

### `dict`

| Arabic | Python |
| --- | --- |
| `امسح` | `clear` |
| `انسخ` | `copy` |
| `من_مفاتيح` | `fromkeys` |
| `اجلب` | `get` |
| `عناصر` | `items` |
| `مفاتيح` | `keys` |
| `قيم` | `values` |
| `اخرج` | `pop` |
| `اخرج_آخر` | `popitem` |
| `عين_مبدئي` | `setdefault` |
| `دمج` | `update` |

### `set`

| Arabic | Python |
| --- | --- |
| `ضم` | `add` |
| `امسح` | `clear` |
| `انسخ` | `copy` |
| `فرق` | `difference` |
| `فرق_حدث` | `difference_update` |
| `تجاهل` | `discard` |
| `تقاطع` | `intersection` |
| `تقاطع_حدث` | `intersection_update` |
| `منفصل` | `isdisjoint` |
| `مجموعة_فرعية` | `issubset` |
| `مجموعة_فوق` | `issuperset` |
| `اخرج` | `pop` |
| `احذف` | `remove` |
| `فرق_متماثل` | `symmetric_difference` |
| `فرق_متماثل_حدث` | `symmetric_difference_update` |
| `اتحاد` | `union` |
| `دمج` | `update` |

### `str`

| Arabic | Python | |
| --- | --- | --- |
| `كبر` | `upper` | |
| `صغر` | `lower` | |
| `حرف_الأول` | `capitalize` | |
| `طوى` | `casefold` | |
| `وسط` | `center` | |
| `عد` | `count` | |
| `رمز` | `encode` | |
| `ينتهي_ب` | `endswith` | |
| `يبدأ_ب` | `startswith` | |
| `وسع_الجداول` | `expandtabs` | |
| `ابحث` | `find` | |
| `ابحث_من_آخر` | `rfind` | |
| `موقع` | `index` | |
| `موقع_من_آخر` | `rindex` | |
| `نسق` | `format` | |
| `نسق_من_قاموس` | `format_map` | |
| `هل_حروف` | `isalpha` | |
| `هل_حروف_ورقام` | `isalnum` | |
| `هل_أسكي` | `isascii` | |
| `هل_عشري` | `isdecimal` | |
| `هل_رقم` | `isdigit` | |
| `هل_رقمي` | `isnumeric` | |
| `هل_معرف` | `isidentifier` | |
| `هل_صغار` | `islower` | |
| `هل_كبار` | `isupper` | |
| `هل_قابل_للطباعة` | `isprintable` | |
| `هل_مسافة` | `isspace` | |
| `هل_عناوين` | `istitle` | |
| `اربط` | `join` | |
| `برر_يسار` | `ljust` | |
| `برر_يمين` | `rjust` | |
| `جرد` | `strip` | |
| `جرد_يسار` | `lstrip` | |
| `جرد_يمين` | `rstrip` | |
| `جهز_ترجمة` | `maketrans` | |
| `اقتسم` | `partition` | |
| `اقتسم_من_آخر` | `rpartition` | |
| `احذف_بادئة` | `removeprefix` | |
| `احذف_لاحقة` | `removesuffix` | |
| `استبدل` | `replace` | |
| `قسم` | `split` | |
| `قسم_من_آخر` | `rsplit` | |
| `قسم_الأسطر` | `splitlines` | |
| `بدل_الحالة` | `swapcase` | |
| `بعناوين` | `title` | |
| `ترجم` | `translate` | |
| `املأ_اصفار` | `zfill` | |

### `bytes`

| Arabic | Python |
| --- | --- |
| `فك_رمز` | `decode` |
| `من_ست_عشري` | `fromhex` |
| `ست_عشري` | `hex` |

---

## 6. Escape sequences (تسلسلات الهروب)

Inside a non-raw string literal, a backslash followed by an **Arabic first
letter** stands for the corresponding Python escape, using the "original
naming" rule (the Arabic word's first letter). The `\` is kept and the letter
is rewritten to Python's ASCII escape.

| Shifra | Python | Arabic word |
| --- | --- | --- |
| `\ج` | `\n` | جديد (new line) |
| `\ط` | `\t` | تبويب (tab) |
| `\ر` | `\r` | رجوع (carriage return) |
| `\م` | `\\` | مائل (backslash) |
| `\ص` | `\0` | صفر (NUL) |
| `\س` | `\b` | سابق (backspace) |
| `\ن` | `\f` | نهاية (form feed) |
| `\ع` | `\v` | عمودي (vertical tab) |
| `\ت` | `\a` | تنبيه (bell) |

Notes:

- `\"` and `\'` keep their usual meaning and are not touched.
- The translation happens **only** inside ordinary string literals; in a raw
  string (`r"…"`, `rb"…"`, `fr"…"`, …) the text is kept verbatim, so writing
  `\ج` there yields the literal characters `\ج`.
- To output a literal backslash, use `\م` (`اطبع("مسار\ملخط")` → a `\`).

### F-strings (سلاسل التنسيق)

Python's f-string prefix `f` is written `مـ` (م + tatweel). The whole literal
is emitted as a Python `f"…"`, the plain text between `{…}` expressions is kept
verbatim (with the escape rules of section 6 still applying), and every `{…}`
expression is translated with the full Shifra rules, so Arabic keywords work
inside expressions too.

| Shifra | Python |
| --- | --- |
| `مـ"{الاسم} = {القيمة}"` | `f"{الاسم} = {القيمة}"` |
| `مـ"الحالة: {'نعم' إذا منتهية وإلا 'لا'}"` | `f"الحالة: {'نعم' if منتهية else 'لا'}"` |

Notes:

- Only the `{…}` expression is translated; the surrounding text (even Arabic
  words such as `المهمة`) is preserved exactly as written.
- A quoted string inside an expression (e.g. `'نعم'`) is kept untouched.
- `{{` and `}}` produce a single literal brace, exactly as in Python.
- Latin `f` / `F` prefixes are also recognized, so existing `f"…"` literals keep
  working and get the same treatment.

## 7. Inline conditions (الشروط السطرية)

A ternary conditional expression stays on one line using `إذا`/`وإلا`:

```shifra
طابع = "موجب" إذا ن >= ٠ وإلا "سالب"
```

A one-line `if`/`else` *statement* is written with the body and the `:` on the
same line; the preprocessor places `وإلا`/`وإذا` on a new line (Python
cannot share a physical line with a finished `if` suite):

```shifra
إذا ن > ٣: اطبع("كبير") وإلا: اطبع("صغير")

# elif chain — all on one line
إذا ن > ١٠٠: اطبع("أ") وإذا ن > ١٠: اطبع("ب") وإلا: اطبع("ج")
```

This also works nested inside an indented block; the split `else`/`elif` keeps
the surrounding indentation.

## 8. Imports & library names (الاستيراد وأسماء المكتبات)

Standard-library module names can be written in Arabic **only in the module
position** of a `from … import` / `import` statement. English spellings keep
working as before.

| Shifra module | Python module |
| --- | --- |
| `معطيات_الصنوف` | `dataclasses` |
| `تصنيف` | `typing` |

Library names below are rewritten **everywhere** they appear as a bare word:
import member lists, decorators, type annotations, and assignment targets.

| Shifra name | Python name | Used for |
| --- | --- | --- |
| `معطيات_الصنف` | `dataclass` | `@معطيات_الصنف` decorator |
| `أي_نوع` | `Any` | type annotations |
| `__الكل__` | `__all__` | module export list (`الكل__ = ["…"]`) |
| `__الاسم__` | `__name__` | `إذا الاسم__ == "__main__":` |
| `__التهيئة__` | `__init__` | class constructor (`عرف __التهيئة__(الذات):`) |

A fully Arabic "package" example:

```shifra
من معطيات_الصنوف استورد معطيات_الصنف
من تصنيف استورد أي_نوع

__الكل__ = ["سياق"]

@معطيات_الصنف
صنف سياق:
    الاسم: نص
    الشيء: أي_نوع

إذا الاسم__ == "__main__":
    اطبع("يعمل")
```

Notes:

- The special value string `"__main__"` stays Latin: strings are preserved
  verbatim, so only `الاسم__` → `__name__` is translatable.
- Module names are looked up **only** right after `من`/`استورد`; elsewhere the
  word passes through (keep using the English module name for attribute access
  such as `dataclasses.معطيات_الصنف`).

---

## Editing the language

1. Edit this file (add/rename/remove rows; keep the table structure).
2. Tell the agent to re-sync: `keywords.rs`, `builtins.rs`, `methods.rs`, `modules.rs`, and the RTL editor's tokenizer sets.
3. Rebuild (`cargo build --release`) and re-package the VS Code extension.