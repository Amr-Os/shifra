"""Shifra Arabic exception reporter.

Installed as ``sys.excepthook`` while running Shifra (``.ar``/``.sf``)
programs. It mirrors RustPython's default traceback rendering but writes the
messages in Arabic and maps exception class names back to their Arabic
spellings (see ``builtins.BUILTINS``). Rendering is best-effort: if anything
fails it falls back to a minimal textual report so the original error is never
lost.

Deliberately avoids ``io``: the Android build ships without the ``fileio``
feature, so importing ``io`` and even ``import io`` itself fails there.
"""

import re
import sys

import __main__

_PY_KEYWORDS = {
    "break": "توقف",
    "case": "حالة",
    "class": "صنف",
    "continue": "استمر",
    "def": "عرف",
    "elif": "وإذا",
    "else": "وإلا",
    "except": "التقط",
    "finally": "ختاما",
    "for": "لكل",
    "if": "إذا",
    "match": "طابق",
    "pass": "تجاوز",
    "return": "أعد",
    "try": "جرب",
    "while": "طالما",
    "with": "مع",
}

_KEYWORD_CLAUSES = {
    "'for' statement": "جملة 'لكل'",
    "'if' statement": "جملة 'إذا'",
    "'elif' statement": "جملة 'وإذا'",
    "'else' statement": "جملة 'وإلا'",
    "'while' statement": "جملة 'طالما'",
    "'with' statement": "جملة 'مع'",
    "'try' statement": "جملة 'جرب'",
    "'except' statement": "جملة 'التقط'",
    "'finally' statement": "جملة 'ختاما'",
    "'match' statement": "جملة 'طابق'",
    "'case' statement": "كتلة 'حالة'",
    "'class' definition": "تعريف 'صنف'",
    "function definition": "تعريف دالة",
    "'except*' statement": "جملة 'التقط*'",
}


def _indented_block(message):
    match = re.fullmatch(
        r"expected an indented block after (.+) on line (\d+)", message
    )
    if not match:
        return message
    clause = _KEYWORD_CLAUSES.get(match.group(1), match.group(1))
    return "متوقع كتلة بإزاحة بعد %s في السطر %s" % (clause, match.group(2))


_MESSAGE_PATTERNS = [
    (re.compile(r"name '([^']*)' is not defined"), r"الاسم '\1' غير معرّف"),
    (
        re.compile(r"free variable '([^']*)' referenced before assignment"),
        r"المتغير الحر '\1' أُشير إليه قبل إسناده",
    ),
    (
        re.compile(r"local variable '([^']*)' referenced before assignment"),
        r"أُشير إلى المتغير المحلي '\1' قبل إسناده",
    ),
    (
        re.compile(r"cannot access local variable '([^']*)'"),
        r"لا يمكن الوصول إلى المتغير المحلي '\1'",
    ),
    (
        re.compile(r"cannot import name '([^']*)' from '([^']*)'"),
        r"لا يمكن استيراد الاسم '\1' من '\2'",
    ),
    (re.compile(r"(?:No|no) module named '([^']*)'"), r"لا توجد وحدة باسم '\1'"),
    (
        re.compile(r"unsupported operand type\(s\) for"),
        r"أنواع معاملات غير مدعومة للعملية",
    ),
    (re.compile(r"'([^']*)' and '([^']*)'"), r"'\1' و '\2'"),
    (
        re.compile(r"'([^']*)' object has no attribute '([^']*)'"),
        r"الكائن '\1' لا يحتوي على خاصية '\2'",
    ),
    (
        re.compile(r"object has no attribute '([^']*)'"),
        r"الكائن لا يحتوي على خاصية '\1'",
    ),
    (re.compile(r"has no attribute"), r"لا يحتوي على خاصية"),
    (re.compile(r"cannot unpack non-iterable"), r"لا يمكن تفكيك غير قابل للتكرار"),
    (re.compile(r"not enough values to unpack"), r"قيم غير كافية للتفكيك"),
    (re.compile(r"too many values to unpack"), r"قيم أكثر من اللازم للتفكيك"),
    (
        re.compile(r"multiple values for argument '([^']*)'"),
        r"قيم متعددة للوسيط '\1'",
    ),
    (
        re.compile(r"missing 1 required positional argument: '([^']*)'"),
        r"نقص وسيط موضعي مطلوب: '\1'",
    ),
    (re.compile(r"is not callable"), r"غير قابل للاستدعاء"),
    (re.compile(r"is not iterable"), r"غير قابل للتكرار"),
    (re.compile(r"not subscriptable"), r"غير قابل للفهرسة"),
    (re.compile(r"division by zero"), r"القسمة على صفر"),
    (re.compile(r"integer division or modulo by zero"), r"قسمة صحيحة أو باقٍ على صفر"),
    (re.compile(r"substring not found"), r"الجزء غير موجود في النص"),
    (re.compile(r"pop from empty list"), r"إخراج من قائمة فارغة"),
    (re.compile(r"list index out of range"), r"مؤشر القائمة خارج النطاق"),
    (re.compile(r"string index out of range"), r"مؤشر النص خارج النطاق"),
    (re.compile(r"tuple index out of range"), r"مؤشر المتسلسلة خارج النطاق"),
    (re.compile(r"index out of range"), r"مؤشر خارج النطاق"),
    (re.compile(r"maximum recursion depth exceeded"), r"تجاوز أقصى عمق للاستدعاء"),
    (re.compile(r"unexpected indent"), r"إزاحة غير متوقعة"),
    (re.compile(r"unexpected unindent"), r"إلغاء إزاحة غير متوقع"),
    (
        re.compile(r"unindent does not match any outer indentation level"),
        r"إلغاء الإزاحة لا يطابق أي مستوى خارجي",
    ),
    (re.compile(r"could not convert string to float"), r"تعذّر تحويل النص إلى كسري"),
    (re.compile(r"invalid literal for int\(\)"), r"قيمة حرفية غير صالحة للعدد"),
    (re.compile(r"cannot be interpreted as an integer"), r"لا يمكن تفسيرها كعدد صحيح"),
    (
        re.compile(r"not all arguments converted during string formatting"),
        r"لم تُحوَّل كل الوسائط أثناء تنسيق النص",
    ),
    (
        re.compile(r"not enough arguments for format string"),
        r"لا تكفي الوسائط لسلسلة التنسيق",
    ),
    (
        re.compile(r"positional argument follows keyword argument"),
        r"وسيط موضعي يأتي بعد وسيط مفتاحي",
    ),
    (re.compile(r"keyword argument repeated"), r"وسيط مفتاحي متكرر"),
    (
        re.compile(r"keyword can't be an expression"),
        r"لا يمكن للكلمة المفتاحية أن تكون تعبيرًا",
    ),
    (
        re.compile(r"argument after \* must be an iterable"),
        r"الوسيط بعد * يجب أن يكون قابلاً للتكرار",
    ),
    (re.compile(r"generator expression without parentheses"), r"تعبير مولّد دون أقواس"),
    (re.compile(r"cannot concatenate"), r"لا يمكن الدمج"),
    (
        re.compile(r"does not support the buffer protocol"),
        r"لا يدعم بروتوكول المخزن المؤقت",
    ),
    (re.compile(r"No such file or directory"), r"لا يوجد ملف أو دليل بهذا الاسم"),
    (re.compile(r"File exists"), r"الملف موجود"),
    (re.compile(r"Is a directory"), r"هو دليل"),
    (re.compile(r"Not a directory"), r"ليس دليلاً"),
    (re.compile(r"Permission denied"), r"تم رفض الصلاحية"),
    (re.compile(r"invalid syntax"), r"صياغة غير صالحة"),
    (re.compile(r"invalid decimal literal"), r"قيمة عشرية غير صالحة"),
    (re.compile(r"expected ':'"), r"متوقع ':'"),
    (re.compile(r"expected an indented block"), r"متوقع كتلة بإزاحة"),
    (re.compile(r"expected an expression"), r"متوقع تعبيرًا"),
    (
        re.compile(r"\":\" expected after dictionary key"),
        r"متوقع ':' بعد مفتاح القاموس",
    ),
    (
        re.compile(r"'in' expected after for-loop variables"),
        r"متوقع 'in' بعد متغيرات حلقة 'لكل'",
    ),
    (
        re.compile(r"expected expression after 'else', but statement is given"),
        r"متوقع تعبير بعد 'وإلا'، لكن عُطيت جملة",
    ),
    (
        re.compile(r"unexpected character after line continuation character"),
        r"حرف غير متوقع بعد حرف استمرار السطر",
    ),
    (
        re.compile(r"unexpected EOF while parsing"),
        r"نهاية ملف غير متوقعة أثناء التحليل",
    ),
    (
        re.compile(r"invalid non-printable character"),
        r"حرف غير قابل للطباعة غير صالح",
    ),
    (
        re.compile(r"cannot assign to expression statement"),
        r"لا يمكن الإسناد إلى تعبير",
    ),
    (re.compile(r"cannot delete"), r"لا يمكن الحذف"),
]

_TYPE_NAMES = {
    "int": "عدد",
    "float": "كسري",
    "complex": "مركب",
    "str": "نص",
    "bytes": "بايت",
    "bytearray": "بيتات",
    "list": "قائمة",
    "tuple": "متسلسلة",
    "dict": "قاموس",
    "set": "مجموعة",
    "frozenset": "مجموعة_مجمد",
    "bool": "منطقي",
    "NoneType": "عدم",
}

_SENTINEL = object()


def _ar_type_name(exc_type):
    globals_dict = getattr(__main__, "__dict__", {})
    for name, value in globals_dict.items():
        try:
            if value is exc_type:
                return name
        except Exception:
            continue
    return getattr(exc_type, "__name__", "")


def _ar_words(message):
    message = _indented_block(message)
    for pattern, replacement in _MESSAGE_PATTERNS:
        message = pattern.sub(replacement, message)
    for english, arabic in _TYPE_NAMES.items():
        message = message.replace("'" + english + "'", "'" + arabic + "'")
    return message


def _render(lines, exc_type, value, tb):
    lines.append("تتبّع (آخر استدعاء أولاً):")
    current = tb
    while current is not None:
        frame = current.tb_frame
        code = frame.f_code
        lines.append(
            '  ملف "%s"، سطر %s، في %s'
            % (code.co_filename, current.tb_lineno, code.co_name)
        )
        current = current.tb_next

    name = _ar_type_name(exc_type)

    if isinstance(value, (SyntaxError, IndentationError, TabError)):
        filename = getattr(value, "filename", "???")
        lineno = getattr(value, "lineno", "?")
        lines.append('  ملف "%s"، سطر %s' % (filename, lineno))
        text = getattr(value, "text", None)
        if text:
            line = str(text).rstrip("\n")
            lines.append("    " + line)
            offset = getattr(value, "offset", None)
            if offset:
                lines.append("    " + " " * max(0, int(offset) - 1) + "^")
        msg = getattr(value, "msg", _SENTINEL)
        if msg is _SENTINEL and value.args:
            msg = value.args[0]
        if msg is not _SENTINEL:
            lines.append("%s: %s" % (name, _ar_words(str(msg))))
        else:
            lines.append(name)
        return

    args = getattr(value, "args", ())
    if not args:
        lines.append(name)
    elif len(args) == 1:
        if isinstance(args[0], str):
            lines.append("%s: %s" % (name, _ar_words(args[0])))
        else:
            lines.append("%s: %r" % (name, args[0]))
    else:
        lines.append("%s: %s" % (name, ", ".join(repr(arg) for arg in args)))


def _render_plain(exc_type, value):
    try:
        name = getattr(exc_type, "__name__", str(exc_type))
        parts = " | ".join(repr(arg) for arg in getattr(value, "args", ()))
        return "%s: %s" % (name, parts)
    except Exception:
        try:
            return "%s" % (exc_type,)
        except Exception:
            return str(exc_type)


def shifra_excepthook(exc_type, value, tb):
    if isinstance(value, KeyboardInterrupt):
        try:
            sys.stderr.write("\n")
        except Exception:
            pass
        return
    lines = []
    try:
        _render(lines, exc_type, value, tb)
    except Exception:
        lines = [_render_plain(exc_type, value)]
    text = "\n".join(lines) + "\n"
    try:
        sys.stderr.write(text)
    except Exception:
        pass


sys.excepthook = shifra_excepthook
