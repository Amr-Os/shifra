" Shifra syntax highlighting
" Language: Shifra (Arabic Python)

if exists("b:current_syntax")
  finish
endif

" Comments
syn match shifraComment "#.*$" contains=@Spell

" Strings
syn region shifraString start=+"+ end=+"+ skip=+\\"+ contains=shifraEscape
syn region shifraString start=+'+ end=+'+ skip=+\\'+ contains=shifraEscape
syn region shifraFString start=+f"+ end=+"+ skip=+\\"+ contains=shifraEscape
syn region shifraFString start=+f'+ end=+'+ skip=+\\'+ contains=shifraEscape
syn region shifraRawString start=+r"+ end=+"+ contains=@NoSpell
syn region shifraRawString start=+r'+ end=+'+ contains=@NoSpell

" Escape sequences
syn match shifraEscape +\\[ntr\\0bfa]+ contained
syn match shifraEscape +\\[جطرمسنعت]+ contained

" Numbers
syn match shifraNumber "\<[0-9]\+\>"
syn match shifraFloat "\<[0-9]\+\.[0-9]\+\>"
syn match shifraArabicNum "[٠-٩]\+"
syn match shifraArabicNum "[٠-٩]\+\.[٠-٩]\+"

" Keywords (الكلمات الأساسية) - shapes code structure
syn keyword shifraKeyword
  \ و
  \ باسم
  \ تحقق
  \ غير_متزامن
  \ انتظر
  \ توقف
  \ حالة
  \ صنف
  \ تابع
  \ عرف
  \ احذف
  \ وإلا
  \ التقط
  \ ختاما
  \ لكل
  \ من
  \ عام
  \ إذا
  \ استورد
  \ ضمن
  \ هو
  \ لامبدا
  \ غير_محلي
  \ ليس
  \ أو
  \ تجاوز
  \ ارم
  \ أعد
  \ جرب
  \ نوع
  \ ما_دام
  \ مع
  \ انتج
  \ طابق

" Boolean / None constants (القيم الثابتة)
syn keyword shifraConstant صحيح كاذب عدم
syn keyword shifraConstant غير_منفذ ثلاث_نقاط

" Builtin functions (الدوال الأساسية)
syn keyword shifraBuiltin
  \ اطبع
  \ ادخل
  \ طول
  \ نطاق
  \ قائمة
  \ قاموس
  \ مجموعة
  \ متسلسلة
  \ مجموع
  \ اصغر
  \ اكبر
  \ عدد
  \ عائم
  \ مركب
  \ منطقي
  \ نص
  \ بايت
  \ مصفوفة_بايت
  \ مطلق
  \ الكل
  \ أي
  \ كرر
  \ التالي
  \ مكرر_غير_متزامن
  \ التالي_غير_متزامن
  \ ترقيم
  \ رشح
  \ طبق
  \ شبك
  \ كائن
  \ خاصية
  \ طريقة_الصنف
  \ طريقة_ثابتة
  \ دور
  \ قوة
  \ قسمة_باقية
  \ معكوس
  \ مرتب
  \ نسق
  \ تمثيل
  \ هوية
  \ قيم
  \ نفذ
  \ هاش
  \ مساعدة
  \ افتح
  \ حرف
  \ رمز_الحرف
  \ ثنائي
  \ ثماني
  \ ست_عشري
  \ أسكي
  \ شريحة
  \ دليل
  \ متغيرات
  \ عالميات
  \ محليات
  \ اجمع
  \ نقطة_توقف
  \ عرض_الذاكرة
  \ علوي
  \ اجلب_سمة
  \ عين_سمة
  \ احذف_سمة
  \ هل_له_سمة
  \ هل_مثال_من
  \ هل_صنف_فرعي
  \ هل_قابل_للاستدعاء

" Exceptions (الأخطاء)
syn keyword shifraException
  \ خطأ
  \ خطأ_أساسي
  \ خطأ_حسابي
  \ خطأ_تحقق
  \ خطأ_سمة
  \ خطأ_نظام_التشغيل
  \ خطأ_نهاية_الملف
  \ خطأ_استيراد
  \ خطأ_فهرس
  \ خطأ_مفتاح
  \ خطأ_بحث
  \ خطأ_ذاكرة
  \ خطأ_اسم
  \ خطأ_غير_منفذ
  \ خطأ_فيضان
  \ خطأ_مرجع
  \ خطأ_وقت_التشغيل
  \ خطأ_إزاحة
  \ خطأ_صياغة
  \ خطأ_تبويب
  \ خطأ_نوع
  \ خطأ_قيمة
  \ خطأ_القسمة_على_صفر
  \ خطأ_متغير_محلي
  \ خطأ_مخزن
  \ خطأ_استدعاء_ذاتي
  \ خطأ_وحدة_غير_موجودة
  \ خطأ_ملف_غير_موجود
  \ خطأ_ملف_موجود
  \ خطأ_صلاحية
  \ خطأ_اتصال
  \ خطأ_اتصال_مرفوض
  \ خطأ_اتصال_مقاطع
  \ خطأ_اتصال_معاد
  \ خطأ_أنبوب_مكسور
  \ خطأ_إدخال_محجوب
  \ خطأ_إدخال_إخراج
  \ خطأ_بيئة
  \ خطأ_نقطة_عائمة
  \ خطأ_مقاطع
  \ خطأ_إنه_دليل
  \ خطأ_إنه_ليس_دليلا
  \ خطأ_عملية_غير_موجودة
  \ خطأ_عملية_طفل
  \ خطأ_يونيكود
  \ خطأ_فك_يونيكود
  \ خطأ_ترميز_يونيكود
  \ خطأ_ترجمة_يونيكود
  \ خطأ_نظام
  \ مجموعة_أخطاء
  \ مجموعة_أخطاء_أساسية
  \ توقف_التكرار
  \ توقف_التكرار_غير_متزامن
  \ خروج_النظام
  \ خروج_المولد
  \ انقطاع_لوحة_المفاتيح
  \ تحذير
  \ تحذير_تقادم
  \ تحذير_ترميز
  \ تحذير_مستقبلي
  \ تحذير_استيراد
  \ تحذير_صياغة
  \ تحذير_وقت_التشغيل
  \ تحذير_مستخدم
  \ تحذير_بايت
  \ تحذير_موارد

" Object methods (after a dot)
syn match shifraMethod "\.\zs\(أضف\|امسح\|انسخ\|عد\|مدد\|موقع\|ادرج\|اخرج\|احذف\|عكس\|رتب\)"
syn match shifraMethod "\.\zs\(من_مفاتيح\|اجلب\|عناصر\|مفاتيح\|قيم\|اخرج_آخر\|عين_مبدئي\|دمج\)"
syn match shifraMethod "\.\zs\(ضم\|فرق\|فرق_حدث\|تجاهل\|تقاطع\|تقاطع_حدث\|منفصل\|مجموعة_فرعية\|مجموعة_فوق\|فرق_متماثل\|فرق_متماثل_حدث\|اتحاد\)"
syn match shifraMethod "\.\zs\(كبر\|صغر\|حرف_الأول\|طوى\|وسيط\|رمز\|ينتهي_ب\|يبدأ_ب\|وسع_الجداول\)"
syn match shifraMethod "\.\zs\(ابحث\|ابحث_من_آخر\|موقع_من_آخر\|نسق_من_قاموس\|هل_حروف\|هل_حروف_ورقام\|هل_أسكي\|هل_عشري\|هل_رقم\|هل_رقمي\|هل_معرف\|هل_صغار\|هل_كبار\|هل_قابل_للطباعة\|هل_مسافة\|هل_عناوين\)"
syn match shifraMethod "\.\zs\(اربط\|برر_يسار\|برر_يمين\|جرد\|جرد_يسار\|جرد_يمين\|جهز_ترجمة\|اقتسم\|اقتسم_من_آخر\|احذف_بادئة\|احذف_لاحقة\|استبدل\|قسم\|قسم_من_آخر\|قسم_الأسطر\|بدل_الحالة\|بعناوين\|ترجم\|املأ_اصفار\)"
syn match shifraMethod "\.\zs\(فك_رمز\|من_ست_عشري\|ست_عشري\)"

" Operators
syn match shifraOperator "\(=\|==\|!=\|<=\|>=\|<\|>\|+\|-\|\*\|//\|\*\*\|%\|+=\|-=\|->\)"
syn match shifraOperator "\(:\|\.\)"

" Special
syn keyword shifraSelf self
syn match shifraDecorator "@\w\+"

" Highlighting links
hi def link shifraComment Comment
hi def link shifraString String
hi def link shifraFString String
hi def link shifraRawString String
hi def link shifraEscape SpecialChar
hi def link shifraNumber Number
hi def link shifraFloat Float
hi def link shifraArabicNum Number
hi def link shifraKeyword Keyword
hi def link shifraConstant Boolean
hi def link shifraBuiltin Function
hi def link shifraException Exception
hi def link shifraMethod Function
hi def link shifraOperator Operator
hi def link shifraSelf Identifier
hi def link shifraDecorator PreProc

let b:current_syntax = "shifra"
