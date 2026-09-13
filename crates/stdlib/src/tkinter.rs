#![allow(
    clippy::disallowed_methods,
    reason = "tkinter environment setup still uses direct host APIs until later extraction"
)]

// spell-checker:ignore createcommand

pub(crate) use self::_tkinter::module_def;

#[pymodule]
mod _tkinter {
    use rustpython_vm::builtins::PyBaseExceptionRef;
    use rustpython_vm::convert::IntoPyException;
    use rustpython_vm::function::{FuncArgs, OptionalArg, PosArgs};
    use rustpython_vm::types::Constructor;
    use rustpython_vm::{AsObject, Py, PyObjectRef, PyPayload, PyResult, VirtualMachine};

    use rustpython_vm::builtins::{
        PyBool, PyBytes, PyFloat, PyInt, PyList, PyStr, PyStrRef, PyTuple, PyType,
    };
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::{ffi, ptr};

    use crate::builtins::PyTypeRef;
    use rustpython_common::atomic::AtomicBool;
    use rustpython_common::atomic::Ordering;

    #[cfg(windows)]
    fn _get_tcl_lib_path() -> String {
        // TODO: fix packaging
        String::from(r"C:\ActiveTcl\lib")
    }

    /// Tk on X11 has no Unicode bidirectional layout or Arabic shaping: it
    /// draws characters left-to-right in codepoint order, so Arabic text would
    /// appear reversed and broken. Work around it in two deterministic steps:
    /// 1) shape Arabic letters into their contextual presentation forms
    ///    (isolated/initial/medial/final, plus lam-alef ligatures) ourselves,
    /// 2) reorder the result to visual order with fribidi (which Tk then draws
    ///    correctly). On Windows/macOS Tk handles this natively, so this is
    ///    only needed on unix.
    #[cfg(unix)]
    mod rtl {
        use std::ffi::c_int;

        // fribidi constants (see fribidi-flags.h / fribidi-bidi-types.h).
        const PAR_ON: u32 = 64;
        // FRIBIDI_FLAGS_DEFAULT (mirroring | reorder_NSM | remove_specials).
        const REORDER_FLAGS: u32 = 0x0000_0001 | 0x0000_0002 | 0x0004_0000;
        // ZWNJ / ZWJ.
        const ZWNJ: u32 = 0x200C;
        const ZWJ: u32 = 0x200D;

        #[link(name = "fribidi")]
        unsafe extern "C" {
            fn fribidi_get_bidi_types(str: *const u32, len: c_int, bidi_types: *mut u32);
            fn fribidi_get_par_embedding_levels_ex(
                bidi_types: *const u32,
                bracket_types: *const u8,
                len: c_int,
                pbase_dir: *mut u32,
                embedding_levels: *mut i8,
            ) -> i8;
            fn fribidi_reorder_line(
                flags: u32,
                bidi_types: *const u32,
                len: c_int,
                off: c_int,
                base_dir: u32,
                embedding_levels: *mut i8,
                visual_str: *mut u32,
                map: *mut usize,
            ) -> i8;
        }

        fn contains_rtl(s: &str) -> bool {
            s.chars().any(|c| {
                let cp = c as u32;
                (0x0591..=0x08FF).contains(&cp)
                    || (0xFB1D..=0xFDFF).contains(&cp)
                    || (0xFE70..=0xFEFE).contains(&cp)
                    || cp == 0x061C
                    || (0x2066..=0x2069).contains(&cp)
                    || (0x200E..=0x200F).contains(&cp)
            })
        }

        /// Right-joining letters: (base, isolated, final). They never connect
        /// forward, so only the isolated/final pair exists.
        const RTL_JOIN: &[(u32, [u32; 2])] = &[
            (0x0622, [0xFE81, 0xFE82]),
            (0x0623, [0xFE83, 0xFE84]),
            (0x0624, [0xFE85, 0xFE86]),
            (0x0625, [0xFE87, 0xFE88]),
            (0x0627, [0xFE8D, 0xFE8E]), // ا alef
            (0x0629, [0xFE93, 0xFE94]),
            (0x062F, [0xFEA9, 0xFEAA]), // د
            (0x0630, [0xFEAB, 0xFEAC]),
            (0x0631, [0xFEAD, 0xFEAE]), // ر
            (0x0632, [0xFEAF, 0xFEB0]),
            (0x0648, [0xFEED, 0xFEEE]), // و
            (0x0649, [0xFEEF, 0xFEF0]),
            (0x0671, [0xFFB0, 0xFFB1]),
        ];
        /// Dual-joining letters: (base, isolated, final, initial, medial).
        const DUAL_JOIN: &[(u32, [u32; 4])] = &[
            (0x0626, [0xFE89, 0xFE8A, 0xFE8B, 0xFE8C]),
            (0x0628, [0xFE8F, 0xFE90, 0xFE91, 0xFE92]), // ب
            (0x062A, [0xFE95, 0xFE96, 0xFE97, 0xFE98]), // ت
            (0x062B, [0xFE99, 0xFE9A, 0xFE9B, 0xFE9C]),
            (0x062C, [0xFE9D, 0xFE9E, 0xFE9F, 0xFEA0]),
            (0x062D, [0xFEA1, 0xFEA2, 0xFEA3, 0xFEA4]),
            (0x062E, [0xFEA5, 0xFEA6, 0xFEA7, 0xFEA8]),
            (0x0633, [0xFEB1, 0xFEB2, 0xFEB3, 0xFEB4]),
            (0x0634, [0xFEB5, 0xFEB6, 0xFEB7, 0xFEB8]),
            (0x0635, [0xFEB9, 0xFEBA, 0xFEBB, 0xFEBC]),
            (0x0636, [0xFEBD, 0xFEBE, 0xFEBF, 0xFEC0]),
            (0x0637, [0xFEC1, 0xFEC2, 0xFEC3, 0xFEC4]),
            (0x0638, [0xFEC5, 0xFEC6, 0xFEC7, 0xFEC8]),
            (0x0639, [0xFEC9, 0xFECA, 0xFECB, 0xFECC]), // ع
            (0x063A, [0xFECD, 0xFECE, 0xFECF, 0xFED0]),
            (0x0641, [0xFED1, 0xFED2, 0xFED3, 0xFED4]),
            (0x0642, [0xFED5, 0xFED6, 0xFED7, 0xFED8]),
            (0x0643, [0xFED9, 0xFEDA, 0xFEDB, 0xFEDC]),
            (0x0644, [0xFEDD, 0xFEDE, 0xFEDF, 0xFEE0]), // ل
            (0x0645, [0xFEE1, 0xFEE2, 0xFEE3, 0xFEE4]),
            (0x0646, [0xFEE5, 0xFEE6, 0xFEE7, 0xFEE8]),
            (0x0647, [0xFEE9, 0xFEEA, 0xFEEB, 0xFEEC]),
            (0x064A, [0xFEF1, 0xFEF2, 0xFEF3, 0xFEF4]),
        ];
        // Alef variants that form a lam-alef ligature with a preceding ل.
        const LAM_ALEF_ALEPS: [u32; 4] = [0x0622, 0x0623, 0x0625, 0x0627];

        /// Transparent marks (do not participate in joining decisions).
        fn is_transparent(c: u32) -> bool {
            (0x0610..=0x061A).contains(&c)
                || (0x064B..=0x065F).contains(&c)
                || (0x0670..=0x0670).contains(&c)
                || (0x06D6..=0x06ED).contains(&c)
                || (0x08D3..=0x08FF).contains(&c)
        }

        /// 0 = not a joining letter, 1 = right-joining, 2 = dual-joining.
        fn joining(c: u32) -> u8 {
            if RTL_JOIN.iter().any(|(b, _)| *b == c) {
                1
            } else if DUAL_JOIN.iter().any(|(b, _)| *b == c) {
                2
            } else {
                0
            }
        }

        fn nearest_left(chars: &[u32], i: usize) -> Option<usize> {
            (0..i).rev().find(|&k| !is_transparent(chars[k]))
        }

        fn nearest_right(chars: &[u32], i: usize) -> Option<usize> {
            ((i + 1)..chars.len()).find(|&k| !is_transparent(chars[k]))
        }

        /// Whether the boundary between `chars` positions `i-1`/`i` is joined.
        fn left_join(chars: &[u32], i: usize) -> bool {
            match nearest_left(chars, i) {
                None => false,
                Some(p) => chars[p] == ZWJ || joining(chars[p]) != 0,
            }
        }

        /// Whether the boundary between `chars` positions `i`/`i+1` is joined.
        fn right_join(chars: &[u32], i: usize) -> bool {
            match nearest_right(chars, i) {
                None => false,
                Some(n) => chars[n] == ZWJ || joining(chars[n]) != 0,
            }
        }

        /// Shape a logical-order Arabic string into presentation forms,
        /// returning a (possibly shorter) string of visual-ready codepoints.
        fn shape_arabic(chars: &[u32]) -> Vec<u32> {
            let n = chars.len();
            let mut out = Vec::with_capacity(n);
            let mut i = 0;
            while i < n {
                let c = chars[i];
                // Lam-alef ligature: ل followed by an alef variant collapses
                // into FEFB (isolated) / FEFC (final).
                if c == 0x0644 {
                    if let Some(j) = nearest_right(chars, i) {
                        if LAM_ALEF_ALEPS.contains(&chars[j]) {
                            out.push(if left_join(chars, i) { 0xFEFC } else { 0xFEFB });
                            for &m in &chars[(i + 1)..j] {
                                out.push(m);
                            }
                            i = j + 1;
                            continue;
                        }
                    }
                }
                match joining(c) {
                    1 => {
                        let forms = RTL_JOIN.iter().find(|(b, _)| *b == c).unwrap().1;
                        out.push(if left_join(chars, i) {
                            forms[1]
                        } else {
                            forms[0]
                        });
                    }
                    2 => {
                        let forms = DUAL_JOIN.iter().find(|(b, _)| *b == c).unwrap().1;
                        let (l, r) = (left_join(chars, i), right_join(chars, i));
                        out.push(match (l, r) {
                            (true, true) => forms[3],
                            (true, false) => forms[1],
                            (false, true) => forms[2],
                            (false, false) => forms[0],
                        });
                    }
                    _ => {
                        // Hamza and other non-joining characters stay as-is.
                        if c != ZWNJ {
                            out.push(c);
                        }
                    }
                }
                i += 1;
            }
            out
        }

        /// Visual presentation-form string for a bidi-less rendering engine.
        /// Returns `None` when `s` has no RTL/Arabic content.
        pub(super) fn shaped(s: &str) -> Option<String> {
            if !contains_rtl(s) {
                return None;
            }
            let mut buf = shape_arabic(&s.chars().map(|c| c as u32).collect::<Vec<_>>());
            let len = buf.len() as c_int;
            let mut bidi_types = vec![0u32; len as usize];
            let mut levels = vec![0i8; len as usize];
            let mut par: u32 = PAR_ON;
            unsafe {
                fribidi_get_bidi_types(buf.as_ptr(), len, bidi_types.as_mut_ptr());
                fribidi_get_par_embedding_levels_ex(
                    bidi_types.as_ptr(),
                    std::ptr::null(),
                    len,
                    &mut par,
                    levels.as_mut_ptr(),
                );
                fribidi_reorder_line(
                    REORDER_FLAGS,
                    bidi_types.as_ptr(),
                    len,
                    0,
                    par,
                    levels.as_mut_ptr(),
                    buf.as_mut_ptr(),
                    std::ptr::null_mut(),
                );
            }
            Some(buf.iter().filter_map(|&c| char::from_u32(c)).collect())
        }
    }

    /// Return `s` reordered/shaped for display when it contains RTL text,
    /// borrowing it unchanged otherwise.
    #[cfg(unix)]
    fn to_display_str(s: &str) -> std::borrow::Cow<'_, str> {
        match rtl::shaped(s) {
            Some(v) => std::borrow::Cow::Owned(v),
            None => std::borrow::Cow::Borrowed(s),
        }
    }
    #[cfg(not(unix))]
    fn to_display_str(s: &str) -> std::borrow::Cow<'_, str> {
        std::borrow::Cow::Borrowed(s)
    }

    #[pyattr(name = "TclError", once)]
    fn tcl_error(vm: &VirtualMachine) -> PyTypeRef {
        vm.ctx.new_exception_type(
            "_tkinter",
            "TclError",
            Some(vec![vm.ctx.exceptions.exception_type.to_owned()]),
        )
    }

    #[pyattr(name = "TkError", once)]
    fn tk_error(vm: &VirtualMachine) -> PyTypeRef {
        vm.ctx.new_exception_type(
            "_tkinter",
            "TkError",
            Some(vec![vm.ctx.exceptions.exception_type.to_owned()]),
        )
    }

    #[pyattr(once, name = "TK_VERSION")]
    fn tk_version(_vm: &VirtualMachine) -> String {
        format!("{}.{}", 8, 6)
    }

    #[pyattr(once, name = "TCL_VERSION")]
    fn tcl_version(_vm: &VirtualMachine) -> String {
        format!(
            "{}.{}",
            tk_sys::TCL_MAJOR_VERSION,
            tk_sys::TCL_MINOR_VERSION
        )
    }

    #[pyattr]
    #[pyclass(name = "TclObject")]
    #[derive(PyPayload)]
    struct TclObject {
        value: *mut tk_sys::Tcl_Obj,
    }

    impl core::fmt::Debug for TclObject {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "TclObject")
        }
    }

    unsafe impl Send for TclObject {}
    unsafe impl Sync for TclObject {}

    #[pyclass]
    impl TclObject {}

    static QUIT_MAIN_LOOP: AtomicBool = AtomicBool::new(false);
    static ERROR_IN_CMD: AtomicBool = AtomicBool::new(false);

    /// Key under which the per-interpreter `TkClientData` is stored.
    const ASSOC_DATA_KEY: &[u8] = b"rustpython_tkapp\0";

    /// The Tcl object type pointers and interpreter settings that the
    /// Python<->Tcl conversion helpers need. Copied into `TkClientData` so the
    /// Tcl callback can convert arguments without borrowing the `TkApp`.
    #[derive(Clone, Copy)]
    struct TclTypes {
        interpreter: *mut tk_sys::Tcl_Interp,
        want_objects: bool,
        old_boolean_type: *const tk_sys::Tcl_ObjType,
        boolean_type: *const tk_sys::Tcl_ObjType,
        byte_array_type: *const tk_sys::Tcl_ObjType,
        double_type: *const tk_sys::Tcl_ObjType,
        int_type: *const tk_sys::Tcl_ObjType,
        wide_int_type: *const tk_sys::Tcl_ObjType,
        bignum_type: *const tk_sys::Tcl_ObjType,
        list_type: *const tk_sys::Tcl_ObjType,
        string_type: *const tk_sys::Tcl_ObjType,
        utf32_string_type: *const tk_sys::Tcl_ObjType,
        pixel_type: *const tk_sys::Tcl_ObjType,
    }

    /// Per-interpreter state attached with `Tcl_SetAssocData`: the VM needed to
    /// call Python callables and the map of Tcl command name -> Python callable.
    struct TkClientData {
        vm: *const VirtualMachine,
        types: TclTypes,
        commands: RefCell<HashMap<String, PyObjectRef>>,
    }

    unsafe impl Send for TkClientData {}
    unsafe impl Sync for TkClientData {}

    fn get_client_data(interp: *mut tk_sys::Tcl_Interp) -> &'static TkClientData {
        unsafe {
            let cd =
                tk_sys::Tcl_GetAssocData(interp, ASSOC_DATA_KEY.as_ptr() as _, ptr::null_mut());
            debug_assert!(!cd.is_null(), "TkClientData must be set in create()");
            &*(cd as *mut TkClientData)
        }
    }

    /// Correct reference-count helpers. The `tk_sys` crate's generated
    /// `Tcl_IncrRefCount`/`Tcl_DecrRefCount` (see `out/custom.rs`) copy the
    /// `Tcl_Obj` struct and never write `refCount` back, so every decrement
    /// frees the object. These write the field back like the real C macros.
    unsafe fn tcl_incr_refcount(obj: *mut tk_sys::Tcl_Obj) {
        unsafe {
            (*obj).refCount += 1;
        }
    }

    unsafe fn tcl_decr_refcount(obj: *mut tk_sys::Tcl_Obj) {
        unsafe {
            let was = (*obj).refCount;
            (*obj).refCount -= 1;
            if was <= 1 {
                tk_sys::TclFreeObj(obj);
            }
        }
    }

    /// Frees the `Box<TkClientData>` allocated in `create()` when the Tcl
    /// interpreter is destroyed.
    unsafe extern "C" fn tcl_interp_delete(
        client_data: tk_sys::ClientData,
        _interp: *mut tk_sys::Tcl_Interp,
    ) {
        unsafe {
            drop(Box::from_raw(client_data as *mut TkClientData));
        }
    }

    fn set_client_data(interp: *mut tk_sys::Tcl_Interp, cd: *mut TkClientData) {
        unsafe {
            tk_sys::Tcl_SetAssocData(
                interp,
                ASSOC_DATA_KEY.as_ptr() as _,
                Some(tcl_interp_delete),
                cd as _,
            );
        }
    }

    fn unicode_from_string(
        s: *mut ffi::c_char,
        size: usize,
        vm: &VirtualMachine,
    ) -> PyResult<PyObjectRef> {
        // terribly unsafe
        let s = unsafe { std::slice::from_raw_parts(s, size) }
            .to_vec()
            .into_iter()
            .map(|c| c as u8)
            .collect::<Vec<u8>>();
        let s = String::from_utf8(s).unwrap();
        Ok(PyObjectRef::from(vm.ctx.new_str(s)))
    }

    fn unicode_from_object(
        types: &TclTypes,
        obj: *mut tk_sys::Tcl_Obj,
        vm: &VirtualMachine,
    ) -> PyResult<PyObjectRef> {
        let type_ptr = unsafe { (*obj).typePtr };
        if !type_ptr.is_null()
            && !types.interpreter.is_null()
            && (type_ptr == types.string_type || type_ptr == types.utf32_string_type)
        {
            let mut len: ffi::c_int = 0;
            let data = unsafe { tk_sys::Tcl_GetUnicodeFromObj(obj, &mut len) };
            return if size_of::<tk_sys::Tcl_UniChar>() == 2 {
                let v = unsafe { std::slice::from_raw_parts(data as *const u16, len as usize) };
                let s = String::from_utf16(v).unwrap();
                Ok(PyObjectRef::from(vm.ctx.new_str(s)))
            } else {
                let v = unsafe { std::slice::from_raw_parts(data as *const u32, len as usize) };
                let s = widestring::U32String::from_vec(v).to_string_lossy();
                Ok(PyObjectRef::from(vm.ctx.new_str(s)))
            };
        }
        let mut len: ffi::c_int = 0;
        let s = unsafe { tk_sys::Tcl_GetStringFromObj(obj, &mut len) };
        unicode_from_string(s, len as usize, vm)
    }

    fn tcl_obj_to_bool(interp: *mut tk_sys::Tcl_Interp, obj: *mut tk_sys::Tcl_Obj) -> bool {
        let mut res = -1;
        unsafe {
            if tk_sys::Tcl_GetBooleanFromObj(interp, obj, &mut res) != tk_sys::TCL_OK as i32 {
                panic!("Tcl_GetBooleanFromObj failed");
            }
        }
        assert!(res == 0 || res == 1);
        res != 0
    }

    fn tcl_obj_to_pyobject(
        types: &TclTypes,
        obj: *mut tk_sys::Tcl_Obj,
        vm: &VirtualMachine,
    ) -> PyResult<PyObjectRef> {
        let type_ptr = unsafe { (*obj).typePtr };
        if type_ptr.is_null() {
            return unicode_from_object(types, obj, vm);
        }
        if type_ptr == types.old_boolean_type || type_ptr == types.boolean_type {
            return Ok(vm
                .ctx
                .new_bool(tcl_obj_to_bool(types.interpreter, obj))
                .into());
        }
        if type_ptr == types.string_type || type_ptr == types.utf32_string_type {
            return unicode_from_object(types, obj, vm);
        }
        if type_ptr == types.byte_array_type {
            let mut size = 0;
            let data = unsafe { tk_sys::Tcl_GetByteArrayFromObj(obj, &mut size) };
            let bytes = unsafe { std::slice::from_raw_parts(data, size as usize) }.to_vec();
            return Ok(vm.ctx.new_bytes(bytes).into());
        }
        if type_ptr == types.int_type
            || type_ptr == types.wide_int_type
            || type_ptr == types.bignum_type
        {
            let mut v: ffi::c_long = 0;
            if unsafe { tk_sys::Tcl_GetLongFromObj(types.interpreter, obj, &mut v) }
                == tk_sys::TCL_OK as i32
            {
                return Ok(PyObjectRef::from(vm.ctx.new_int(v)));
            }
        }
        if type_ptr == types.double_type {
            let mut v: f64 = 0.0;
            if unsafe { tk_sys::Tcl_GetDoubleFromObj(types.interpreter, obj, &mut v) }
                == tk_sys::TCL_OK as i32
            {
                return Ok(PyObjectRef::from(vm.ctx.new_float(v)));
            }
        }
        if type_ptr == types.list_type {
            let mut objc: ffi::c_int = 0;
            let mut objv: *mut *mut tk_sys::Tcl_Obj = ptr::null_mut();
            if unsafe {
                tk_sys::Tcl_ListObjGetElements(types.interpreter, obj, &mut objc, &mut objv)
            } == tk_sys::TCL_OK as i32
            {
                let mut items = Vec::with_capacity(objc as usize);
                for i in 0..objc as usize {
                    items.push(tcl_obj_to_pyobject(types, unsafe { *objv.add(i) }, vm)?);
                }
                return Ok(vm.ctx.new_list(items).into());
            }
        }
        if type_ptr == types.pixel_type {
            return unicode_from_object(types, obj, vm);
        }
        // Unknown types fall back to a string representation.
        unicode_from_object(types, obj, vm)
    }

    /// Convert a Python object into a `Tcl_Obj` (caller owns the reference).
    fn to_tcl_obj(obj: &PyObjectRef, vm: &VirtualMachine) -> PyResult<*mut tk_sys::Tcl_Obj> {
        if let Some(s) = obj.downcast_ref::<PyStr>() {
            let data = s.to_string_lossy();
            let display = to_display_str(&data);
            return Ok(unsafe {
                tk_sys::Tcl_NewStringObj(display.as_bytes().as_ptr() as _, display.len() as _)
            });
        }
        if let Some(i) = obj.downcast_ref::<PyInt>() {
            let n = i.as_bigint();
            if let Ok(v) = i64::try_from(n) {
                return Ok(unsafe { tk_sys::Tcl_NewWideIntObj(v) });
            }
            // Fall back to a string for very large integers.
            let s = n.to_string();
            return Ok(unsafe { tk_sys::Tcl_NewStringObj(s.as_ptr() as _, s.len() as _) });
        }
        if let Some(f) = obj.downcast_ref::<PyFloat>() {
            let v = f.to_f64();
            return Ok(unsafe { tk_sys::Tcl_NewDoubleObj(v) });
        }
        if obj.downcast_ref::<PyBool>().is_some() {
            let flag = obj.clone().is_true(vm).unwrap_or(false);
            return Ok(unsafe { tk_sys::Tcl_NewBooleanObj(flag as _) });
        }
        if let Some(bytes) = obj.downcast_ref::<PyBytes>() {
            let s = bytes.as_bytes().to_vec();
            return Ok(unsafe { tk_sys::Tcl_NewByteArrayObj(s.as_ptr() as _, s.len() as _) });
        }
        if let Some(tcl_obj) = obj.downcast_ref::<TclObject>() {
            unsafe { tcl_incr_refcount(tcl_obj.value) };
            return Ok(tcl_obj.value);
        }
        // Lists / tuples become a Tcl list (via a string list).
        if let Some(list) = obj.downcast_ref::<PyList>() {
            let items: PyResult<Vec<*mut tk_sys::Tcl_Obj>> = list
                .borrow_vec()
                .iter()
                .map(|x| to_tcl_obj(x, vm))
                .collect();
            let items = items?;
            let list_obj = unsafe { tk_sys::Tcl_NewListObj(items.len() as _, items.as_ptr() as _) };
            return Ok(list_obj);
        }
        if let Some(tuple) = obj.downcast_ref::<PyTuple>() {
            let items: PyResult<Vec<*mut tk_sys::Tcl_Obj>> =
                tuple.iter().map(|x| to_tcl_obj(x, vm)).collect();
            let items = items?;
            let list_obj = unsafe { tk_sys::Tcl_NewListObj(items.len() as _, items.as_ptr() as _) };
            return Ok(list_obj);
        }
        // Generic fallback: use `str()` (as CPython's AsObj does), so widget
        // objects stringify to their window path via `Misc.__str__`.
        let s = obj.str(vm)?;
        let data = s.to_string_lossy();
        let display = to_display_str(&data);
        Ok(unsafe {
            tk_sys::Tcl_NewStringObj(display.as_bytes().as_ptr() as _, display.len() as _)
        })
    }

    /// Call the Python callable associated with a Tcl command.
    unsafe extern "C" fn tcl_command_callback(
        client_data: tk_sys::ClientData,
        interp: *mut tk_sys::Tcl_Interp,
        objc: ffi::c_int,
        objv: *const *mut tk_sys::Tcl_Obj,
    ) -> ffi::c_int {
        unsafe {
            ERROR_IN_CMD.store(false, Ordering::Relaxed);
            let name = client_data as *const ffi::c_char;
            let cd = get_client_data(interp);
            let vm_ref = &*(cd.vm);
            let cmd_name = ffi::CStr::from_ptr(name).to_string_lossy().into_owned();
            let callable = cd.commands.borrow().get(&cmd_name).cloned();
            let args: Result<Vec<PyObjectRef>, _> = (1..objc as usize)
                .map(|i| tcl_obj_to_pyobject(&cd.types, *objv.add(i), vm_ref))
                .collect();
            match (callable, args) {
                (Some(callable), Ok(args)) => {
                    let result = callable.call(FuncArgs::from(args), vm_ref);
                    match result {
                        Ok(ret) => match to_tcl_obj(&ret, vm_ref) {
                            Ok(tcl_ret) => {
                                // CPython's CommandProc pattern: `Tcl_SetObjResult`
                                // does not incref; the interp takes ownership of the
                                // refcount-0 object and will free it when the result
                                // is next replaced.
                                tk_sys::Tcl_SetObjResult(interp, tcl_ret);
                                tk_sys::TCL_OK as ffi::c_int
                            }
                            Err(e) => {
                                set_tcl_error_from(interp, e, vm_ref);
                                ERROR_IN_CMD.store(true, Ordering::Relaxed);
                                tk_sys::TCL_ERROR as ffi::c_int
                            }
                        },
                        Err(e) => {
                            // Surface the Python exception through `tkerror` so the Tcl
                            // event loop keeps going.
                            set_tcl_error_from(interp, e, vm_ref);
                            ERROR_IN_CMD.store(true, Ordering::Relaxed);
                            tk_sys::TCL_ERROR as ffi::c_int
                        }
                    }
                }
                (_, Err(e)) => {
                    set_tcl_error_from(interp, e, vm_ref);
                    ERROR_IN_CMD.store(true, Ordering::Relaxed);
                    tk_sys::TCL_ERROR as ffi::c_int
                }
                (None, _) => {
                    let msg = ffi::CString::new("unknown Tk command callback").unwrap();
                    tk_sys::Tcl_SetObjResult(
                        interp,
                        tk_sys::Tcl_NewStringObj(msg.as_ptr() as _, -1),
                    );
                    tk_sys::TCL_ERROR as ffi::c_int
                }
            }
        }
    }

    /// Free the leaked command-name CString when Tcl deletes a command.
    unsafe extern "C" fn tcl_command_delete(client_data: tk_sys::ClientData) {
        let ptr = client_data as *mut ffi::c_char;
        if !ptr.is_null() {
            unsafe {
                drop(ffi::CString::from_raw(ptr));
            }
        }
    }

    fn set_tcl_error_from(
        interp: *mut tk_sys::Tcl_Interp,
        e: PyBaseExceptionRef,
        vm: &VirtualMachine,
    ) {
        let msg = e
            .as_object()
            .repr(vm)
            .ok()
            .map(|r| format!("{r}"))
            .unwrap_or_default();
        let bytes = msg.as_bytes();
        unsafe {
            tk_sys::Tcl_SetObjResult(
                interp,
                tk_sys::Tcl_NewStringObj(bytes.as_ptr() as _, bytes.len() as _),
            );
        }
    }

    #[expect(dead_code, reason = "TODO: Impl more methods")]
    #[pyattr]
    #[pyclass(name = "tkapp")]
    #[derive(PyPayload)]
    struct TkApp {
        // Tcl_Interp *interp;
        interpreter: *mut tk_sys::Tcl_Interp,
        // int wantobjects;
        want_objects: bool,
        // int threaded; /* True if tcl_platform[threaded] */
        threaded: bool,
        // Tcl_ThreadId thread_id;
        thread_id: Option<tk_sys::Tcl_ThreadId>,
        // int dispatching;
        dispatching: bool,
        // PyObject *trace;
        trace: Option<()>,
        // /* We cannot include tclInt.h, as this is internal.
        //    So we cache interesting types here. */
        old_boolean_type: *const tk_sys::Tcl_ObjType,
        boolean_type: *const tk_sys::Tcl_ObjType,
        byte_array_type: *const tk_sys::Tcl_ObjType,
        double_type: *const tk_sys::Tcl_ObjType,
        int_type: *const tk_sys::Tcl_ObjType,
        wide_int_type: *const tk_sys::Tcl_ObjType,
        bignum_type: *const tk_sys::Tcl_ObjType,
        list_type: *const tk_sys::Tcl_ObjType,
        string_type: *const tk_sys::Tcl_ObjType,
        utf32_string_type: *const tk_sys::Tcl_ObjType,
        pixel_type: *const tk_sys::Tcl_ObjType,
    }

    unsafe impl Send for TkApp {}
    unsafe impl Sync for TkApp {}

    impl core::fmt::Debug for TkApp {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "TkApp")
        }
    }

    #[derive(FromArgs, Debug)]
    struct TkAppConstructorArgs {
        #[pyarg(any)]
        screen_name: Option<String>,
        #[pyarg(any)]
        _base_name: Option<String>,
        #[pyarg(any)]
        class_name: String,
        #[pyarg(any)]
        interactive: i32,
        #[pyarg(any)]
        wantobjects: i32,
        #[pyarg(any, default = true)]
        want_tk: bool,
        #[pyarg(any)]
        sync: i32,
        #[pyarg(any)]
        use_: Option<String>,
    }

    impl Constructor for TkApp {
        type Args = TkAppConstructorArgs;

        fn py_new(_cls: &Py<PyType>, args: Self::Args, vm: &VirtualMachine) -> PyResult<Self> {
            create(args, vm)
        }
    }

    fn varname_converter(obj: PyObjectRef, vm: &VirtualMachine) -> PyResult<String> {
        // if let Ok(bytes) = obj.bytes(vm) {
        //     todo!()
        // }

        // str

        if let Some(varname) = obj.downcast_ref::<PyStr>().map(|s| s.to_string()) {
            return Ok(varname);
        }

        if let Some(tcl_obj) = obj.downcast_ref::<TclObject>() {
            let c_str = unsafe { tk_sys::Tcl_GetString(tcl_obj.value) };
            let bytes = unsafe { ffi::CStr::from_ptr(c_str as _) }.to_bytes();
            let varname = core::str::from_utf8(bytes)
                .map_err(|e| {
                    vm.new_unicode_decode_error(
                        vm.ctx.new_str("utf-8"),
                        vm.ctx.new_bytes(bytes.to_vec()),
                        e.valid_up_to(),
                        e.error_len()
                            .map_or(bytes.len(), |len| e.valid_up_to() + len),
                        vm.ctx.new_str(e.to_string()),
                    )
                })?
                .to_owned();
            return Ok(varname);
        }

        // Construct an error message using the type name (truncated to 50 characters).
        Err(vm.new_type_error(format!(
            "must be str, bytes or Tcl_Obj, not {:.50}",
            obj.class().name(),
        )))
    }

    #[derive(Debug, FromArgs)]
    struct TkAppGetVarArgs {
        #[pyarg(any)]
        name: PyObjectRef,
        #[pyarg(any, default)]
        name2: Option<String>,
    }

    // TODO: DISALLOW_INSTANTIATION
    #[pyclass(with(Constructor))]
    impl TkApp {
        fn types(&self) -> TclTypes {
            TclTypes {
                interpreter: self.interpreter,
                want_objects: self.want_objects,
                old_boolean_type: self.old_boolean_type,
                boolean_type: self.boolean_type,
                byte_array_type: self.byte_array_type,
                double_type: self.double_type,
                int_type: self.int_type,
                wide_int_type: self.wide_int_type,
                bignum_type: self.bignum_type,
                list_type: self.list_type,
                string_type: self.string_type,
                utf32_string_type: self.utf32_string_type,
                pixel_type: self.pixel_type,
            }
        }

        fn tcl_obj_to_pyobject(
            &self,
            obj: *mut tk_sys::Tcl_Obj,
            vm: &VirtualMachine,
        ) -> PyResult<PyObjectRef> {
            tcl_obj_to_pyobject(&self.types(), obj, vm)
        }

        fn unicode_from_object(
            &self,
            obj: *mut tk_sys::Tcl_Obj,
            vm: &VirtualMachine,
        ) -> PyResult<PyObjectRef> {
            unicode_from_object(&self.types(), obj, vm)
        }

        fn var_invoke(&self) {
            if self.threaded && self.thread_id != Some(unsafe { tk_sys::Tcl_GetCurrentThread() }) {
                // TODO: do stuff
            }
        }

        fn inner_getvar(
            &self,
            args: TkAppGetVarArgs,
            flags: u32,
            vm: &VirtualMachine,
        ) -> PyResult<PyObjectRef> {
            let TkAppGetVarArgs { name, name2 } = args;
            // TODO: technically not thread safe
            let name = varname_converter(name, vm)?;

            let name = ffi::CString::new(name).map_err(|e| e.into_pyexception(vm))?;
            let name2 =
                ffi::CString::new(name2.unwrap_or_default()).map_err(|e| e.into_pyexception(vm))?;
            let name2_ptr = if name2.is_empty() {
                ptr::null()
            } else {
                name2.as_ptr()
            };
            let res = unsafe {
                tk_sys::Tcl_GetVar2Ex(
                    self.interpreter,
                    name.as_ptr() as _,
                    name2_ptr as _,
                    flags as _,
                )
            };
            if res.is_null() {
                // TODO: Should be tk error
                unsafe {
                    let err_obj = tk_sys::Tcl_GetObjResult(self.interpreter);
                    let err_str_obj = tk_sys::Tcl_GetString(err_obj);
                    let err_cstr = ffi::CStr::from_ptr(err_str_obj as _);
                    return Err(vm.new_type_error(format!("{err_cstr:?}")));
                }
            }
            let res = if self.want_objects {
                self.tcl_obj_to_pyobject(res, vm)
            } else {
                self.unicode_from_object(res, vm)
            }?;
            Ok(res)
        }

        #[pymethod]
        fn getvar(&self, args: TkAppGetVarArgs, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            self.var_invoke();
            self.inner_getvar(args, tk_sys::TCL_LEAVE_ERR_MSG, vm)
        }

        #[pymethod]
        fn globalgetvar(
            &self,
            args: TkAppGetVarArgs,
            vm: &VirtualMachine,
        ) -> PyResult<PyObjectRef> {
            self.var_invoke();
            self.inner_getvar(
                args,
                tk_sys::TCL_LEAVE_ERR_MSG | tk_sys::TCL_GLOBAL_ONLY,
                vm,
            )
        }

        #[pymethod]
        fn getint(&self, arg: PyObjectRef, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            if let Some(int) = arg.downcast_ref::<PyInt>() {
                return Ok(PyObjectRef::from(vm.ctx.new_int(int.as_bigint().clone())));
            }
            if let Some(obj) = arg.downcast_ref::<TclObject>() {
                let mut v: ffi::c_int = 0;
                if unsafe { tk_sys::Tcl_GetIntFromObj(self.interpreter, obj.value, &mut v) }
                    == tk_sys::TCL_OK as i32
                {
                    return Ok(PyObjectRef::from(vm.ctx.new_int(v)));
                }
            }
            // Fall back to Python `int(x)` for str / other objects.
            let int_type = vm.ctx.types.int_type.to_owned();
            <PyInt as Constructor>::slot_new(int_type, FuncArgs::from(vec![arg]), vm)
                .map(PyObjectRef::from)
        }

        #[pymethod]
        fn getdouble(&self, arg: PyObjectRef, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            if let Some(f) = arg.downcast_ref::<PyFloat>() {
                return Ok(PyObjectRef::from(vm.ctx.new_float(f.to_f64())));
            }
            if let Some(i) = arg.downcast_ref::<PyInt>() {
                if let Ok(v) = i64::try_from(i.as_bigint()) {
                    return Ok(PyObjectRef::from(vm.ctx.new_float(v as f64)));
                }
            }
            if let Some(obj) = arg.downcast_ref::<TclObject>() {
                let mut v: f64 = 0.0;
                if unsafe { tk_sys::Tcl_GetDoubleFromObj(self.interpreter, obj.value, &mut v) }
                    == tk_sys::TCL_OK as i32
                {
                    return Ok(PyObjectRef::from(vm.ctx.new_float(v)));
                }
            }
            let float_type = vm.ctx.types.float_type.to_owned();
            <PyFloat as Constructor>::slot_new(float_type, FuncArgs::from(vec![arg]), vm)
                .map(PyObjectRef::from)
        }

        #[pymethod]
        fn getboolean(&self, arg: PyObjectRef, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            if let Some(obj) = arg.downcast_ref::<TclObject>() {
                let mut v: ffi::c_int = 0;
                if unsafe { tk_sys::Tcl_GetBooleanFromObj(self.interpreter, obj.value, &mut v) }
                    == tk_sys::TCL_OK as i32
                {
                    return Ok(PyObjectRef::from(vm.ctx.new_bool(v != 0)));
                }
            }
            if let Some(s) = arg.downcast_ref::<PyStr>() {
                let tcl = unsafe {
                    tk_sys::Tcl_NewStringObj(s.as_bytes().as_ptr() as _, s.as_bytes().len() as _)
                };
                let mut v: ffi::c_int = 0;
                let ok = unsafe { tk_sys::Tcl_GetBooleanFromObj(self.interpreter, tcl, &mut v) };
                unsafe { tcl_decr_refcount(tcl) };
                if ok == tk_sys::TCL_OK as i32 {
                    return Ok(PyObjectRef::from(vm.ctx.new_bool(v != 0)));
                }
            }
            // Fall back to Python truthiness for bool/int objects.
            Ok(PyObjectRef::from(vm.ctx.new_bool(arg.clone().is_true(vm)?)))
        }

        #[pymethod]
        fn eval(&self, script: PyStrRef, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            let script = script.as_bytes();
            let status = unsafe {
                tk_sys::Tcl_EvalEx(self.interpreter, script.as_ptr() as _, script.len() as _, 0)
            };
            self.tcl_result(status, vm)
        }

        #[pymethod]
        fn call(&self, args: PosArgs<PyObjectRef>, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            // Build a Tcl_Obj array from the Python arguments and evaluate it
            // as a single Tcl command (this is what tkinter's `self.tk.call`
            // maps to).
            // If args is a single tuple, replace with contents of tuple (as
            // CPython's Tkapp_Call does).
            let args: Vec<&PyObjectRef> = args.iter().collect();
            let obj_args: Vec<*mut tk_sys::Tcl_Obj> = if args.len() == 1 {
                if let Some(tuple) = args.first().and_then(|a| a.downcast_ref::<PyTuple>()) {
                    tuple
                        .iter()
                        .map(|a| to_tcl_obj(a, vm))
                        .collect::<PyResult<_>>()?
                } else {
                    args.iter()
                        .map(|a| to_tcl_obj(a, vm))
                        .collect::<PyResult<_>>()?
                }
            } else {
                args.iter()
                    .map(|a| to_tcl_obj(a, vm))
                    .collect::<PyResult<_>>()?
            };
            for it in &obj_args {
                // Tcl_New*Obj constructors return refcount-0 objects; CPython
                // increfs each argument before evaluation and decrefs after.
                unsafe { tcl_incr_refcount(*it) };
            }
            let argv: Vec<*mut tk_sys::Tcl_Obj> = obj_args.iter().copied().collect();
            let status = unsafe {
                tk_sys::Tcl_EvalObjv(
                    self.interpreter,
                    argv.len() as _,
                    argv.as_ptr() as _,
                    (tk_sys::TCL_EVAL_GLOBAL | tk_sys::TCL_EVAL_DIRECT) as i32,
                )
            };
            for it in &obj_args {
                unsafe { tcl_decr_refcount(*it) };
            }
            self.tcl_result(status, vm)
        }

        fn tcl_result(&self, status: i32, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            if status == tk_sys::TCL_OK as i32 {
                let result = unsafe { tk_sys::Tcl_GetObjResult(self.interpreter) };
                self.tcl_obj_to_pyobject(result, vm)
            } else {
                let err = unsafe { tk_sys::Tcl_GetStringResult(self.interpreter) };
                let msg = unsafe { ffi::CStr::from_ptr(err) }
                    .to_string_lossy()
                    .into_owned();
                let tcl_error_type = vm.import("_tkinter", 0)?.get_attr("TclError", vm)?;
                let exc = tcl_error_type.call((msg,), vm)?;
                match exc.downcast::<rustpython_vm::builtins::PyBaseException>() {
                    Ok(e) => Err(e),
                    Err(obj) => {
                        Err(vm
                            .new_type_error(format!("could not raise TclError: {}", obj.repr(vm)?)))
                    }
                }
            }
        }

        #[pymethod]
        fn createcommand(
            &self,
            args: (PyObjectRef, PyObjectRef),
            vm: &VirtualMachine,
        ) -> PyResult<Option<()>> {
            let (name, func) = args;
            let name_str = varname_converter(name, vm)?;
            let client = ffi::CString::new(name_str.clone())
                .map_err(|e| e.into_pyexception(vm))?
                .into_raw();
            let cd = get_client_data(self.interpreter);
            cd.commands.borrow_mut().insert(name_str, func);
            unsafe {
                tk_sys::Tcl_CreateObjCommand(
                    self.interpreter,
                    client,
                    Some(tcl_command_callback),
                    client as tk_sys::ClientData,
                    Some(tcl_command_delete),
                );
            }
            Ok(None)
        }

        #[pymethod]
        fn deletecommand(&self, name: PyObjectRef, vm: &VirtualMachine) -> PyResult<Option<()>> {
            let name_str = varname_converter(name, vm)?;
            let name_c = ffi::CString::new(name_str.clone()).map_err(|e| e.into_pyexception(vm))?;
            let cd = get_client_data(self.interpreter);
            cd.commands.borrow_mut().remove(&name_str);
            unsafe {
                tk_sys::Tcl_DeleteCommand(self.interpreter, name_c.as_ptr() as _);
            }
            Ok(None)
        }

        #[pymethod]
        fn setvar(
            &self,
            args: (PyObjectRef, PyObjectRef),
            vm: &VirtualMachine,
        ) -> PyResult<PyObjectRef> {
            self.inner_setvar(args, tk_sys::TCL_LEAVE_ERR_MSG, vm)
        }

        #[pymethod]
        fn globalsetvar(
            &self,
            args: (PyObjectRef, PyObjectRef),
            vm: &VirtualMachine,
        ) -> PyResult<PyObjectRef> {
            self.inner_setvar(
                args,
                tk_sys::TCL_LEAVE_ERR_MSG | tk_sys::TCL_GLOBAL_ONLY,
                vm,
            )
        }

        fn inner_setvar(
            &self,
            args: (PyObjectRef, PyObjectRef),
            flags: u32,
            vm: &VirtualMachine,
        ) -> PyResult<PyObjectRef> {
            let (name, value) = args;
            let name = varname_converter(name, vm)?;
            let name = ffi::CString::new(name).map_err(|e| e.into_pyexception(vm))?;
            let tcl_value = to_tcl_obj(&value, vm)?;
            let res = unsafe {
                tk_sys::Tcl_SetVar2Ex(
                    self.interpreter,
                    name.as_ptr() as _,
                    ptr::null(),
                    tcl_value,
                    flags as _,
                )
            };
            if res.is_null() {
                unsafe {
                    let err_obj = tk_sys::Tcl_GetObjResult(self.interpreter);
                    let err_str_obj = tk_sys::Tcl_GetString(err_obj);
                    let err_cstr = ffi::CStr::from_ptr(err_str_obj as _);
                    return Err(vm.new_type_error(format!("{err_cstr:?}")));
                }
            }
            self.unicode_from_object(res, vm)
        }

        #[pymethod]
        fn unsetvar(&self, args: PyObjectRef, vm: &VirtualMachine) -> PyResult<()> {
            self.inner_unsetvar(args, tk_sys::TCL_LEAVE_ERR_MSG, vm)
        }

        #[pymethod]
        fn globalunsetvar(&self, args: PyObjectRef, vm: &VirtualMachine) -> PyResult<()> {
            self.inner_unsetvar(
                args,
                tk_sys::TCL_LEAVE_ERR_MSG | tk_sys::TCL_GLOBAL_ONLY,
                vm,
            )
        }

        fn inner_unsetvar(
            &self,
            name: PyObjectRef,
            flags: u32,
            vm: &VirtualMachine,
        ) -> PyResult<()> {
            let name = varname_converter(name, vm)?;
            let name = ffi::CString::new(name).map_err(|e| e.into_pyexception(vm))?;
            let res = unsafe {
                tk_sys::Tcl_UnsetVar2(
                    self.interpreter,
                    name.as_ptr() as _,
                    ptr::null(),
                    flags as _,
                )
            };
            if res == tk_sys::TCL_ERROR as i32 {
                let err = unsafe { tk_sys::Tcl_GetStringResult(self.interpreter) };
                let msg = unsafe { ffi::CStr::from_ptr(err) }
                    .to_string_lossy()
                    .into_owned();
                return Err(vm.new_type_error(msg));
            }
            Ok(())
        }

        #[pymethod]
        fn splitlist(&self, arg: PyObjectRef, vm: &VirtualMachine) -> PyResult<PyObjectRef> {
            // TKinter calls `splitlist` with a Tcl list string; use Tcl_SplitList.
            let tcl_obj = to_tcl_obj(&arg, vm)?;
            let str_repr = unsafe { tk_sys::Tcl_GetString(tcl_obj) };
            let mut argc: ffi::c_int = 0;
            let mut argv: *mut *const ffi::c_char = ptr::null_mut();
            let rc =
                unsafe { tk_sys::Tcl_SplitList(self.interpreter, str_repr, &mut argc, &mut argv) };
            unsafe { tcl_decr_refcount(tcl_obj) };
            if rc != tk_sys::TCL_OK as i32 {
                let err = unsafe { tk_sys::Tcl_GetStringResult(self.interpreter) };
                let msg = unsafe { ffi::CStr::from_ptr(err) }
                    .to_string_lossy()
                    .into_owned();
                return Err(vm.new_type_error(msg));
            }
            let mut items = Vec::with_capacity(argc as usize);
            for i in 0..argc as usize {
                let p = unsafe { *argv.add(i) };
                let s = unsafe { ffi::CStr::from_ptr(p) }
                    .to_string_lossy()
                    .into_owned();
                items.push(PyObjectRef::from(vm.ctx.new_str(s)));
            }
            unsafe { tk_sys::Tcl_Free(argv as *mut ffi::c_char) };
            Ok(PyObjectRef::from(vm.ctx.new_tuple(items)))
        }

        #[pymethod]
        fn settrace(&self, arg: PyObjectRef, vm: &VirtualMachine) -> PyResult<Option<()>> {
            // Store the trace callback so Python exceptions raised in Tcl
            // command callbacks can be surfaced. For now the CPU loop's
            // ERROR_IN_CMD flag together with the registered tkerror handler
            // is sufficient; keep the callback referenced so it isn't GC'd.
            let cd = get_client_data(self.interpreter);
            cd.commands.borrow_mut().insert("__trace__".to_owned(), arg);
            let _ = vm;
            Ok(None)
        }

        // TODO: Fix arguments
        #[pymethod]
        fn mainloop(&self, threshold: OptionalArg<i32>) -> PyResult<()> {
            let threshold = threshold.into_option().unwrap_or(0);
            // self.dispatching = true;
            QUIT_MAIN_LOOP.store(false, Ordering::Relaxed);
            while unsafe { tk_sys::Tk_GetNumMainWindows() } > threshold
                && !QUIT_MAIN_LOOP.load(Ordering::Relaxed)
                && !ERROR_IN_CMD.load(Ordering::Relaxed)
            {
                if self.threaded {
                    unsafe { tk_sys::Tcl_DoOneEvent(0 as _) };
                } else {
                    unsafe { tk_sys::Tcl_DoOneEvent(tk_sys::TCL_DONT_WAIT as _) };
                    // TODO: sleep for the proper time
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
            }
            Ok(())
        }

        #[pymethod]
        fn quit(&self) {
            QUIT_MAIN_LOOP.store(true, Ordering::Relaxed);
        }
    }

    #[pyfunction]
    fn create(args: TkAppConstructorArgs, vm: &VirtualMachine) -> PyResult<TkApp> {
        unsafe {
            let interp = tk_sys::Tcl_CreateInterp();
            let want_objects = args.wantobjects != 0;
            let threaded = !{
                let part1 = String::from("tcl_platform");
                let part2 = String::from("threaded");
                let part1 = ffi::CString::new(part1).map_err(|e| e.into_pyexception(vm))?;
                let part2 = ffi::CString::new(part2).map_err(|e| e.into_pyexception(vm))?;
                let part1_ptr = part1.as_ptr();
                let part2_ptr = part2.as_ptr();
                tk_sys::Tcl_GetVar2Ex(
                    interp,
                    part1_ptr as _,
                    part2_ptr as _,
                    tk_sys::TCL_GLOBAL_ONLY as ffi::c_int,
                )
            }
            .is_null();
            let thread_id = tk_sys::Tcl_GetCurrentThread();
            let dispatching = false;
            let trace = None;
            // TODO: Handle threaded build
            let bool_str = String::from("oldBoolean");
            let old_boolean_type = tk_sys::Tcl_GetObjType(bool_str.as_ptr() as _);
            let (boolean_type, byte_array_type) = {
                let true_str = String::from("true");
                let value = tk_sys::Tcl_NewStringObj(true_str.as_ptr() as _, -1);
                let mut bool_value = 0;
                tk_sys::Tcl_GetBooleanFromObj(interp, value, &mut bool_value);
                let boolean_type = (*value).typePtr;
                tcl_decr_refcount(value);

                let value = tk_sys::Tcl_NewByteArrayObj(&bool_value as *const i32 as *const u8, 1);
                let byte_array_type = (*value).typePtr;
                tcl_decr_refcount(value);
                (boolean_type, byte_array_type)
            };
            let double_str = String::from("double");
            let double_type = tk_sys::Tcl_GetObjType(double_str.as_ptr() as _);
            let int_str = String::from("int");
            let int_type = tk_sys::Tcl_GetObjType(int_str.as_ptr() as _);
            let int_type = if int_type.is_null() {
                let value = tk_sys::Tcl_NewWideIntObj(0);
                let res = (*value).typePtr;
                tcl_decr_refcount(value);
                res
            } else {
                int_type
            };
            let wide_int_str = String::from("wideInt");
            let wide_int_type = tk_sys::Tcl_GetObjType(wide_int_str.as_ptr() as _);
            let bignum_str = String::from("bignum");
            let bignum_type = tk_sys::Tcl_GetObjType(bignum_str.as_ptr() as _);
            let list_str = String::from("list");
            let list_type = tk_sys::Tcl_GetObjType(list_str.as_ptr() as _);
            let string_str = String::from("string");
            let string_type = tk_sys::Tcl_GetObjType(string_str.as_ptr() as _);
            let utf32_str = String::from("utf32");
            let utf32_string_type = tk_sys::Tcl_GetObjType(utf32_str.as_ptr() as _);
            let pixel_str = String::from("pixel");
            let pixel_type = tk_sys::Tcl_GetObjType(pixel_str.as_ptr() as _);

            let exit_str = String::from("exit");
            tk_sys::Tcl_DeleteCommand(interp, exit_str.as_ptr() as _);

            if let Some(name) = args.screen_name {
                tk_sys::Tcl_SetVar2(
                    interp,
                    "env".as_ptr() as _,
                    "DISPLAY".as_ptr() as _,
                    name.as_ptr() as _,
                    tk_sys::TCL_GLOBAL_ONLY as i32,
                );
            }

            if args.interactive != 0 {
                tk_sys::Tcl_SetVar2(
                    interp,
                    "tcl_interactive".as_ptr() as _,
                    ptr::null(),
                    "1".as_ptr() as _,
                    tk_sys::TCL_GLOBAL_ONLY as i32,
                );
            } else {
                tk_sys::Tcl_SetVar2(
                    interp,
                    "tcl_interactive".as_ptr() as _,
                    ptr::null(),
                    "0".as_ptr() as _,
                    tk_sys::TCL_GLOBAL_ONLY as i32,
                );
            }

            let argv0 = args.class_name.clone().to_lowercase();
            tk_sys::Tcl_SetVar2(
                interp,
                "argv0".as_ptr() as _,
                ptr::null(),
                argv0.as_ptr() as _,
                tk_sys::TCL_GLOBAL_ONLY as i32,
            );

            if !args.want_tk {
                tk_sys::Tcl_SetVar2(
                    interp,
                    "_tkinter_skip_tk_init".as_ptr() as _,
                    ptr::null(),
                    "1".as_ptr() as _,
                    tk_sys::TCL_GLOBAL_ONLY as i32,
                );
            }

            if args.sync != 0 || args.use_.is_some() {
                let mut argv = String::with_capacity(4);
                if args.sync != 0 {
                    argv.push_str("-sync");
                }
                if args.use_.is_some() {
                    if args.sync != 0 {
                        argv.push(' ');
                    }
                    argv.push_str("-use ");
                    argv.push_str(&args.use_.unwrap());
                }
                argv.push('\0');
                let argv_ptr = argv.as_ptr() as *mut *mut i8;
                tk_sys::Tcl_SetVar2(
                    interp,
                    "argv".as_ptr() as _,
                    ptr::null(),
                    argv_ptr as *const i8,
                    tk_sys::TCL_GLOBAL_ONLY as i32,
                );
            }

            #[cfg(windows)]
            {
                let ret = std::env::var("TCL_LIBRARY");
                if ret.is_err() {
                    let loc = _get_tcl_lib_path();
                    std::env::set_var("TCL_LIBRARY", loc);
                }
            }

            // Bindgen cannot handle Tcl_AppInit
            if tk_sys::Tcl_Init(interp) != tk_sys::TCL_OK as ffi::c_int {
                todo!("Tcl_Init failed");
            }

            if args.want_tk {
                // Register the Tk commands (`winfo`, `tk`, …) on the interpreter
                // so that tkinter's `_loadtk` can query the Tk version and create
                // windows. Without this, `Tk()` fails with "can't read
                // "tk_version": no such variable".
                let tk_result = tk_sys::Tk_Init(interp);
                if tk_result != tk_sys::TCL_OK as ffi::c_int {
                    let msg = {
                        let err_obj = tk_sys::Tcl_GetObjResult(interp);
                        let err_str_obj = tk_sys::Tcl_GetString(err_obj);
                        ffi::CStr::from_ptr(err_str_obj as _)
                            .to_string_lossy()
                            .into_owned()
                    };
                    return Err(vm.new_os_error(msg).into());
                }
                // Tk's own startup (tkAppInit) defines the version globals that
                // `tkinter._loadtk` sanity-checks (`tk_version`, `tk_patchLevel`,
                // `tcl_version`). When embedding through a plain init path these
                // can be missing even though Tk_Init succeeded (a main window is
                // created), so define them from the compiled-in Tk/Tcl version.
                let tk_version =
                    format!("{}.{}", tk_sys::TK_MAJOR_VERSION, tk_sys::TK_MINOR_VERSION);
                let tk_patch = std::str::from_utf8(&tk_sys::TK_PATCH_LEVEL[..])
                    .map(|s| s.trim_end_matches('\0').to_owned())
                    .unwrap_or_default();
                let tcl_version = std::str::from_utf8(&tk_sys::TCL_VERSION[..])
                    .map(|s| s.trim_end_matches('\0').to_owned())
                    .unwrap_or_default();
                let setvar = |name: &str, value: &str| {
                    tk_sys::Tcl_SetVar2(
                        interp,
                        name.as_ptr() as _,
                        ptr::null(),
                        value.as_ptr() as _,
                        tk_sys::TCL_GLOBAL_ONLY as i32,
                    );
                };
                setvar("tk_version", &tk_version);
                setvar("tk_patchLevel", &tk_patch);
                setvar("tcl_version", &tcl_version);
            }

            // This tk build links only X11 core fonts (no fontconfig/Xft),
            // so TrueType fonts are unavailable to Tk. Its stock default
            // ("Helvetica" 9pt) resolves to the scalable URW Nimbus font but
            // renders ~12px on 96-dpi displays: too small and soft. Bump the
            // named fonts to a readable size while keeping the families Tk
            // can actually resolve (unknown families fall back to the
            // bitmap "fixed" font, which looks worse).
            let font_script = concat!(
                "font configure TkDefaultFont -size 12\n",
                "font configure TkTextFont -size 12\n",
                "font configure TkMenuFont -size 12\n",
                "font configure TkHeadingFont -size 12 -weight bold\n",
                "font configure TkCaptionFont -size 12 -weight bold\n",
                "font configure TkTooltipFont -size 11\n",
                "font configure TkIconFont -size 12",
            );
            tk_sys::Tcl_EvalEx(interp, font_script.as_ptr() as _, font_script.len() as _, 0);

            // Register per-interpreter bookkeeping (VM pointer, Tcl type
            // descriptors, live Python command objects). `createcommand` and
            // the Tcl command callback need these once a Tk script registers a
            // Python callback. Ownership is transferred to Tcl; it is freed by
            // the Tcl_InterpDeleteProc we pass to Tcl_SetAssocData.
            let types = TclTypes {
                interpreter: interp,
                want_objects,
                old_boolean_type,
                boolean_type,
                byte_array_type,
                double_type,
                int_type,
                wide_int_type,
                bignum_type,
                list_type,
                string_type,
                utf32_string_type,
                pixel_type,
            };
            let client_data = Box::new(TkClientData {
                vm: vm as *const VirtualMachine,
                types,
                commands: std::cell::RefCell::new(std::collections::HashMap::new()),
            });
            set_client_data(interp, Box::into_raw(client_data));

            Ok(TkApp {
                interpreter: interp,
                want_objects,
                threaded,
                thread_id: Some(thread_id),
                dispatching,
                trace,
                old_boolean_type,
                boolean_type,
                byte_array_type,
                double_type,
                int_type,
                wide_int_type,
                bignum_type,
                list_type,
                string_type,
                utf32_string_type,
                pixel_type,
            })
        }
    }

    #[pyattr]
    const READABLE: i32 = tk_sys::TCL_READABLE as i32;
    #[pyattr]
    const WRITABLE: i32 = tk_sys::TCL_WRITABLE as i32;
    #[pyattr]
    const EXCEPTION: i32 = tk_sys::TCL_EXCEPTION as i32;

    #[pyattr]
    const TIMER_EVENTS: i32 = tk_sys::TCL_TIMER_EVENTS as i32;
    #[pyattr]
    const IDLE_EVENTS: i32 = tk_sys::TCL_IDLE_EVENTS as i32;
    #[pyattr]
    const DONT_WAIT: i32 = tk_sys::TCL_DONT_WAIT as i32;
}
