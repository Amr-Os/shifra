package com.shifra.language;

import android.content.Context;
import android.graphics.Canvas;
import android.graphics.Paint;
import android.graphics.Typeface;
import android.text.Editable;
import android.text.InputType;
import android.text.Layout;
import android.text.Spanned;
import android.text.TextWatcher;
import android.text.style.BackgroundColorSpan;
import android.text.style.ForegroundColorSpan;
import android.util.AttributeSet;
import android.view.KeyEvent;
import android.view.MotionEvent;
import android.view.ViewConfiguration;
import android.widget.EditText;

import java.util.HashSet;
import java.util.Set;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

public class KodeEditor extends EditText {

    private static final int COLOR_DEFAULT = 0xffd4d4d4;
    private static final int COLOR_COMMENT = 0xff6a9955;
    private static final int COLOR_STRING = 0xffce9178;
    private static final int COLOR_KEYWORD = 0xff569cd6;
    private static final int COLOR_BUILTIN = 0xffdcdcaa;
    private static final int COLOR_VARIABLE = 0xff9cdcfe;
    private static final int COLOR_NUMBER = 0xffb5cea8;
    private static final int COLOR_GUTTER = 0xff858585;
    private static final int COLOR_BRACKET = 0xff264f78;
    private static final int COLOR_GUTTER_DIVIDER = 0xff3c3c3c;
    private static final int COLOR_GUIDE = 0xff3c3c3c;

    private static final Pattern TOKEN = Pattern.compile(
            "(#[^\\n]*)"
                    + "|(\"\"\"[\\s\\S]*?\"\"\"|'''[\\s\\S]*?'''|\"(?:[^\"\\\\]|\\\\.)*\"|'(?:[^'\\\\]|\\\\.)*')"
                    + "|(?<![\\p{L}\\p{N}_])([0-9٠-٩]+(?:\\.[0-9٠-٩]+)?)"
                    + "|([\\p{L}_][\\p{L}\\p{N}_]*)");

    private static final Set<String> KEYWORDS = new HashSet<>();
    private static final Set<String> BUILTINS = new HashSet<>();

    static {
        String[] kw = {
                "و", "باسم", "تحقق", "لازمني", "انتظر", "توقف", "حالة", "صنف", "استمر", "عرف", "احذف",
                "وإذا", "وإلا", "التقط", "خاطئ", "ختاما", "لكل", "من", "عام", "إذا", "استورد", "ضمن", "هو",
                "لامبدا", "عدم", "لامحلي", "ليس", "أو", "تجاوز", "ارم", "أعد", "صحيح", "جرب", "نوع",
                "طالما", "مع", "انتج", "طابق",
                "def", "as", "assert", "async", "await", "break", "case", "class", "continue", "del", "elif",
                "else", "except", "False", "finally", "for", "from", "global", "if", "import", "in", "is",
                "lambda", "None", "nonlocal", "not", "or", "pass", "raise", "return", "True", "try", "type",
                "while", "with", "yield", "match", "self"};
        for (String s : kw) KEYWORDS.add(s);

        String[] bi = {
                "اطبع", "ادخل", "طول", "نطاق", "قائمة", "قاموس", "مجموعة", "متسلسلة", "مجموع", "اصغر", "اكبر",
                "عدد", "كسري", "مركب", "منطقي", "نص", "بايت", "بيتات", "مطلق", "الكل", "أي", "كرر",
                "التالي", "مكرر_غير_متزامن", "التالي_غير_متزامن", "ترقيم", "رشح", "طبق", "دمج", "كائن", "خاصية",
                "دالة_صنف", "دالة_ثابتة", "دور", "قوة", "قسمة_باقية", "معكوس", "مرتب", "نسق", "تمثيل",
                "هوية", "قيم", "نفذ", "هاش", "مساعدة", "افتح", "حرف", "رمز_الحرف", "ثنائي", "ثماني", "ست_عشري",
                "أسكي", "شريحة", "دليل", "متغيرات", "عالميات", "محليات", "اجمع", "نقطة_توقف", "عرض_الذاكرة",
                "علوي", "اجلب_سمة", "عين_سمة", "احذف_سمة", "هل_له_سمة", "مثيل", "مشتق",
                "استدعائي", "غير_منفذ", "ثلاث_نقاط", "خطأ", "خطأ_أساسي", "خطأ_حسابي", "خطأ_تحقق",
                "خطأ_سمة", "خطأ_نظام_التشغيل", "خطأ_نهاية_الملف", "خطأ_استيراد", "حيد", "خطأ_مفتاح",
                "خطأ_بحث", "خطأ_ذاكرة", "خطأ_اسم", "خطأ_غير_منفذ", "خطأ_فيضان", "خطأ_مرجع",
                "خطأ_وقت_التشغيل", "خطأ_إزاحة", "خطأ_صياغة", "خطأ_تبويب", "خطأ_نوع", "خطأ_قيمة",
                "خطأ_القسمة_على_صفر", "خطأ_متغير_محلي", "خطأ_مخزن", "خطأ_استدعاء_ذاتي",
                "خطأ_وحدة_غير_موجودة", "خطأ_ملف_غير_موجود", "خطأ_ملف_موجود", "خطأ_صلاحية", "خطأ_اتصال",
                "خطأ_اتصال_مرفوض", "خطأ_اتصال_مقاطع", "خطأ_اتصال_معاد", "خطأ_أنبوب_مكسور", "خطأ_إدخال_محجوب",
                "خطأ_إدخال_إخراج", "خطأ_بيئة", "خطأ_نقطة_كسرية", "خطأ_مقاطع", "خطأ_مسار_دليل",
                "خطأ_ليس_دليلا", "خطأ_عملية_غير_موجودة", "خطأ_عملية_طفل", "خطأ_يونيكود",
                "خطأ_فك_يونيكود", "خطأ_ترميز_يونيكود", "خطأ_ترجمة_يونيكود", "خطأ_نظام", "مجموعة_أخطاء",
                "مجموعة_أخطاء_أساسية", "توقف_التكرار", "توقف_التكرار_غير_متزامن", "خروج_النظام",
                "خروج_المولد", "انقطاع_لوحة_المفاتيح", "تحذير", "تحذير_تقادم", "تحذير_ترميز",
                "تحذير_مستقبلي", "تحذير_استيراد", "تحذير_صياغة", "تحذير_وقت_التشغيل", "تحذير_مستخدم",
                "تحذير_بايت", "تحذير_موارد",
                "print", "input", "len", "range", "list", "dict", "set", "tuple", "sum", "min", "max", "int",
                "float", "complex", "bool", "str", "bytes", "bytearray", "abs", "all", "any", "iter", "next",
                "aiter", "anext", "enumerate", "filter", "map", "zip", "object", "property", "classmethod",
                "staticmethod", "round", "pow", "divmod", "reversed", "sorted", "format", "repr", "id", "eval",
                "exec", "hash", "help", "open", "chr", "ord", "bin", "oct", "hex", "ascii", "slice", "dir",
                "vars", "globals", "locals", "compile", "breakpoint", "memoryview", "super", "getattr",
                "setattr", "delattr", "hasattr", "isinstance", "issubclass", "callable"};
        for (String s : bi) BUILTINS.add(s);
    }

    private final Paint gutterPaint = new Paint();
    private final Paint gutterDivider = new Paint();
    private final Paint indentGuidePaint = new Paint();
    private int gutterWidth;
    private String lastText = "";
    private boolean suppressWatch;
    private int lastA, lastB, lastC;
    private boolean lastFirstNewline;
    private boolean pendingSingle;
    private int pendingPos = -1;
    private char pendingChar;
    private int version;
    private final Runnable highlightTask = this::doHighlight;
    private final int touchSlop;
    private float downX, downY;
    private boolean dragging;
    private boolean softWrap;

    public static final int INDENT = 4;

    public KodeEditor(Context context, AttributeSet attrs) {
        super(context, attrs);
        touchSlop = ViewConfiguration.get(context).getScaledTouchSlop();
        gutterPaint.setColor(COLOR_GUTTER);
        gutterPaint.setAntiAlias(true);
        gutterPaint.setTextSize(getTextSize() * 0.75f);
        gutterDivider.setColor(COLOR_GUTTER_DIVIDER);
        gutterDivider.setStrokeWidth(dp(1));
        indentGuidePaint.setColor(COLOR_GUIDE);
        indentGuidePaint.setStrokeWidth(dp(1));
        setTypeface(Typeface.MONOSPACE);
        setTextColor(COLOR_DEFAULT);
        setHorizontallyScrolling(true);
        computeGutter();
        addTextChangedListener(new TextWatcher() {
            @Override public void beforeTextChanged(CharSequence s, int a, int b, int c) {}
            @Override public void onTextChanged(CharSequence s, int a, int b, int c) {
                lastA = a;
                lastB = b;
                lastC = c;
                lastFirstNewline = c > 0 && a < s.length() && s.charAt(a) == '\n';
                // remember exactly when the user types a single character, so the
                // auto-close logic can tell "typed a quote" apart from "a quote just
                // happens to sit after the caret"
                pendingSingle = !suppressWatch && c == 1 && b == 0;
                if (pendingSingle) {
                    pendingPos = a;
                    pendingChar = s.charAt(a);
                }
            }
            @Override public void afterTextChanged(Editable s) {
                if (suppressWatch) return;
                boolean single = pendingSingle;
                pendingSingle = false;
                autoIndent(s);
                if (single) autoBrackets(s, pendingPos, pendingChar);
                lastText = s.toString();
                scheduleHighlight();
            }
        });
    }

    /** @noinspection unused*/
    private void autoIndent(Editable s) {
        String text = s.toString();
        if (lastFirstNewline && lastB <= 1 && lastC <= 2
                && lastA < text.length() && text.charAt(lastA) == '\n') {
            // Enter pressed on the soft/hard keyboard: carry the current line's
            // indentation to the new line. Also handles IMEs that deliver Enter
            // as a one-char replacement (b == 1), keeping any trailing chars the
            // IME appended after the newline intact.
            String indent = nextIndent(text, lastA);
            if (indent.isEmpty()) return;
            suppressWatch = true;
            s.insert(lastA + 1, indent);
            int caret = lastA + 1 + indent.length() + (lastC - 1);
            if (caret > s.length()) caret = s.length();
            setSelection(caret);
            suppressWatch = false;
            return;
        }
        if (text.length() <= lastText.length()) return;
        int p = commonPrefixLen(text, lastText);
        int q = commonSuffixLen(text, lastText, p);
        if (p + q != lastText.length()) return; // replace/edit, not a plain insert
        String inserted = text.substring(p, text.length() - q);
        int nl = inserted.indexOf('\n');
        if (inserted.length() > 1 && nl >= 0) {
            // pasted multi-line block: re-indent continuation lines relative
            // to the base indent of the first pasted line
            reindentPaste(s, text, p, inserted);
        }
    }

    private void reindentPaste(Editable s, String text, int insStart, String inserted) {
        String[] lines = inserted.split("\n", -1);
        if (lines.length < 2) return;
        int minIndent = Integer.MAX_VALUE;
        for (int i = 1; i < lines.length; i++) {
            minIndent = Math.min(minIndent, leadingSpaces(lines[i]));
        }
        if (minIndent == Integer.MAX_VALUE || minIndent < 0) minIndent = 0;

        int baseLineStart = text.lastIndexOf('\n', insStart - 1) + 1;
        int base = leadingSpaces(text.substring(baseLineStart, text.length()));

        // absolute start of each pasted line (i=1..) before any edits
        int lineStart = insStart;
        int[] starts = new int[lines.length];
        for (int i = 0; i < lines.length; i++) {
            starts[i] = lineStart;
            lineStart += lines[i].length() + 1;
        }
        for (int i = lines.length - 1; i >= 1; i--) {
            String line = s.toString();
            int slStart = starts[i];
            if (slStart > line.length()) continue;
            int cur = leadingSpaces(line.substring(slStart, line.length()));
            int target = base + (leadingSpaces(lines[i]) - minIndent);
            if (target < cur) {
                suppressWatch = true;
                s.delete(slStart, slStart + (cur - target));
                suppressWatch = false;
            } else if (target > cur) {
                StringBuilder sb = new StringBuilder();
                for (int k = 0; k < target - cur; k++) sb.append(' ');
                suppressWatch = true;
                s.insert(slStart, sb.toString());
                suppressWatch = false;
            }
        }
    }

    private int commonPrefixLen(String a, String b) {
        int n = Math.min(a.length(), b.length());
        int i = 0;
        while (i < n && a.charAt(i) == b.charAt(i)) i++;
        return i;
    }

    private int commonSuffixLen(String a, String b, int prefix) {
        int i = a.length() - 1, j = b.length() - 1;
        int n = 0;
        while (i >= prefix && j >= prefix && a.charAt(i) == b.charAt(j)) {
            i--;
            j--;
            n++;
        }
        return n;
    }

    private int leadingSpaces(String s) {
        int i = 0;
        while (i < s.length() && s.charAt(i) == ' ') i++;
        return i;
    }

    private void autoBrackets(Editable s, int pos, char c) {
        if (pos < 0 || pos >= s.length()) return;
        int insertEnd = pos + 1;
        char next = insertEnd < s.length() ? s.charAt(insertEnd) : '\0';
        int oi = "([{".indexOf(c);
        int ci = ")]}".indexOf(c);
        suppressWatch = true;
        if (oi >= 0) {
            char close = ")]}".charAt(oi);
            if (next != close) {
                s.insert(insertEnd, String.valueOf(close));
            }
            setSelection(insertEnd);
        } else if (c == '"' || c == '\'') {
            if (next == c) {
                // typing over the auto-closed quote: skip it instead of stacking
                s.delete(pos, insertEnd);
                setSelection(pos + 1);
            } else {
                s.insert(insertEnd, String.valueOf(c));
                setSelection(insertEnd);
            }
        } else if (ci >= 0 && next == c) {
            // typing a closing bracket right before its auto-inserted match: skip
            s.delete(pos, insertEnd);
            setSelection(pos + 1);
        }
        suppressWatch = false;
    }

    private String nextIndent(String text, int nlPos) {
        int lineStart = text.lastIndexOf('\n', nlPos - 1) + 1;
        String line = text.substring(lineStart, nlPos);
        String trimmed = line.trim();
        if (trimmed.isEmpty()) return "";
        int base = 0;
        while (base < line.length() && line.charAt(base) == ' ') base++;
        if (trimmed.endsWith(":")) base += INDENT;
        StringBuilder sb = new StringBuilder(base);
        for (int i = 0; i < base; i++) sb.append(' ');
        return sb.toString();
    }

    @Override
    public boolean onKeyDown(int keyCode, KeyEvent event) {
        if (keyCode == KeyEvent.KEYCODE_TAB) {
            handleTab(event.isShiftPressed());
            return true;
        }
        if (keyCode == KeyEvent.KEYCODE_DEL) {
            if (dedentBackspace()) return true;
        }
        return super.onKeyDown(keyCode, event);
    }

    private void handleTab(boolean shiftDown) {
        int selStart = getSelectionStart();
        int selEnd = getSelectionEnd();
        if (selStart < 0 || selEnd < 0) return;
        if (selStart == selEnd) {
            if (shiftDown) dedentLine(selStart);
            else indentAt(selStart);
        } else {
            indentLineRange(selStart, selEnd, !shiftDown);
        }
    }

    private void indentAt(int pos) {
        if (pos < 0) pos = 0;
        StringBuilder sb = new StringBuilder();
        for (int i = 0; i < INDENT; i++) sb.append(' ');
        Editable e = getText();
        boolean sel = pos == getSelectionStart();
        e.insert(pos, sb.toString());
        if (sel) setSelection(pos + INDENT);
    }

    private void dedentLine(int pos) {
        Editable e = getText();
        int lineStart = lineStartAt(e.toString(), pos);
        int lineEnd = lineEndAt(e.toString(), pos);
        int spaces = 0;
        for (int i = lineStart; i < lineEnd && e.charAt(i) == ' '; i++) spaces++;
        int remove = Math.min(INDENT, spaces);
        if (remove > 0) {
            e.delete(lineStart, lineStart + remove);
            if (pos > lineStart) setSelection(Math.max(pos - remove, lineStart));
            else setSelection(lineStart);
        }
    }

    private void indentLineRange(int selStart, int selEnd, boolean indent) {
        String text = getText().toString();
        int startLine = text.substring(0, selStart).isEmpty() ? 0 : countLines(text.substring(0, selStart));
        int endLine = countLines(text.substring(0, selEnd));
        // iterate from bottom so offsets stay valid
        String[] lines = text.split("\n", -1);
        Editable e = getText();
        int[] lineStarts = new int[lines.length];
        int at = 0;
        for (int i = 0; i < lines.length; i++) {
            lineStarts[i] = at;
            at += lines[i].length() + 1;
        }
        for (int i = Math.min(endLine, lines.length - 1); i >= startLine; i--) {
            int ls = lineStarts[i];
            if (indent) {
                e.insert(ls, "    ");
            } else {
                int spaces = leadingSpaces(lines[i]);
                int remove = Math.min(INDENT, spaces);
                if (remove > 0) e.delete(ls, ls + remove);
            }
        }
        // keep a sensible selection: whole block
        setSelection(selStart + (indent ? INDENT : 0), selEnd + (indent ? INDENT : 0));
    }

    private int lineStartAt(String text, int pos) {
        return text.lastIndexOf('\n', Math.max(0, pos - 1)) + 1;
    }

    private int lineEndAt(String text, int pos) {
        int nl = text.indexOf('\n', pos);
        return nl < 0 ? text.length() : nl;
    }

    private int countLines(String s) {
        int n = 0;
        for (int i = 0; i < s.length(); i++) if (s.charAt(i) == '\n') n++;
        return n;
    }

    private boolean dedentBackspace() {
        int selStart = getSelectionStart();
        int selEnd = getSelectionEnd();
        if (selStart != selEnd) return false;
        Editable e = getText();
        String text = e.toString();
        int lineStart = lineStartAt(text, selStart);
        int lineEnd = lineEndAt(text, selStart);
        String fullLine = text.substring(lineStart, lineEnd);
        boolean onlySpaces = !fullLine.isEmpty() && fullLine.trim().isEmpty();
        int spaces = leadingSpaces(fullLine);
        if (onlySpaces && selStart > lineStart && selStart <= lineStart + spaces) {
            int toRemove = Math.min(INDENT, selStart - lineStart);
            if (toRemove > 0) {
                e.delete(selStart - toRemove, selStart);
                setSelection(selStart - toRemove);
                return true;
            }
        }
        return false;
    }

    @Override
    public boolean onTouchEvent(MotionEvent ev) {
        int action = ev.getActionMasked();
        if (action == MotionEvent.ACTION_DOWN) {
            downX = ev.getX();
            downY = ev.getY();
            dragging = false;
        } else if (action == MotionEvent.ACTION_MOVE) {
            if (hasOverflow()) {
                float dx = ev.getX() - downX;
                float dy = ev.getY() - downY;
                if (!dragging && (Math.abs(dx) > touchSlop || Math.abs(dy) > touchSlop)) {
                    dragging = true;
                }
                if (dragging) {
                    scrollBy((int) -dx, (int) -dy);
                    downX = ev.getX();
                    downY = ev.getY();
                    return true;
                }
                return true;
            }
        } else if (action == MotionEvent.ACTION_UP || action == MotionEvent.ACTION_CANCEL) {
            if (dragging) {
                dragging = false;
                return true;
            }
        }
        return super.onTouchEvent(ev);
    }

    @Override
    public void scrollTo(int x, int y) {
        int maxX = maxScrollX();
        int maxY = maxScrollY();
        if (x < 0) x = 0;
        if (x > maxX) x = maxX;
        if (y < 0) y = 0;
        if (y > maxY) y = maxY;
        super.scrollTo(x, y);
    }

    private int contentWidth() {
        Layout l = getLayout();
        if (l == null) return 0;
        int w = 0;
        for (int i = 0; i < l.getLineCount(); i++) {
            w = Math.max(w, (int) Math.ceil(l.getLineRight(i)));
        }
        return w;
    }

    private int maxScrollX() {
        return Math.max(0, contentWidth() - (getWidth() - getPaddingLeft() - getPaddingRight()));
    }

    private int maxScrollY() {
        Layout l = getLayout();
        if (l == null) return 0;
        return Math.max(0, l.getHeight() + getPaddingTop() + getPaddingBottom() - getHeight());
    }

    private boolean hasOverflow() {
        return maxScrollX() > 0 || maxScrollY() > 0;
    }

    @Override
    protected void onDraw(Canvas canvas) {
        Layout layout = getLayout();
        if (layout != null) {
            int digits = Math.max(2, String.valueOf(getLineCount()).length());
            int need = (int) (digits * gutterPaint.measureText("8") + dp(18));
            if (need != gutterWidth) gutterWidth = need;
            if (!softWrap) {
                drawIndentGuides(canvas, layout);
                int scrollX = getScrollX();
                int scrollY = getScrollY();
                int w = getWidth();
                int h = getHeight();
                int gutterLeft = scrollX + w - gutterWidth;
                int first = layout.getLineForVertical(scrollY);
                int last = Math.min(layout.getLineCount() - 1, layout.getLineForVertical(scrollY + h));
                for (int l = first; l <= last; l++) {
                    String num = String.valueOf(l + 1);
                    float baseline = layout.getLineBaseline(l) + getPaddingTop();
                    float x = scrollX + w - dp(8) - gutterPaint.measureText(num);
                    canvas.drawText(num, x, baseline, gutterPaint);
                }
                canvas.drawLine(gutterLeft + dp(3), scrollY, gutterLeft + dp(3), scrollY + h, gutterDivider);
            }
        }
        super.onDraw(canvas);
    }

    private void drawIndentGuides(Canvas canvas, Layout layout) {
        String text = getText().toString();
        int n = layout.getLineCount();
        int[] indent = new int[n];
        int maxSpaces = 0;
        for (int l = 0; l < n; l++) {
            int start = layout.getLineStart(l);
            int end = layout.getLineEnd(l);
            int spaces = 0;
            int k = start;
            while (k < end) {
                char c = text.charAt(k);
                if (c == ' ') spaces += 1;
                else if (c == '\t') spaces += INDENT;
                else break;
                k++;
            }
            boolean blank = true;
            for (int i = k; i < end; i++) {
                if (!Character.isWhitespace(text.charAt(i))) {
                    blank = false;
                    break;
                }
            }
            indent[l] = blank ? 0 : spaces;
            if (spaces > maxSpaces) maxSpaces = spaces;
        }
        float inset = Math.max(1, dp(2));
        int scrollY = getScrollY();
        int firstVis = layout.getLineForVertical(scrollY);
        int lastVis = Math.min(n - 1, layout.getLineForVertical(scrollY + getHeight()));
        int padLeft = getCompoundPaddingLeft();
        int padTop = getPaddingTop();
        for (int k = 1; k * INDENT <= maxSpaces && k <= 24; k++) {
            int col = k * INDENT;
            int a = -1;
            for (int l = 0; l <= n; l++) {
                boolean inRun = l < n && indent[l] >= col;
                if (inRun) {
                    if (a < 0) a = l;
                } else if (a >= 0) {
                    int b = l - 1;
                    if (b - a + 1 >= 2 && a <= lastVis && b >= firstVis) {
                        int end = layout.getLineEnd(a);
                        int off = Math.min(layout.getLineStart(a) + col, end);
                        float x = layout.getPrimaryHorizontal(off) + padLeft + inset;
                        int top = layout.getLineTop(a) + padTop;
                        int bottom = layout.getLineBottom(b) + padTop;
                        canvas.drawLine(x, top, x, bottom, indentGuidePaint);
                    }
                    a = -1;
                }
            }
        }
    }

    private void computeGutter() {
        int digits = Math.max(2, String.valueOf(getLineCount()).length());
        gutterWidth = (int) (digits * gutterPaint.measureText("8") + dp(18));
    }

    @Override
    protected void onLayout(boolean changed, int left, int top, int right, int bottom) {
        super.onLayout(changed, left, top, right, bottom);
        if (changed) {
            computeGutter();
            setPadding(dp(10), dp(10), softWrap ? dp(10) : gutterWidth, dp(10));
        }
    }

    /**
     * Insert text without the auto-indent/auto-bracket watcher interfering, then
     * place the caret at {@code caret} (an absolute index into the result).
     * Both the insert position and the caret are clamped to the text length so a
     * stale selection can never turn into an off-by-N setSelection crash.
     */
    public void insertAtomically(int pos, String text, int caret) {
        Editable e = getText();
        if (e == null) return;
        if (pos < 0) pos = 0;
        if (pos > e.length()) pos = e.length();
        suppressWatch = true;
        try {
            e.insert(pos, text);
            lastText = e.toString();
        } finally {
            suppressWatch = false;
        }
        int len = e.length();
        if (caret < 0) caret = 0;
        if (caret > len) caret = len;
        setSelection(caret);
        requestFocus();
        scheduleHighlight();
    }

    public void scheduleHighlight() {
        version++;
        removeCallbacks(highlightTask);
        postDelayed(highlightTask, 150);
    }

    /**
     * A whole-document set (tab restore, undo/redo, replace-all, etc.) must not
     * go through the auto-indent/auto-bracket watcher: reindentPaste would treat
     * the freshly loaded text as a pasted block and strip the indentation of
     * every continuation line relative to the first line.
     */
    @Override
    public void setText(CharSequence text, BufferType type) {
        if (text == null) text = "";
        suppressWatch = true;
        super.setText(text, type);
        suppressWatch = false;
        lastText = text.toString();
        scheduleHighlight();
    }

    private void doHighlight() {
        int v = version;
        Editable e = getText();
        if (e == null) return;
        String text = e.toString();
        for (ForegroundColorSpan s : e.getSpans(0, e.length(), ForegroundColorSpan.class)) {
            e.removeSpan(s);
        }
        if (v != version) return;
        Matcher m = TOKEN.matcher(text);
        while (m.find()) {
            if (v != version) return;
            int color;
            if (m.group(1) != null) color = COLOR_COMMENT;
            else if (m.group(2) != null) color = COLOR_STRING;
            else if (m.group(3) != null) color = COLOR_NUMBER;
            else {
                String id = m.group(4);
                boolean isCall = m.end() < text.length() && text.charAt(m.end()) == '(';
                if (KEYWORDS.contains(id)) color = COLOR_KEYWORD;
                else if (BUILTINS.contains(id) || isCall) color = COLOR_BUILTIN;
                else color = COLOR_VARIABLE;
            }
            e.setSpan(new ForegroundColorSpan(color), m.start(), m.end(), Spanned.SPAN_EXCLUSIVE_EXCLUSIVE);
        }
        e.setSpan(new ForegroundColorSpan(COLOR_DEFAULT), 0, 0, Spanned.SPAN_EXCLUSIVE_EXCLUSIVE);
    }

    private BackgroundColorSpan bracketSpan1;
    private BackgroundColorSpan bracketSpan2;

    @Override
    public void onSelectionChanged(int selStart, int selEnd) {
        super.onSelectionChanged(selStart, selEnd);
        invalidate();
        updateBracketMatch();
    }

    private void updateBracketMatch() {
        Editable e = getText();
        if (e == null) return;
        if (bracketSpan1 != null) e.removeSpan(bracketSpan1);
        if (bracketSpan2 != null) e.removeSpan(bracketSpan2);
        bracketSpan1 = null;
        bracketSpan2 = null;
        int pos = getSelectionStart();
        if (pos != getSelectionEnd()) return;
        CharSequence text = e;
        if (pos < 0 || pos > text.length()) return;
        char c = pos > 0 ? text.charAt(pos - 1) : 0;
        int open = -1, close = -1;
        char openC = 0, closeC = 0;
        if (c == '(' || c == ')' || c == '[' || c == ']' || c == '{' || c == '}') {
            int other;
            if (c == '(' || c == '[' || c == '{') {
                other = findMatching(text, pos - 1, c);
                if (other >= 0) {
                    open = pos - 1;
                    close = other;
                }
            } else {
                other = findMatchingReverse(text, pos - 1, c);
                if (other >= 0) {
                    open = other;
                    close = pos - 1;
                }
            }
        }
        if (open >= 0) {
            int h = COLOR_BRACKET;
            bracketSpan1 = new BackgroundColorSpan(h);
            bracketSpan2 = new BackgroundColorSpan(h);
            e.setSpan(bracketSpan1, open, open + 1, Spanned.SPAN_EXCLUSIVE_EXCLUSIVE);
            e.setSpan(bracketSpan2, close, close + 1, Spanned.SPAN_EXCLUSIVE_EXCLUSIVE);
        }
    }

    private int findMatching(CharSequence text, int openPos, char openC) {
        char closeC;
        if (openC == '(') closeC = ')';
        else if (openC == '[') closeC = ']';
        else if (openC == '{') closeC = '}';
        else return -1;
        int depth = 1;
        for (int i = openPos + 1; i < text.length(); i++) {
            char ch = text.charAt(i);
            if (ch == openC) depth++;
            else if (ch == closeC) {
                depth--;
                if (depth == 0) return i;
            }
        }
        return -1;
    }

    private int findMatchingReverse(CharSequence text, int closePos, char closeC) {
        char openC;
        if (closeC == ')') openC = '(';
        else if (closeC == ']') openC = '[';
        else if (closeC == '}') openC = '{';
        else return -1;
        int depth = 1;
        for (int i = closePos - 1; i >= 0; i--) {
            char ch = text.charAt(i);
            if (ch == closeC) depth++;
            else if (ch == openC) {
                depth--;
                if (depth == 0) return i;
            }
        }
        return -1;
    }

    public void applyFontSize(float sp) {
        setTextSize(sp);
        gutterPaint.setTextSize(getTextSize() * 0.75f);
        computeGutter();
        requestLayout();
        setPadding(dp(10), dp(10), softWrap ? dp(10) : gutterWidth, dp(10));
        scheduleHighlight();
    }

    public void applyFontFamily(String family) {
        if ("serif".equals(family)) setTypeface(Typeface.SERIF);
        else if ("sans".equals(family)) setTypeface(Typeface.SANS_SERIF);
        else setTypeface(Typeface.MONOSPACE);
    }

    /** Enable/disable soft word-wrapping (keeps the line-number gutter in both modes). */
    public void setSoftWrap(boolean wrap) {
        if (softWrap == wrap) return;
        softWrap = wrap;
        setHorizontallyScrolling(!wrap);
        setPadding(dp(10), dp(10), softWrap ? dp(10) : gutterWidth, dp(10));
        setInputType(getInputType() | InputType.TYPE_TEXT_FLAG_MULTI_LINE);
        requestLayout();
    }

    public boolean isSoftWrap() {
        return softWrap;
    }

    private int dp(int value) {
        return Math.round(getResources().getDisplayMetrics().density * value);
    }
}