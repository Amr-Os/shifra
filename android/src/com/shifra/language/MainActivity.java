package com.shifra.language;

import android.app.Activity;
import android.app.AlertDialog;
import android.content.Context;
import android.content.Intent;
import android.content.SharedPreferences;
import android.database.Cursor;
import android.net.Uri;
import android.os.Build;
import android.os.Bundle;
import android.provider.DocumentsContract;
import android.provider.OpenableColumns;
import android.text.Editable;
import android.text.TextWatcher;
import android.view.Gravity;
import android.view.Menu;
import android.view.MotionEvent;
import android.view.SubMenu;
import android.view.View;
import android.view.Window;
import android.view.WindowInsets;
import android.view.WindowInsetsController;
import android.view.WindowManager;
import android.view.inputmethod.EditorInfo;
import android.view.inputmethod.InputMethodManager;
import android.widget.EditText;
import android.widget.FrameLayout;
import android.widget.LinearLayout;
import android.widget.PopupMenu;
import android.widget.TextView;
import android.widget.Toast;

import java.io.BufferedReader;
import java.io.BufferedInputStream;
import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStreamReader;
import java.io.OutputStream;
import java.nio.charset.StandardCharsets;
import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

import org.json.JSONArray;
import org.json.JSONException;
import org.json.JSONObject;

public class MainActivity extends Activity {

    private static final int REQ_OPEN = 1;
    private static final int REQ_SAVE_AS = 2;

    private static final int FLAG_READ_WRITE =
            Intent.FLAG_GRANT_READ_URI_PERMISSION | Intent.FLAG_GRANT_WRITE_URI_PERMISSION;

    private static final int MENU_NEW = 1;
    private static final int MENU_DEMO = 2;
    private static final int MENU_OPEN = 3;
    private static final int MENU_SAVE = 4;
    private static final int MENU_SAVE_AS = 5;
    private static final int MENU_SNIPPET = 6;
    private static final int MENU_SAVE_SNIP = 7;
    private static final int MENU_FIX = 8;
    private static final int MENU_CLOSE = 9;
    private static final int MENU_UNDO = 10;
    private static final int MENU_REDO = 11;
    private static final int MENU_SETTINGS = 12;
    private static final int MENU_REPLACE = 14;
    private static final int MENU_GOTO = 15;

    private static final int MAX_UNDO = 100;

    private static final String[][] SNIPPETS = {
            {"دالة", "عرف اسم_الدالة(وسيط):\n    أعد"},
            {"شرط", "إذا شرط:\n    "},
            {"حلقة لكل", "لكل عنصر ضمن قائمة:\n    "},
            {"حلقة طالما", "طالما شرط:\n    "},
            {"محاولة", "جرب:\n    \nالتقط خطأ باسم خطأ:\n    \nختاما:\n    "},
            {"صنف", "صنف اسم_الصنف:\n    عرف __التهيئة__(الذات):\n        الذات.سمة = قيمة\n\n    عرف طريقة(الذات):\n        أعد"},
            {"استيراد", "استورد وحدة"},
    };

    private static final String[][] EXAMPLES = {
            {"السلام عليكم",
                    "# أول برنامج لك بلغة شِفرة\n" +
                    "# التعليمات تعمل سطرا سطرا من الأعلى إلى الأسفل\n" +
                    "اطبع(\"السلام عليكم\")\n" +
                    "اطبع(\"أهلا بك في لغة شِفرة\")\n"
            },
            {"الأرقام والحساب",
                    "# الأرقام والحساب\n" +
                    "# شِفرة تفهم الأرقام العربية ٠-٩ وتعرف كل العمليات الحسابية\n" +
                    "الكمية = ١٠          # نخصص قيمة لمتغير\n" +
                    "السعر = ٣.٥\n" +
                    "المجموع = الكمية * السعر + ٥\n" +
                    "اطبع(\"المجموع =\", المجموع)\n" +
                    "اطبع(\"مربع ٧ =\", قوة(٧, ٢), \"| مطلق -٧ =\", مطلق(-٧))\n" +
                    "اطبع(\"قسمة باقية ١٧÷٥ =\", قسمة_باقية(١٧, ٥))\n" +
                    "اطبع(\"تقريب ٣.٧ =\", دور(٣.٧), \"| القسمة =\", ١٧ / ٥, \"| الصحيحة =\", ١٧ // ٥)\n"
            },
            {"النصوص",
                    "# النصوص\n" +
                    "# النص بين علامتي اقتباس، ولديه دوال جاهزة تتعامل معه\n" +
                    "الاسم = \"رنا\"\n" +
                    "اطبع(الاسم.كبر())          # أحرف كبيرة\n" +
                    "اطبع(\"مرحبا\".صغر(), \"|\", \"shifra\".حرف_الأول())\n" +
                    "الكلمات = \"أحمد محمد علي\".قسم(\" \")   # تقطيع إلى قائمة\n" +
                    "اطبع(\"بعد القسم:\", الكلمات)\n" +
                    "اطبع(\"إعادة الربط:\", \"..\".اربط(الكلمات))\n" +
                    "جملة = \"مرحبا بك في عالم البرمجة\"\n" +
                    "اطبع(\"موضع 'عالم':\", جملة.ابحث(\"عالم\"))\n" +
                    "اطبع(\"يبدأ بمرحبا؟\", جملة.يبدأ_ب(\"مرحبا\"), \"| ينتهي بعالم؟\", جملة.ينتهي_ب(\"عالم\"))\n" +
                    "اطبع(\"استبدال:\", جملة.استبدل(\"البرمجة\", \"شِفرة\"))\n" +
                    "اطبع(\"بدون مسافات:\", \"   نص مع مسافات   \".جرد(), \"!\")\n"
            },
            {"الشروط والمنطق",
                    "# الشروط والمنطق\n" +
                    "# إذا / وإلا / وإذا توجّه البرنامج حسب الشروط\n" +
                    "السن = ٢٠\n" +
                    "إذا السن >= ١٨:\n" +
                    "    اطبع(\"بالغ\")\n" +
                    "وإلا:\n" +
                    "    اطبع(\"قاصر\")\n" +
                    "\n" +
                    "الدرجة = ٨٥\n" +
                    "إذا الدرجة >= ٩٠:\n" +
                    "    التقدير = \"ممتاز\"\n" +
                    "وإذا الدرجة >= ٧٠:\n" +
                    "    التقدير = \"جيد\"\n" +
                    "وإلا:\n" +
                    "    التقدير = \"يحتاج مراجعة\"\n" +
                    "اطبع(\"التقدير:\", التقدير)\n" +
                    "\n" +
                    "# و / أو / ليس\n" +
                    "اسم = \"أحمد\"\n" +
                    "إذا اسم ضمن (\"أحمد\", \"ليلى\") و السن >= ١٨:\n" +
                    "    اطبع(\"مقبول\")\n" +
                    "\n" +
                    "# شرط مختصر في سطر واحد\n" +
                    "النتيجة = \"ناجح\" إذا الدرجة >= ٥٠ وإلا \"راسب\"\n" +
                    "اطبع(\"النتيجة:\", النتيجة)\n"
            },
            {"الحلقة لكل",
                    "# الحلقة لكل\n" +
                    "# تكرر على عناصر قائمة أو نطاق من الأرقام\n" +
                    "لكل رقم ضمن نطاق(٥):\n" +
                    "    اطبع(\"الرقم:\", رقم)\n" +
                    "\n" +
                    "# نطاق ببداية ونهاية وقفزة\n" +
                    "لكل رقم ضمن نطاق(١, ١٠, ٢):\n" +
                    "    اطبع(\"فردي:\", رقم)\n" +
                    "\n" +
                    "# ترقيم يعطي المؤشر والقيمة معا\n" +
                    "الأسماء = [\"رنا\", \"سارة\", \"ليان\"]\n" +
                    "لكل المؤشر, الاسم ضمن ترقيم(الأسماء):\n" +
                    "    اطبع(المؤشر, الاسم)\n" +
                    "\n" +
                    "# الحلقة لكل على النصوص\n" +
                    "لكل حرف ضمن \"مرحبا\":\n" +
                    "    اطبع(\"الحرف:\", حرف)\n"
            },
            {"الحلقة طالما",
                    "# الحلقة طالما\n" +
                    "# تعيد التنفيذ ما دام الشرط صحيحا\n" +
                    "ن = ١٠\n" +
                    "المجموع = ٠\n" +
                    "طالما ن > ٠:\n" +
                    "    المجموع += ن\n" +
                    "    ن -= ١\n" +
                    "اطبع(\"مجموع ١ إلى ١٠ =\", المجموع)\n" +
                    "\n" +
                    "# استمر يتجاوز، توقف يخرج من الحلقة\n" +
                    "لكل ن ضمن نطاق(١٠):\n" +
                    "    إذا ن % ٢ == ٠:\n" +
                    "        استمر\n" +
                    "    إذا ن == ٧:\n" +
                    "        توقف\n" +
                    "    اطبع(\"فردي:\", ن)\n"
            },
            {"القوائم",
                    "# القوائم\n" +
                    "# القائمة تحفظ قيما مرتبة، ونصل إليها بالمؤشر (يبدأ من صفر)\n" +
                    "المهام = [\"دراسة\", \"عمل\", \"رياضة\"]\n" +
                    "اطبع(\"الكل:\", المهام)\n" +
                    "اطبع(\"الأول:\", المهام[٠], \"| الأخير:\", المهام[-١])\n" +
                    "اطبع(\"قطعة من ١:\", المهام[١:])\n" +
                    "\n" +
                    "المهام.أضف(\"نوم\")\n" +
                    "المهام.ادرج(١, \"أكل\")\n" +
                    "المهام.رتب()\n" +
                    "اطبع(\"بعد الإضافة والترتيب:\", المهام)\n" +
                    "اطبع(\"العدد:\", طول(المهام), \"| هل يوجد نوم؟\", \"نوم\" ضمن المهام)\n" +
                    "\n" +
                    "# اختصار لكل: قائمة من نطاق مع شرط\n" +
                    "المربعات = [س * س لكل س ضمن نطاق(١٠) إذا س % ٢ == ٠]\n" +
                    "اطبع(\"مربعات الأعداد الزوجية:\", المربعات)\n"
            },
            {"القواميس",
                    "# القواميس\n" +
                    "# القاموس يخزن القيم باسم (مفتاح) بدل مؤشر رقمي\n" +
                    "الطالب = قاموس()\n" +
                    "الطالب[\"الاسم\"] = \"ليلى\"\n" +
                    "الطالب[\"العمر\"] = ١٤\n" +
                    "الطالب[\"المدينة\"] = \"الرياض\"\n" +
                    "اطبع(\"الكل:\", الطالب)\n" +
                    "اطبع(\"الاسم:\", الطالب.اجلب(\"الاسم\"))\n" +
                    "اطبع(\"هاتف غير موجود:\", الطالب.اجلب(\"الهاتف\", \"غير مسجل\"))   # قيمة افتراضية\n" +
                    "الطالب.عين_مبدئي(\"الهاتف\", \"٥٥٥\")\n" +
                    "اطبع(\"بعد عين_مبدئي:\", الطالب.اجلب(\"الهاتف\"))\n" +
                    "\n" +
                    "# المرور على المفاتيح والقيم\n" +
                    "لكل المفتاح, القيمة ضمن الطالب.عناصر():\n" +
                    "    اطبع(\"-\", المفتاح, \"=\", القيمة)\n" +
                    "\n" +
                    "# اختصار لكل: قاموس الأرقام ومربعاتها\n" +
                    "المربعات = {ن: ن * ن لكل ن ضمن نطاق(٥)}\n" +
                    "اطبع(\"مربعات:\", المربعات)\n"
            },
            {"المجموعات والمتسلسلات",
                    "# المجموعات والمتسلسلات\n" +
                    "# المجموعة لا تحفظ تكرارا فتزيله تلقائيا\n" +
                    "الأرقام = مجموعة([١, ٢, ٢, ٣, ٣, ٣])\n" +
                    "اطبع(\"بعد إزالة التكرار:\", الأرقام)\n" +
                    "أخرى = {٣, ٤, ٥}\n" +
                    "اطبع(\"اتحاد:\", الأرقام.اتحاد(أخرى))\n" +
                    "اطبع(\"تقاطع:\", الأرقام.تقاطع(أخرى))\n" +
                    "اطبع(\"فرق:\", الأرقام.فرق(أخرى))\n" +
                    "الأرقام.ضم(٦)\n" +
                    "اطبع(\"بعد ضم ٦:\", الأرقام)\n" +
                    "\n" +
                    "# المتسلسلة ثابتة لا تتغير، ونفككها إلى متغيرات\n" +
                    "الاسم, العمر, المدينة = (\"رنا\", ١٢, \"جدة\")\n" +
                    "اطبع(\"الاسم:\", الاسم, \"| العمر:\", العمر, \"| المدينة:\", المدينة)\n" +
                    "\n" +
                    "# تبادل قيمتين بسهولة\n" +
                    "أ, ب = ٣, ٧\n" +
                    "أ, ب = ب, أ\n" +
                    "اطبع(\"بعد التبادل:\", أ, ب)\n"
            },
            {"الدوال",
                    "# الدوال\n" +
                    "# الدالة تحزّم أوامر لنعيد استعمالها، وتستقبل وسائط\n" +
                    "عرف ترحيب(اسم):\n" +
                    "    أعد \"مرحبا \" + اسم + \"!\"\n" +
                    "\n" +
                    "اطبع(ترحيب(\"ليان\"))\n" +
                    "\n" +
                    "عرف جمع_الكل(*الأرقام):              # عدد الوسائط غير محدد\n" +
                    "    الناتج = ٠\n" +
                    "    لكل ن ضمن الأرقام:\n" +
                    "        الناتج += ن\n" +
                    "    أعد الناتج\n" +
                    "\n" +
                    "اطبع(\"جمع:\", جمع_الكل(١, ٢, ٣, ٤, ٥))\n" +
                    "\n" +
                    "عرف معلومات(اسم, العمر = ٠):         # وسيط افتراضي\n" +
                    "    أعد اسم, العمر                   # إرجاع أكثر من قيمة\n" +
                    "\n" +
                    "الاسم, العمر = معلومات(\"ليلى\", ١٤)\n" +
                    "اطبع(\"الاسم:\", الاسم, \"| العمر:\", العمر)\n" +
                    "اطبع(\"بدون العمر:\", معلومات(\"سارة\"))\n"
            },
            {"دوال مدمجة عالية المستوى",
                    "# دوال مدمجة عالية المستوى\n" +
                    "الأرقام = [٤, ٢, ٦, ٩, ١, ٣]\n" +
                    "اطبع(\"مرتب:\", مرتب(الأرقام))\n" +
                    "اطبع(\"مجموع:\", مجموع(الأرقام), \"| أصغر:\", اصغر(الأرقام), \"| أكبر:\", اكبر(الأرقام))\n" +
                    "\n" +
                    "# لامبدا: دالة في سطر واحد نمررها كوسيطة\n" +
                    "المضاعفة = قائمة(طبق(لامبدا س: س * ٢, الأرقام))\n" +
                    "اطبع(\"طبق مضاعفة:\", المضاعفة)\n" +
                    "الفردية = قائمة(رشح(لامبدا س: س % ٢ == ١, الأرقام))\n" +
                    "اطبع(\"رشح فردية:\", الفردية)\n" +
                    "\n" +
                    "# دمج يزاوج قائمتين، والكل/أي فحوص سريعة\n" +
                    "الأسماء = [\"رنا\", \"سارة\", \"ليان\", \"نور\"]\n" +
                    "الأعمار = [١٢, ١٥, ١٤, ١٦]\n" +
                    "لكل الاسم, العمر ضمن دمج(الأسماء, الأعمار):\n" +
                    "    اطبع(الاسم, \"تعمر\", العمر)\n" +
                    "\n" +
                    "اطبع(\"الكل أكبر من ٠؟\", الكل([س > ٠ لكل س ضمن الأرقام]))\n" +
                    "اطبع(\"أي منها يساوي ٥؟\", أي([س == ٥ لكل س ضمن الأرقام]))\n"
            },
            {"المولدات",
                    "# المولدات\n" +
                    "# المولد يعطي قيمة قيمة، فيوفر الذاكرة بدل إنتاج كل القيم دفعة واحدة\n" +
                    "عرف زوجية(الحد):\n" +
                    "    ن = ٠\n" +
                    "    طالما ن < الحد:\n" +
                    "        انتج ن                # نسلم القيمة ونكمل من النقطة نفسها لاحقا\n" +
                    "        ن += ٢\n" +
                    "\n" +
                    "لكل زوجي ضمن زوجية(١٠):\n" +
                    "    اطبع(\"زوجي:\", زوجي)\n" +
                    "\n" +
                    "الأولون = كرر(زوجية(٢٠))       # كرر يعيد مكرِّرا نأخذ منه بالترتيب\n" +
                    "اطبع(\"أول:\", التالي(الأولون))\n" +
                    "اطبع(\"ثان:\", التالي(الأولون))\n"
            },
            {"الصفوف",
                    "# الصفوف\n" +
                    "# الصف يجمع البيانات (سمات) والسلوك (طرائق) في كائن واحد\n" +
                    "صنف حيوان:\n" +
                    "    عرف __التهيئة__(الذات, الاسم, الصوت):\n" +
                    "        الذات.الاسم = الاسم\n" +
                    "        الذات.الصوت = الصوت\n" +
                    "\n" +
                    "    عرف نطق(الذات):\n" +
                    "        أعد الذات.الاسم + \" يقول: \" + الذات.الصوت + \"!\"\n" +
                    "\n" +
                    "    عرف صوت_مكرر(الذات, مرات):\n" +
                    "        أعد (الذات.الصوت + \" \") * مرات\n" +
                    "\n" +
                    "قط = حيوان(\"قط\", \"مياو\")\n" +
                    "كلب = حيوان(\"كلب\", \"هوو\")\n" +
                    "اطبع(قط.نطق())\n" +
                    "اطبع(كلب.نطق())\n" +
                    "اطبع(كلب.صوت_مكرر(٣))\n" +
                    "اطبع(\"قط من نوع حيوان؟\", مثيل(قط, حيوان))\n"
            },
            {"الاستثناءات",
                    "# الاستثناءات\n" +
                    "# التقط الأخطاء كي لا يتوقف البرنامج فجأة\n" +
                    "جرب:\n" +
                    "    عدد(\"ليست رقما\")\n" +
                    "التقط خطأ_قيمة باسم الخطأ:\n" +
                    "    اطبع(\"خطأ في القيمة:\", الخطأ)\n" +
                    "ختاما:\n" +
                    "    اطبع(\"ختاما يعمل دائما\")\n" +
                    "\n" +
                    "# تحقق: نؤكد شرطا ونرمي رسالة واضحة إن انكسر\n" +
                    "عرف قسمة_آمنة(أ, ب):\n" +
                    "    تحقق ب != ٠, \"ممنوع القسمة على صفر!\"\n" +
                    "    أعد أ / ب\n" +
                    "\n" +
                    "اطبع(\"١٠ ÷ ٤ =\", قسمة_آمنة(١٠, ٤))\n" +
                    "\n" +
                    "جرب:\n" +
                    "    اطبع(قسمة_آمنة(١٠, ٠))\n" +
                    "التقط خطأ باسم الخطأ:\n" +
                    "    اطبع(\"التقطنا:\", الخطأ)\n"
            },
            {"مشروع صغير: قائمة المهام",
                    "# مشروع صغير: قائمة المهام\n" +
                    "# نطبق ما تعلمناه: صف + قائمة + حلقة + شرط\n" +
                    "صنف مدير_المهام:\n" +
                    "    عرف __التهيئة__(الذات):\n" +
                    "        الذات.العناصر = قائمة()\n" +
                    "\n" +
                    "    عرف أضف_مهمة(الذات, الوصف):\n" +
                    "        الذات.العناصر.أضف((الوصف, خاطئ))     # (العنوان، منجز؟)\n" +
                    "\n" +
                    "    عرف انجز(الذات, المؤشر):\n" +
                    "        الوصف, _ = الذات.العناصر[المؤشر]\n" +
                    "        الذات.العناصر[المؤشر] = (الوصف, صحيح)\n" +
                    "\n" +
                    "    عرف عرض(الذات):\n" +
                    "        لكل المؤشر, (الوصف, منجز) ضمن ترقيم(الذات.العناصر):\n" +
                    "            الشارة = \"✓\" إذا منجز وإلا \"○\"\n" +
                    "            اطبع(المؤشر + ١, الشارة, الوصف)\n" +
                    "\n" +
                    "المهام = مدير_المهام()\n" +
                    "المهام.أضف_مهمة(\"شراء الخبز\")\n" +
                    "المهام.أضف_مهمة(\"قراءة كتاب\")\n" +
                    "المهام.أضف_مهمة(\"الركض\")\n" +
                    "المهام.عرض()\n" +
                    "اطبع(\"--- ننجز رقم ٢ ---\")\n" +
                    "المهام.انجز(١)\n" +
                    "المهام.عرض()\n"
            }
    };
    private static final String[][] EDITOR_SNIPPETS = {
            {"سلام", "اطبع(\"السلام عليكم\")"},
            {"اطبع", "اطبع(\"$\")"},
            {"إذا", "إذا $:\n    "},
            {"لكل", "لكل س ضمن نطاق(١٠):\n    $"},
            {"دام", "طالما $:\n    "},
            {"عرف", "عرف $():\n    أعد "},
            {"صنف", "صنف $:\n    عرف __التهيئة__(الذات):\n        الذات.السمة = القيمة"},
            {"جرب", "جرب:\n    $\nالتقط خطأ باسم خطأ:\n    "}
    };

    private static final char SNIP_SEP_NAME = '\u0002';
    private static final char SNIP_SEP_LIST = '\u0001';
    private static final String PREF_SNIPPETS_FIELD = "snippets_full";
    private static final String PREF_SNIPPETS_LEGACY = "snippets";
    private static final String PREF_TABS = "tabs_json";
    private static final String PREF_ACTIVE = "tabs_active";

    /** MIME used for CREATE_DOCUMENT. Not text/plain: Android appends ".txt" for it. */
    private static final String SF_MIME = "application/octet-stream";

    private KodeEditor code;
    private LinearLayout tabsRow;
    private LinearLayout snippetStrip;
    private View findBar;
    private EditText findField;
    private EditText replaceField;

    private View termPane;
    private TextView termOut;
    private EditText termIn;
    private Process termProc;
    private final ArrayDeque<String> termLines = new ArrayDeque<>();
    private final StringBuilder termPartial = new StringBuilder();
    private boolean termUiPending;
    private static final int MAX_OUTPUT_LINES = 400;
    private boolean termRunning;
    private float splitRatio = 0.6f;
    private float dividerDownX;

    private static final String PREF_SPLIT = "split_ratio";

    private final ArrayList<TabDoc> docs = new ArrayList<>();
    private int active = -1;
    private boolean applying;
    private boolean applyingWrap;

    private static class TabDoc {
        String title;
        Uri uri;
        String content = "";
        boolean dirty;
        TextView chip;
        ArrayDeque<String> undo = new ArrayDeque<>();
        ArrayDeque<String> redo = new ArrayDeque<>();
        long undoTime;
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        getWindow().addFlags(WindowManager.LayoutParams.FLAG_FULLSCREEN);
        setContentView(R.layout.main);
        enableImmersive(this);

        code = findViewById(R.id.code);
        tabsRow = findViewById(R.id.tabs);
        snippetStrip = findViewById(R.id.snippet_strip);

        findBar = findViewById(R.id.find_bar);
        findField = findViewById(R.id.find_field);
        replaceField = findViewById(R.id.replace_field);

        findViewById(R.id.tb_undo).setOnClickListener(v -> undo());
        findViewById(R.id.tb_redo).setOnClickListener(v -> redo());
        findViewById(R.id.tb_find).setOnClickListener(v -> showFindBar(false));

        findViewById(R.id.find_next).setOnClickListener(v -> findNext());
        findViewById(R.id.find_prev).setOnClickListener(v -> findPrev());
        findViewById(R.id.find_close).setOnClickListener(v -> hideFindBar());
        findViewById(R.id.replace_one).setOnClickListener(v -> replaceOne());
        findViewById(R.id.replace_all).setOnClickListener(v -> replaceAll());
        findField.setOnEditorActionListener((v, actionId, event) -> {
            if (actionId == EditorInfo.IME_ACTION_SEARCH || actionId == EditorInfo.IME_ACTION_GO
                    || actionId == EditorInfo.IME_ACTION_DONE) {
                findNext();
                return true;
            }
            return false;
        });

        code.addTextChangedListener(new TextWatcher() {
            @Override public void beforeTextChanged(CharSequence s, int a, int b, int c) {}
            @Override public void onTextChanged(CharSequence s, int a, int b, int c) {}
            @Override public void afterTextChanged(Editable s) {
                if (applying || active < 0) return;
                TabDoc d = docs.get(active);
                String before = d.content;
                long now = System.currentTimeMillis();
                if (d.undo.isEmpty() || now - d.undoTime >= 400) {
                    d.undo.push(before);
                    while (d.undo.size() > MAX_UNDO) d.undo.removeLast();
                    d.undoTime = now;
                }
                d.redo.clear();
                d.content = s.toString();
                d.dirty = true;
                refreshChip(d);
                schedulePersist();
            }
        });

        findViewById(R.id.tb_menu).setOnClickListener(this::showMenu);
        findViewById(R.id.fab_run).setOnClickListener(v -> runCurrent());

        termPane = findViewById(R.id.term_pane);
        termOut = findViewById(R.id.term_out);
        termOut.setMovementMethod(new android.text.method.ScrollingMovementMethod());
        termIn = findViewById(R.id.term_in);
        final View divider = findViewById(R.id.split_divider);
        divider.setOnTouchListener((v, ev) -> {
            switch (ev.getActionMasked()) {
                case MotionEvent.ACTION_DOWN:
                    dividerDownX = ev.getRawX();
                    return true;
                case MotionEvent.ACTION_MOVE: {
                    float d = dividerDownX - ev.getRawX();
                    splitRatio = Math.max(0.2f, Math.min(0.8f, splitRatio + d / getResources().getDisplayMetrics().widthPixels));
                    applySplit();
                    getSharedPreferences("shifra", MODE_PRIVATE)
                            .edit().putFloat(PREF_SPLIT, splitRatio).apply();
                    dividerDownX = ev.getRawX();
                    return true;
                }
                default:
                    return true;
            }
        });
        findViewById(R.id.term_send).setOnClickListener(v -> {
            sendTermInput(termIn.getText().toString());
            termIn.setText("");
        });
        termIn.setOnEditorActionListener((v, actionId, event) -> {
            if (actionId == EditorInfo.IME_ACTION_SEND || actionId == EditorInfo.IME_ACTION_GO) {
                sendTermInput(termIn.getText().toString());
                termIn.setText("");
                return true;
            }
            return false;
        });
        configureSplit();

        final View content = findViewById(R.id.content);
        content.setOnApplyWindowInsetsListener((v, insets) -> {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
                int ime = insets.getInsets(WindowInsets.Type.ime()).bottom;
                int basePad = dp(10);
                v.setPadding(basePad, basePad, basePad, basePad + ime);
            }
            return insets;
        });

        buildSnippetChips();
        restoreTabs();
    }

    static void enableImmersive(Activity a) {
        Window w = a.getWindow();
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            w.setDecorFitsSystemWindows(false);
            WindowInsetsController c = w.getInsetsController();
            if (c != null) {
                c.setSystemBarsBehavior(WindowInsetsController.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE);
                c.hide(WindowInsets.Type.navigationBars() | WindowInsets.Type.statusBars());
            }
        } else {
            w.getDecorView().setSystemUiVisibility(
                    View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY
                            | View.SYSTEM_UI_FLAG_FULLSCREEN
                            | View.SYSTEM_UI_FLAG_HIDE_NAVIGATION);
        }
    }

    private static final String JSON_TITLE = "t";
    private static final String JSON_URI = "u";
    private static final String JSON_CONTENT = "c";
    private static final String JSON_DIRTY = "d";

    /** Recreate the open tabs from the last session (survives closing the app). */
    private void restoreTabs() {
        SharedPreferences p = getSharedPreferences("shifra", MODE_PRIVATE);
        String json = p.getString(PREF_TABS, null);
        int activeIdx = p.getInt(PREF_ACTIVE, 0);
        int activePos = -1;
        if (json != null && !json.isEmpty()) {
            try {
                JSONArray arr = new JSONArray(json);
                for (int i = 0; i < arr.length(); i++) {
                    JSONObject o = arr.getJSONObject(i);
                    TabDoc d = new TabDoc();
                    d.title = o.getString(JSON_TITLE);
                    String uri = o.optString(JSON_URI, null);
                    if (uri != null && !uri.isEmpty()) {
                        d.uri = Uri.parse(uri);
                        try {
                            getContentResolver().takePersistableUriPermission(d.uri, FLAG_READ_WRITE);
                        } catch (SecurityException ignored) {
                        }
                    }
                    d.content = o.optString(JSON_CONTENT, "");
                    d.dirty = o.optBoolean(JSON_DIRTY, false);
                    addDoc(d, false);
                    if (i == activeIdx) activePos = docs.size() - 1;
                }
            } catch (JSONException ignored) {
            }
        }
        if (docs.isEmpty()) {
            newTab();
        } else if (activePos >= 0) {
            switchTo(activePos);
        }
    }

    private void persistActive() {
        getSharedPreferences("shifra", MODE_PRIVATE)
                .edit().putInt(PREF_ACTIVE, Math.max(0, active)).apply();
    }

    /** Save all open tabs (title, uri, content, dirty) so nothing is lost on exit. */
    private void persistDocs() {
        try {
            JSONArray arr = new JSONArray();
            for (TabDoc d : docs) {
                JSONObject o = new JSONObject();
                o.put(JSON_TITLE, d.title);
                if (d.uri != null) o.put(JSON_URI, d.uri.toString());
                o.put(JSON_CONTENT, d.content);
                o.put(JSON_DIRTY, d.dirty);
                arr.put(o);
            }
            SharedPreferences.Editor ed = getSharedPreferences("shifra", MODE_PRIVATE).edit();
            ed.putString(PREF_TABS, arr.toString());
            ed.putInt(PREF_ACTIVE, Math.max(0, active));
            ed.apply();
        } catch (JSONException ignored) {
        }
    }

    private final Runnable persistTask = this::persistDocs;
    private long lastPersistTime;

    /**
     * Persist open tabs soon, but throttle to avoid churning the prefs on every
     * keystroke. Applies immediately if the last write was a while ago and
     * otherwise schedules a short follow-up, so a burst of typing followed by a
     * sudden kill still loses at most ~400ms of edits (onPause fully covers a
     * normal close).
     */
    private void schedulePersist() {
        long now = System.currentTimeMillis();
        getWindow().getDecorView().removeCallbacks(persistTask);
        if (now - lastPersistTime >= 400) {
            persistDocs();
        } else {
            getWindow().getDecorView().postDelayed(persistTask, 400);
        }
    }

    @Override
    protected void onPause() {
        super.onPause();
        if (active >= 0 && active < docs.size()) docs.get(active).content = code.getText().toString();
        persistDocs();
        persistActive();
    }

    private void configureSplit() {
        View divider = findViewById(R.id.split_divider);
        boolean wide = getResources().getDisplayMetrics().widthPixels
                > getResources().getDisplayMetrics().heightPixels;
        if (wide) {
            divider.setVisibility(View.VISIBLE);
            termPane.setVisibility(View.VISIBLE);
            splitRatio = getSharedPreferences("shifra", MODE_PRIVATE).getFloat(PREF_SPLIT, 0.6f);
            applySplit();
        } else {
            divider.setVisibility(View.GONE);
            termPane.setVisibility(View.GONE);
        }
    }

    private void applySplit() {
        View host = findViewById(R.id.code_wrap);
        LinearLayout.LayoutParams le = (LinearLayout.LayoutParams) host.getLayoutParams();
        LinearLayout.LayoutParams lt = (LinearLayout.LayoutParams) termPane.getLayoutParams();
        le.weight = splitRatio;
        lt.weight = 1f - splitRatio;
        host.setLayoutParams(le);
        termPane.setLayoutParams(lt);
    }

    @Override
    public void onConfigurationChanged(android.content.res.Configuration newConfig) {
        super.onConfigurationChanged(newConfig);
        configureSplit();
        enableImmersive(this);
    }

    private void buildSnippetChips() {
        for (String[] s : parseSnippets(snippetFieldText(this))) addSnippetChip(s[0], s[1]);
    }

    static String snippetFieldText(Context ctx) {
        SharedPreferences p = ctx.getSharedPreferences("shifra", MODE_PRIVATE);
        String full = p.getString(PREF_SNIPPETS_FIELD, null);
        if (full != null) {
            if (full.indexOf('⏳') >= 0) {
                full = full.replace("⏳", "$");
                p.edit().putString(PREF_SNIPPETS_FIELD, full).apply();
            }
            return full;
        }
        List<String[]> list = new ArrayList<>(Arrays.asList(EDITOR_SNIPPETS));
        String legacy = p.getString(PREF_SNIPPETS_LEGACY, "");
        if (!legacy.isEmpty()) {
            for (String entry : legacy.split(SNIP_SEP_LIST + "")) {
                String[] s = splitSnippet(entry);
                list.add(new String[]{s[0], s[1].replace("⏳", "$")});
            }
        }
        return serializeSnippets(list.toArray(new String[0][]));
    }

    static void saveSnippetFieldText(Context ctx, String text) {
        ctx.getSharedPreferences("shifra", MODE_PRIVATE)
                .edit().putString(PREF_SNIPPETS_FIELD, text).apply();
    }

    static String serializeSnippets(String[][] list) {
        StringBuilder sb = new StringBuilder();
        for (String[] s : list) {
            if (s[0].trim().isEmpty()) continue;
            if (sb.length() > 0) sb.append('\n');
            sb.append(s[0].trim()).append('\n');
            sb.append(s[1]);
            if (!s[1].endsWith("\n")) sb.append('\n');
        }
        return sb.toString();
    }

    static String[][] parseSnippets(String text) {
        List<String[]> list = new ArrayList<>();
        String name = null;
        StringBuilder body = new StringBuilder();
        for (String line : text.split("\n", -1)) {
            if (line.isEmpty()) {
                if (name != null) {
                    while (body.toString().endsWith("\n")) body.setLength(body.length() - 1);
                    list.add(new String[]{name, body.toString()});
                    name = null;
                    body.setLength(0);
                }
            } else if (name == null) {
                name = line.trim();
            } else {
                body.append(line).append('\n');
            }
        }
        if (name != null) {
            while (body.toString().endsWith("\n")) body.setLength(body.length() - 1);
            list.add(new String[]{name, body.toString()});
        }
        return list.toArray(new String[0][]);
    }

    @Override
    protected void onResume() {
        super.onResume();
        SharedPreferences p = getSharedPreferences("shifra", MODE_PRIVATE);
        code.applyFontSize(p.getFloat("font_size", 15f));
        code.applyFontFamily(p.getString("font_family", "monospace"));
        code.setSoftWrap(p.getBoolean("wrap", false));
        snippetStrip.removeAllViews();
        buildSnippetChips();
    }

    private void addSnippetChip(String label, final String text) {
        TextView t = new TextView(this);
        t.setText(label);
        t.setTextSize(12);
        t.setTextColor(getColor(R.color.text));
        t.setGravity(Gravity.CENTER);
        t.setPadding(dp(10), dp(6), dp(10), dp(6));
        LinearLayout.LayoutParams lp =
                new LinearLayout.LayoutParams(LinearLayout.LayoutParams.WRAP_CONTENT, LinearLayout.LayoutParams.WRAP_CONTENT);
        lp.setMarginEnd(dp(6));
        t.setLayoutParams(lp);
        t.setBackgroundResource(R.drawable.tab_chip_inactive);
        t.setOnClickListener(v -> insertSnippetAtCaret(text));
        snippetStrip.addView(t);
    }

    private void insertSnippetAtCaret(String text) {
        int pos = code.getSelectionStart();
        if (pos < 0) pos = code.getText().length();
        int marker = text.indexOf('$');
        String body = marker < 0 ? text : text.replace("$", "");
        if (marker < 0) marker = 0;
        code.insertAtomically(pos, body, pos + marker);
    }

    private void showMenu(View anchor) {
        PopupMenu pm = new PopupMenu(this, anchor);
        Menu m = pm.getMenu();
        m.add(0, MENU_DEMO, 0, R.string.demo);
        m.add(0, MENU_SNIPPET, 1, R.string.snippets);
        m.add(0, MENU_SAVE_SNIP, 2, R.string.save_snippet);
        SubMenu file = m.addSubMenu(0, 0, 3, R.string.menu_file);
        file.add(0, MENU_NEW, 0, R.string.new_file);
        file.add(0, MENU_OPEN, 1, R.string.open);
        file.add(0, MENU_SAVE, 2, R.string.save);
        file.add(0, MENU_SAVE_AS, 3, R.string.save_as);
        file.add(0, MENU_CLOSE, 4, R.string.close_tab);
        SubMenu edit = m.addSubMenu(0, 0, 4, R.string.menu_edit);
        edit.add(0, MENU_REPLACE, 0, R.string.replace);
        edit.add(0, MENU_GOTO, 1, R.string.goto_line);
        edit.add(0, MENU_FIX, 2, R.string.fix_indent);
        m.add(0, MENU_SETTINGS, 5, R.string.menu_settings);
        pm.setOnMenuItemClickListener(item -> {
            handleMenu(item.getItemId());
            return true;
        });
        pm.show();
    }

    private void handleMenu(int itemId) {
        switch (itemId) {
            case MENU_NEW: newTab(); break;
            case MENU_DEMO: chooseExample(); break;
            case MENU_OPEN: openFile(); break;
            case MENU_SAVE: save(docs.get(active)); break;
            case MENU_SAVE_AS: saveAs(docs.get(active)); break;
            case MENU_UNDO: undo(); break;
            case MENU_REDO: redo(); break;
            case MENU_SNIPPET: showSnippets(); break;
            case MENU_SAVE_SNIP: promptSaveSnippet(); break;
            case MENU_FIX: fixIndent(); break;
            case MENU_CLOSE: closeDoc(docs.get(active)); break;
            case MENU_REPLACE: showFindBar(true); break;
            case MENU_GOTO: gotoLine(); break;
            case MENU_SETTINGS: startActivity(new Intent(this, SettingsActivity.class)); break;
            default: break;
        }
    }

    private void newTab() {
        if (!debounce("new")) return;
        TabDoc d = new TabDoc();
        d.title = getString(R.string.unsaved_title) + " " + (docs.size() + 1);
        addDoc(d);
    }

    private void chooseExample() {
        if (!debounce("demo")) return;
        final List<String> names = new ArrayList<>();
        for (String[] s : EXAMPLES) names.add(s[0]);
        new AlertDialog.Builder(this)
                .setTitle(R.string.choose_example)
                .setItems(names.toArray(new String[0]), (dialog, which) -> {
                    TabDoc d = new TabDoc();
                    d.title = EXAMPLES[which][0];
                    d.content = EXAMPLES[which][1];
                    d.dirty = true;
                    addDoc(d);
                })
                .show();
    }

    private long lastDebounceTime;
    private String lastDebounceKey;

    private boolean debounce(String key) {
        long now = System.currentTimeMillis();
        if (key.equals(lastDebounceKey) && now - lastDebounceTime < 400) return false;
        lastDebounceKey = key;
        lastDebounceTime = now;
        return true;
    }

    private void addDoc(final TabDoc d) {
        addDoc(d, true);
    }

    private void addDoc(final TabDoc d, boolean activate) {
        d.chip = buildChip(d);
        tabsRow.addView(d.chip);
        docs.add(d);
        if (activate) {
            switchTo(docs.size() - 1);
            code.requestFocus();
        }
    }

    private void switchTo(int index) {
        if (active >= 0 && active < docs.size()) docs.get(active).content = code.getText().toString();
        active = index;
        applying = true;
        code.setText(docs.get(index).content);
        applying = false;
        code.setSelection(code.length());
        code.scheduleHighlight();
        for (TabDoc d : docs) refreshChip(d);
        persistDocs();
    }

    private void refreshChip(TabDoc d) {
        boolean isActive = docs.get(active) == d;
        d.chip.setText(d.title + (d.dirty ? " ●" : ""));
        d.chip.setTextColor(isActive ? getColor(R.color.text_on_accent) : getColor(R.color.text));
        d.chip.setBackgroundResource(isActive ? R.drawable.tab_chip_active : R.drawable.tab_chip_inactive);
    }

    private TextView buildChip(final TabDoc d) {
        TextView t = new TextView(this);
        t.setTextSize(14);
        t.setGravity(Gravity.CENTER);
        t.setPadding(dp(14), dp(8), dp(14), dp(8));
        FrameLayout.LayoutParams lp =
                new FrameLayout.LayoutParams(FrameLayout.LayoutParams.WRAP_CONTENT, FrameLayout.LayoutParams.WRAP_CONTENT);
        lp.setMarginEnd(dp(6));
        t.setLayoutParams(lp);
        t.setOnClickListener(v -> switchTo(docs.indexOf(d)));
        t.setOnLongClickListener(v -> closeDoc(d));
        return t;
    }

    private boolean closeDoc(TabDoc d) {
        int index = docs.indexOf(d);
        if (index < 0) return true;
        if (docs.get(active) == d) d.content = code.getText().toString();
        if (d.dirty) {
            new AlertDialog.Builder(this)
                    .setTitle(R.string.close_confirm_title)
                    .setMessage(getString(R.string.close_confirm_msg, d.title))
                    .setPositiveButton(R.string.save, (dialog, which) -> {
                        save(d);
                        removeDoc(d);
                    })
                    .setNegativeButton(R.string.discard, (dialog, which) -> removeDoc(d))
                    .setNeutralButton(R.string.cancel, null)
                    .show();
            return true;
        }
        removeDoc(d);
        return true;
    }

    private void removeDoc(TabDoc d) {
        int index = docs.indexOf(d);
        if (index < 0) return;
        docs.remove(index);
        tabsRow.removeView(d.chip);
        if (docs.isEmpty()) {
            newTab();
            persistDocs();
        } else if (active > index) {
            active--;
        } else if (active == index) {
            switchTo(Math.min(index, docs.size() - 1));
        }
        persistDocs();
    }

    private void openFile() {
        Intent i = new Intent(Intent.ACTION_OPEN_DOCUMENT);
        i.addCategory(Intent.CATEGORY_OPENABLE);
        i.setType("*/*");
        startActivityForResult(i, REQ_OPEN);
    }

    private void save(TabDoc d) {
        if (d == null) return;
        if (docs.get(active) == d) d.content = code.getText().toString();
        if (d.uri != null) {
            writeToUri(d, d.uri);
        } else {
            // No uri yet (new tab): always go through the system picker so the
            // file lands somewhere the user can actually find again.
            saveAs(d);
        }
    }

    private void saveAs(final TabDoc d) {
        Intent i = new Intent(Intent.ACTION_CREATE_DOCUMENT);
        i.addCategory(Intent.CATEGORY_OPENABLE);
        i.setType(SF_MIME);
        i.putExtra(Intent.EXTRA_TITLE, sanitizeName(d.title) + ".sf");
        startActivityForResult(i, REQ_SAVE_AS);
    }

    private String sanitizeName(String name) {
        String n = name.replaceAll("[^\\p{L}\\p{N}_\\- ]", "").trim();
        if (n.isEmpty()) n = "script";
        return n;
    }

    @Override
    protected void onActivityResult(int requestCode, int resultCode, Intent data) {
        super.onActivityResult(requestCode, resultCode, data);
        if (resultCode != RESULT_OK || data == null || data.getData() == null) return;
        final Uri uri = data.getData();
        try {
            getContentResolver().takePersistableUriPermission(uri, FLAG_READ_WRITE);
        } catch (SecurityException ignored) {
        }
        if (requestCode == REQ_OPEN) {
            TabDoc d = new TabDoc();
            d.title = displayName(uri);
            d.uri = uri;
            d.content = readUri(uri);
            addDoc(d);
            persistDocs();
            Toast.makeText(this, getString(R.string.open) + ": " + d.title, Toast.LENGTH_SHORT).show();
        } else if (requestCode == REQ_SAVE_AS) {
            TabDoc d = docs.get(active);
            writeToUri(d, uri);
            d.uri = uri;
            d.title = displayName(uri);
            if (d.title.endsWith(".sf.txt")) d.title = d.title.substring(0, d.title.length() - 4);
            d.dirty = false;
            refreshChip(d);
            persistDocs();
        }
    }

    private void writeToUri(TabDoc d, Uri uri) {
        try {
            OutputStream os = getContentResolver().openOutputStream(uri, "w");
            os.write(d.content.getBytes(StandardCharsets.UTF_8));
            os.flush();
            os.close();
            d.uri = uri;
            d.dirty = false;
            refreshChip(d);
            persistDocs();
            Toast.makeText(this, R.string.saved_toast, Toast.LENGTH_SHORT).show();
        } catch (Exception e) {
            Toast.makeText(this, "خطأ: " + e.getMessage(), Toast.LENGTH_LONG).show();
        }
    }

    private String readUri(Uri uri) {
        try {
            BufferedReader r =
                    new BufferedReader(new InputStreamReader(getContentResolver().openInputStream(uri), StandardCharsets.UTF_8));
            return collect(r);
        } catch (Exception e) {
            return "";
        }
    }

    private String collect(BufferedReader r) throws IOException {
        StringBuilder sb = new StringBuilder();
        String line;
        while ((line = r.readLine()) != null) sb.append(line).append('\n');
        r.close();
        return sb.toString();
    }

    private String displayName(Uri uri) {
        try {
            Cursor c = getContentResolver().query(uri, null, null, null, null);
            if (c != null) {
                if (c.moveToFirst()) {
                    int idx = c.getColumnIndex(OpenableColumns.DISPLAY_NAME);
                    if (idx >= 0) return c.getString(idx);
                }
                c.close();
            }
        } catch (Exception ignored) {
        }
        return getString(R.string.open) + " " + (docs.size() + 1);
    }

    private void showSnippets() {
        final List<String> names = new ArrayList<>();
        final List<String> texts = new ArrayList<>();
        for (String[] s : SNIPPETS) {
            names.add(s[0]);
            texts.add(s[1]);
        }
        for (String[] s : parseSnippets(snippetFieldText(this))) {
            names.add(s[0] + " (" + getString(R.string.snippet_custom) + ")");
            texts.add(s[1]);
        }
        new AlertDialog.Builder(this)
                .setTitle(R.string.snippets)
                .setItems(names.toArray(new String[0]), (dialog, which) -> insertSnippet(texts.get(which)))
                .show();
    }

    private void insertSnippet(String text) {
        int pos = code.getSelectionStart();
        if (pos < 0) pos = code.getText().length();
        code.insertAtomically(pos, text + "\n", pos + text.length() + 1);
    }

    private void promptSaveSnippet() {
        int ss = code.getSelectionStart();
        int se = code.getSelectionEnd();
        String body;
        if (ss >= 0 && se > ss) {
            body = code.getText().subSequence(ss, se).toString();
        } else {
            body = code.getText().toString();
        }
        final EditText nameField = new EditText(this);
        nameField.setHint(R.string.snippet_name_hint);
        new AlertDialog.Builder(this)
                .setTitle(R.string.save_snippet)
                .setView(nameField)
                .setPositiveButton(android.R.string.ok, (dialog, which) -> {
                    String name = nameField.getText().toString().trim();
                    if (name.isEmpty()) {
                        Toast.makeText(this, R.string.snippet_name_required, Toast.LENGTH_SHORT).show();
                        return;
                    }
                    saveCustomSnippet(name, body);
                    Toast.makeText(this, R.string.snippet_saved, Toast.LENGTH_SHORT).show();
                })
                .setNegativeButton(android.R.string.cancel, null)
                .show();
    }

    private void saveCustomSnippet(String name, String body) {
        String text = snippetFieldText(this);
        String[][] list = parseSnippets(text);
        List<String[]> merged = new ArrayList<>();
        boolean replaced = false;
        for (String[] s : list) {
            if (s[0].trim().equals(name)) {
                merged.add(new String[]{name, body});
                replaced = true;
            } else {
                merged.add(s);
            }
        }
        if (!replaced) merged.add(new String[]{name, body});
        saveSnippetFieldText(this, serializeSnippets(merged.toArray(new String[0][])));
    }

    private static String[] splitSnippet(String entry) {
        int i = entry.indexOf(SNIP_SEP_NAME);
        if (i < 0) return new String[]{"?", ""};
        return new String[]{entry.substring(0, i), entry.substring(i + 1)};
    }

    private void showFindBar(boolean withReplace) {
        findBar.setVisibility(View.VISIBLE);
        findViewById(R.id.replace_field).setVisibility(withReplace ? View.VISIBLE : View.GONE);
        findField.requestFocus();
        InputMethodManager imm = (InputMethodManager) getSystemService(INPUT_METHOD_SERVICE);
        imm.showSoftInput(findField, InputMethodManager.SHOW_IMPLICIT);
    }

    private void hideFindBar() {
        findBar.setVisibility(View.GONE);
        InputMethodManager imm = (InputMethodManager) getSystemService(INPUT_METHOD_SERVICE);
        imm.hideSoftInputFromWindow(findBar.getWindowToken(), 0);
        code.requestFocus();
    }

    private String findQuery() {
        return findField.getText().toString();
    }

    private void findNext() {
        String q = findQuery();
        if (q.isEmpty()) return;
        String text = code.getText().toString();
        int from = Math.max(0, code.getSelectionEnd());
        if (from > text.length()) from = 0;
        int idx = text.indexOf(q, from);
        if (idx < 0) idx = text.indexOf(q);
        if (idx < 0) {
            Toast.makeText(this, R.string.no_matches, Toast.LENGTH_SHORT).show();
            return;
        }
        code.setSelection(idx, idx + q.length());
    }

    private void findPrev() {
        String q = findQuery();
        if (q.isEmpty()) return;
        String text = code.getText().toString();
        int from = code.getSelectionStart();
        int idx = -1;
        int fromLoop = Math.min(from, text.length());
        for (int i = fromLoop - 1; i >= 0; i--) {
            if (text.regionMatches(i, q, 0, q.length())) {
                idx = i;
                break;
            }
        }
        if (idx < 0) {
            for (int i = text.length() - q.length(); i >= from; i--) {
                if (text.regionMatches(i, q, 0, q.length())) {
                    idx = i;
                    break;
                }
            }
        }
        if (idx < 0) {
            Toast.makeText(this, R.string.no_matches, Toast.LENGTH_SHORT).show();
            return;
        }
        code.setSelection(idx, idx + q.length());
    }

    private void replaceOne() {
        String q = findQuery();
        if (q.isEmpty()) return;
        String text = code.getText().toString();
        int selStart = code.getSelectionStart();
        int selEnd = code.getSelectionEnd();
        if (selStart >= 0 && selEnd > selStart
                && q.equals(text.substring(Math.min(selStart, text.length()), Math.min(selEnd, text.length())))) {
            Editable e = code.getText();
            e.replace(selStart, selEnd, replaceField.getText().toString());
            findNext();
        } else {
            findNext();
        }
    }

    private void replaceAll() {
        String q = findQuery();
        if (q.isEmpty()) return;
        String repl = replaceField.getText().toString();
        int count;
        String text = code.getText().toString();
        int idx = 0;
        count = 0;
        while ((idx = text.indexOf(q, idx)) >= 0) {
            count++;
            idx += q.length();
        }
        if (count == 0) {
            Toast.makeText(this, R.string.no_matches, Toast.LENGTH_SHORT).show();
            return;
        }
        applying = true;
        code.setText(text.replace(q, repl));
        applying = false;
        code.setSelection(code.length());
        code.scheduleHighlight();
        if (docs.get(active) != null) {
            TabDoc d = docs.get(active);
            d.content = code.getText().toString();
            d.dirty = true;
            refreshChip(d);
            schedulePersist();
        }
        Toast.makeText(this, count + " " + getString(R.string.replace_done), Toast.LENGTH_SHORT).show();
    }

    private void gotoLine() {
        if (code.getText() == null) return;
        final int maxLine = code.getLineCount();
        final EditText nf = new EditText(this);
        nf.setInputType(android.text.InputType.TYPE_CLASS_NUMBER);
        nf.setHint("1-" + maxLine);
        new AlertDialog.Builder(this)
                .setTitle(R.string.goto_line)
                .setView(nf)
                .setPositiveButton(android.R.string.ok, (dialog, which) -> {
                    try {
                        int line = Math.max(1, Math.min(maxLine, Integer.parseInt(nf.getText().toString())));
                        int off = code.getLayout().getLineStart(line - 1);
                        code.setSelection(off);
                        code.requestFocus();
                    } catch (Exception ignored) {
                    }
                })
                .setNegativeButton(android.R.string.cancel, null)
                .show();
    }

    private void fixIndent() {
        String t = code.getText().toString();
        String out = t.replace("\t", "    ").replaceAll("(?m)[ \\t]+$", "");
        applying = true;
        code.setText(out);
        applying = false;
        code.setSelection(code.length());
        code.scheduleHighlight();
    }

    private void undo() {
        if (active < 0) return;
        TabDoc d = docs.get(active);
        if (d.undo.isEmpty()) return;
        d.redo.push(code.getText().toString());
        while (d.redo.size() > MAX_UNDO) d.redo.removeLast();
        restoreTextFrom(d.undo.pop());
    }

    private void redo() {
        if (active < 0) return;
        TabDoc d = docs.get(active);
        if (d.redo.isEmpty()) return;
        d.undo.push(code.getText().toString());
        while (d.undo.size() > MAX_UNDO) d.undo.removeLast();
        d.undoTime = 0;
        restoreTextFrom(d.redo.pop());
    }

    private void restoreTextFrom(String text) {
        TabDoc d = docs.get(active);
        applying = true;
        code.setText(text);
        applying = false;
        d.content = text;
        code.setSelection(code.length());
        code.scheduleHighlight();
        refreshChip(d);
    }

    private void runCurrent() {
        final TabDoc d = docs.get(active);
        d.content = code.getText().toString();
        if (d.content.trim().isEmpty()) {
            Toast.makeText(this, R.string.empty_code, Toast.LENGTH_SHORT).show();
            return;
        }
        File script = new File(getCacheDir(), "run.sf");
        try {
            FileOutputStream os = new FileOutputStream(script);
            os.write(d.content.getBytes(StandardCharsets.UTF_8));
            os.flush();
            os.close();
        } catch (IOException e) {
            Toast.makeText(this, "خطأ: " + e.getMessage(), Toast.LENGTH_LONG).show();
            return;
        }
        InputMethodManager imm = (InputMethodManager) getSystemService(INPUT_METHOD_SERVICE);
        imm.hideSoftInputFromWindow(code.getWindowToken(), 0);
        code.clearFocus();
        code.postDelayed(() -> {
            if (termPane.getVisibility() == View.VISIBLE) {
                runInTerminal();
            } else {
                startActivity(buildRunIntent(d));
                overridePendingTransition(0, 0);
            }
        }, 300);
    }

    private void runInTerminal() {
        if (termRunning) {
            Toast.makeText(this, R.string.running, Toast.LENGTH_SHORT).show();
            return;
        }
        termRunning = true;
        termOut.setText("");
        termLines.clear();
        termPartial.setLength(0);
        new Thread(() -> {
            try {
                File script = new File(getCacheDir(), "run.sf");
                File binary = new File(getApplicationInfo().nativeLibraryDir, "libshifra.so");
                Process p = new ProcessBuilder(binary.getAbsolutePath(), script.getAbsolutePath())
                        .directory(getCacheDir())
                        .redirectErrorStream(true)
                        .start();
                termProc = p;
                BufferedInputStream bin = new BufferedInputStream(p.getInputStream(), 8192);
                byte[] buf = new byte[4096];
                int n;
                while ((n = bin.read(buf)) > 0) {
                    synchronized (termLines) {
                        termPartial.append(new String(buf, 0, n, StandardCharsets.UTF_8));
                        int nl;
                        while ((nl = termPartial.indexOf("\n")) >= 0) {
                            String full = termPartial.substring(0, nl);
                            termPartial.delete(0, nl + 1);
                            termLines.addLast(full);
                            while (termLines.size() > MAX_OUTPUT_LINES) termLines.removeFirst();
                        }
                    }
                    flushTermUi();
                }
                synchronized (termLines) {
                    if (termPartial.length() > 0) {
                        termLines.addLast(termPartial.toString());
                        termPartial.setLength(0);
                    }
                }
                bin.close();
                int e = p.waitFor();
                termProc = null;
                final int ec = e;
                if (ec != 0) termCommitLine("[" + getString(R.string.exit_code, ec) + "]");
                runOnUiThread(() -> termRunning = false);
            } catch (Exception ex) {
                final String msg = ex.getMessage();
                runOnUiThread(() -> {
                    termOut.setText("خطأ: " + msg);
                    termRunning = false;
                });
            }
        }).start();
    }

    private void termCommitLine(String line) {
        synchronized (termLines) {
            termLines.addLast(line);
            while (termLines.size() > MAX_OUTPUT_LINES) termLines.removeFirst();
        }
        flushTermUi();
    }

    private void flushTermUi() {
        if (termUiPending) return;
        termUiPending = true;
        runOnUiThread(() -> {
            termUiPending = false;
            StringBuilder s = new StringBuilder();
            synchronized (termLines) {
                for (String l : termLines) s.append(l).append('\n');
                s.append(termPartial);
            }
            termOut.setText(s.toString());
            termOut.post(this::scrollTerm);
        });
    }

    private void scrollTerm() {
        if (termOut.getLayout() == null) return;
        int target = termOut.getLayout().getHeight() + termOut.getCompoundPaddingTop()
                + termOut.getCompoundPaddingBottom() - termOut.getHeight();
        termOut.scrollTo(0, Math.max(0, target));
    }

private void sendTermInput(String text) {
        if (termProc == null || text.isEmpty()) return;
        synchronized (termLines) {
            termPartial.append(text);
            String echo = termPartial.toString();
            termPartial.setLength(0);
            termLines.addLast(echo);
            while (termLines.size() > MAX_OUTPUT_LINES) termLines.removeFirst();
        }
        flushTermUi();
        try {
            OutputStream os = termProc.getOutputStream();
            os.write((text + "\n").getBytes(StandardCharsets.UTF_8));
            os.flush();
        } catch (IOException ignored) {
        }
    }

    private Intent buildRunIntent(TabDoc d) {
        Intent i = new Intent(this, OutputActivity.class);
        i.putExtra(Intent.EXTRA_TEXT, d.content);
        i.putExtra(Intent.EXTRA_TITLE, d.title);
        i.addFlags(Intent.FLAG_ACTIVITY_NO_ANIMATION);
        return i;
    }

    private int dp(int value) {
        return Math.round(getResources().getDisplayMetrics().density * value);
    }
}