# tkinter / turtle support in Shifra — work log

**Goal:** make Python's `tkinter` and `turtle` libraries run inside Shifra
(the Arabic programming language), independent of the Android app.

**Status:** DONE — tkinter and turtle both run inside Shifra on the host (all 4
demo files reach their GUI `mainloop`). See the final STATUS section below. The
Android app (`android/`) was not touched.

---

## What was done

### Build environment discovered
- RustPython already ships a **real `_tkinter` implementation** in
  `crates/stdlib/src/tkinter.rs` (wraps Tcl/Tk 8.6 via `tcl-sys`/`tk-sys`
  bindgen crates from the git repo `arihant2math/tkinter`, tag `v0.2.0`).
- `Lib/tkinter/` and `Lib/turtle.py` already exist in the repo.
- The `tkinter` cargo feature exists: `cargo build --features tkinter`.
- The `tk`/`tcl` crates need **system Tcl/Tk dev packages with pkg-config
  files** (`tk.pc`, `tcl.pc`), which are NOT installed on this machine.
  `sudo apt-get install tk8.6-dev tcl8.6-dev` failed (no sudo TTY).
- Solved by pointing `PKG_CONFIG_PATH` / `LD_LIBRARY_PATH` at an existing
  **conda env** that already has Tcl/Tk 8.6.13:
  ```
  export PKG_CONFIG_PATH=/home/amr/my-mojo-app/my-mojo-app/.pixi/envs/default/lib/pkgconfig
  export LD_LIBRARY_PATH=/home/amr/my-mojo-app/my-mojo-app/.pixi/envs/default/lib
  ```
- **Font problem (later):** the conda `tk` build in that env is the `noxft`
  variant — `libtk8.6.so` links only `libX11`. Tk therefore renders with
  **X server core fonts only** (no fontconfig/Xft): "Helvetica" the raw URW
  Type1 "nimbus sans l", DejaVu/Noto invisible, output tiny + thin/fragmented.
- **Font fix:** built Tcl 8.6.13 + Tk 8.6.13 **from source with Xft** into a
  private prefix `~/.shifra-tk` (configure: `--enable-threads`,
  `--prefix=$HOME/.shifra-tk`, Tk additionally `--with-tcl=$HOME/.shifra-tk/lib`).
  It auto-detected Xft/fontconfig (`checking whether to use xft... yes`).
  The new `libtk8.6.so` links `libXft`, `libfontconfig`, `libfreetype` + `libX11`.
  Sources/tarballs are under `/tmp/tksrc` (tcl8.6.13-src, tk8.6.13-src).
- **To use it**, prepend the prefix to both env vars (order matters — the
  `tk-sys`/`tcl-sys` build scripts do `pkg-config .probe("tk"/"tcl")` and pick
  the first match, and at runtime `LD_LIBRARY_PATH` must resolve our libtk):
  ```
  export SHIFRA_TK=$HOME/.shifra-tk
  export PKG_CONFIG_PATH=$SHIFRA_TK/lib/pkgconfig:/home/amr/my-mojo-app/my-mojo-app/.pixi/envs/default/lib/pkgconfig
  export LD_LIBRARY_PATH=$SHIFRA_TK/lib:/home/amr/my-mojo-app/my-mojo-app/.pixi/envs/default/lib
  ```
- With the Xft build, `font families` includes DejaVu Sans/Noto and the default
  resolves to **Noto Sans 12pt** (TrueType, properly hinted) instead of the
  9pt Type1 "nimbus sans l" + smooth rendering.
- This host Linux has X11 dev libs (`libx11-dev`, `libxft-dev`, etc.) and a
  live display `DISPLAY=:0` — so Tk can actually open windows on the desktop.

### Builds that work (host, x86_64-linux-gnu)
- `cargo build --features tkinter --lib -p rustpython-stdlib` — ✅ compiles.
- `cargo build --features tkinter --bin rustpython` (debug) — ✅ compiles.
- Runtime results with the above env vars:
  - `import tkinter` → ✅ prints `tkinter OK 8.6`
  - `import turtle` → ✅ imports fine
  - `tkinter.Tk()` (window create) → failed: `can't read "tk_version": no such variable`

### Root cause found and partially fixed
- `crates/stdlib/src/tkinter.rs` only ever called `Tcl_Init` (Tcl commands) and
  **never `Tk_Init`** (Tk commands: `winfo`, `tk`, …). So `_loadtk` couldn't
  read `tk_version`.
- Added `Tk_Init` call in the `create()` function, guarded by `args.want_tk`:
  ```
  if args.want_tk {
      let tk_result = unsafe { tk_sys::Tk_Init(interp) };
      if tk_result != tk_sys::TCL_OK as ffi::c_int {
          // read Tcl_GetObjResult -> error string, return vm.new_os_error(...)
      }
  }
  ```
  (returns `Err(vm.new_os_error(msg).into())` on failure).
- Rebuilt successfully (only a benign "unnecessary unsafe block" warning which
  was cleaned up; last build was clean except for one remaining warning from an
  unrelated part of the file).

### Current state after the fix
- `tkinter.Tk()` now gets past `_loadtk` further. New error:
  ```
  RuntimeError: tk.h version (8.6) doesn't match libtk.a version ()
  ```
  So `Tk_Init` now works, but the version-detection step still fails.

---

## Where we stopped

> Superseded — see the final STATUS section. (Historical: the old `_loadtk`
> version-check blocker and how it was diagnosed with a length-ptr bug.)

The blocker is in `Lib/tkinter/__init__.py`, `_loadtk()` (~line 2498–2505):

```python
def _loadtk(self):
    ...
    tk_version = self.tk.getvar('tk_version')
    if tk_version != _tkinter.TK_VERSION:
        raise RuntimeError("tk.h version (%s) doesn't match libtk.a version (%s)"
                           % (_tkinter.TK_VERSION, tk_version))
```

- `tkinter.TK_VERSION` is `"8.6"` (hardcoded in `tkinter.rs` line 47–50).
- `self.tk.getvar('tk_version')` returns the wrong/empty value
  (`"libtk.a version ()"`).
- This suggests either:
  1. The `tkapp.getvar` / `Tcl_GetVar2Ex` path isn't reading Tk's global
     `tk_version` variable correctly, OR
  2. `Tk_Init` isn't fully initializing Tk's global variables (e.g. it needs
     `argv0`/`Tk_MainWindow`/geometry setup), OR
  3. `tkinter.TK_VERSION` should match the actual lib (should be `"8.6"`).

---

## What to do next (resume here)

> Superseded — the diagnosis steps below already threw the actual bug (the
> length-ptr in `unicode_from_object`) and was fixed. Remaining ideas for the
> future are at the end of the final STATUS section.

1. **Diagnose the `tk_version` mismatch.**
   - Print `_tkinter.TK_VERSION` and `self.tk.getvar('tk_version')`
     (`self.tk` is the `_tkinter.tkapp` object).
   - Check whether the condition/variable lives at Tcl global scope and whether
     `getvar` uses the right flags (`TCL_GLOBAL_ONLY`).
   - Test directly from Rust with a small probe: after `Tk_Init`, call
     `Tcl_GetVar2Ex(interp, "tk_version", NULL, TCL_GLOBAL_ONLY)` and see what
     comes back. If empty, Tk didn't create it → compare how CPython's
     `_tkinter` bootstraps (it sets `argv0` from `class_name`, calls
     `Tk_MainWindow`, and initializes Tk app init). The current code sets
     `argv0` before `Tcl_Init`; CPython also generally calls `Tk_GetNumMainWindows`
     and ensures `Tk_Init` runs on the app's `Tcl_Interp`.
   - Possible quick workaround (acceptable for Shifra): instead of comparing a
     real `tk_version` var, make the Python check pass by ensuring the var is
     set, e.g. define `tk_version` global from `_tkinter.TK_VERSION` after
     `Tk_Init` if missing. But prefer fixing the real cause.

2. **Once `Tk()` constructs**, test a minimal window:
   ```
   import tkinter
   r = tkinter.Tk()
   print('window created OK')
   r.mainloop()  # must not hang/hang badly; run with timeout + DISPLAY=:0
   ```

3. **Then test turtle end-to-end** (drawing in a real window):
   ```
   import turtle
   t = turtle.Turtle()   # creates its own Tk root
   t.forward(100)
   t.left(90)
   t.forward(50)
   turtle.done() / t.getscreen().mainloop()
   ```
   Note: `turtle` calls `mainloop()` at import/`done()`, which blocks — run
   under `timeout` and a display.

4. **Wire it into Shifra.**
   - Add Shifra spellings for module/name imports if desired. Current library
     name mappings live in `crates/arabiya/src/modules.rs` (`MODULES`/`NAMES`)
     and must stay in sync with `SHIFRA.md`. `import`/`استورد` already works for
     module names; `turtle` and `tkinter` are English module names so
     `استورد turtle` already works — but consider adding Arabic aliases
     (e.g. `سلحفاة` for turtle) if the user wants fully-Arabic imports.
   - Test the Shifra runner on the host (`.sf` file):
     `cargo run --example arabiya -- file.sf` — BUT note that example file is in
     `examples/to_be_deleted/arabiya.rs` and is NOT auto-discovered as a cargo
     example; it was temporarily built as `arabiya_runner` during testing and the
     `Cargo.toml` entry was reverted. Re-register it or use `rustpython` directly
     to test Shifra syntax in real time.

 5. **Important build caveats to remember**
   - The `tcl-sys`/`tk-sys` build requires `PKG_CONFIG_PATH` pointing at a
     Tcl/Tk pkg-config dir; without it the build fails with "Package tk not found".
   - At runtime the binary needs `LD_LIBRARY_PATH` (or rpath) to reach `libtk8.6.so`
     from that conda env; otherwise dynamic-link errors.
   - `DISPLAY` must be set (here `:0`) to actually open windows; for CI/headless
     use `Xvfb`.
   - Work paused at `createcommand` — resume with step 1.

---

## STATUS / CURRENT BLOCKER (updated)

**DONE — tkinter AND turtle now run end-to-end inside Shifra on the host.**

The `tk_version` empty-string bug (`unicode_from_object` passing a NULL
`lengthPtr`) was fixed earlier. What blocked the rest of `Tk()` was a set of
object-lifetime (refcount) bugs in the Tcl↔Python bridge plus missing `TkApp`
methods. All are fixed. Summary of the fixes:

### Root cause: broken tk-sys refcount wrappers
The generated `Tcl_IncrRefCount`/`Tcl_DecrRefCount` in
`target/debug/build/tk-sys-*/out/custom.rs` copy the struct and never write
`refCount` back. Local wrappers `tcl_incr_refcount`/`tcl_decr_refcount` in
`tkinter.rs` write back and free via `tk_sys::TclFreeObj` — matching the real
macro `if (refCount-- <= 1) TclFreeObj(objPtr)`. (`Tcl_Free(obj)` alone is
wrong: it crashes with "alloc: invalid block" on objects with internal reps.)

### Ownership semantics (all verified against CPython `_tkinter.c`)
- `Tcl_New*Obj` → refcount 0. In `call`: incref each arg → `Tcl_EvalObjv` →
  decref each. Flags = `TCL_EVAL_DIRECT | TCL_EVAL_GLOBAL` (u32 262144|131072).
- `Tcl_NewListObj` **increfs its elements** (list owns them) → do NOT decref
  after; the old decrefs freed list elements and segfaulted `UpdateStringOfList`
  during widget creation.
- `Tcl_SetObjResult` does NOT incref: create the result object at refcount 0 and
  set it directly (CPython `CommandProc` pattern); a pre-incr + post-decr freed
  it prematurely.
- `Tcl_SetVar2Ex` increfs the stored value → do NOT decref after (the variable
  owns it); the old extra decref crashed later `IntVar.get()`.
- Flags: `setvar`/`getvar`/`unsetvar` → `TCL_LEAVE_ERR_MSG`; the global variants
  add `TCL_GLOBAL_ONLY`.

### Methods added / fixed on `TkApp`
- `call`: added CPython's single-tuple unwrap (`if args.len()==1 && first is
  PyTuple → eval the tuple items`) — widget creation passes
  `self.tk.call((widgetName, self._w) + extra + self._options(cnf))`.
- Added `getboolean`, `eval` (`Tcl_EvalEx`), `globalsetvar`, `unsetvar`,
  `globalunsetvar`; `setvar`/`globalsetvar` are 2-arg `(name, value)`.
- `to_tcl_obj` fallback now uses **`str(obj)`** not `repr(obj)` — matches CPython
  `AsObj`; widget objects stringify to their window path via `Misc.__str__`,
  fixing turtle's `bad window path name "<turtle.ScrolledCanvas object ...>"`.
- `mainloop` signature: `Option<i32>` → `OptionalArg<i32>` (RustPython's pymethod
  macro doesn't treat `Option` as optional → "expected 1 argument, got 0").
- Callbacks: `createcommand`, `IntVar`, `Button.invoke()`, `after`+`update`
  all verified.

### Shifra runner
`examples/to_be_deleted/arabiya.rs` (`arabiya_runner`) now prints real
tracebacks (`traceback.format_exception`). Earlier "PyBaseException" output was
just a Debug-print of the exception ref. The demo's real failure — `IndexError:
list index out of range` at `Lib/tkinter/__init__.py:2479`
(`os.path.basename(sys.argv[0])`) — was fixed by setting `sys.argv = [path]`
(`vm.ctx.new_list(...)`) in the runner before running the code.

### All 4 Shifra demos verified on host (`DISPLAY=:0`, `arabiya_runner`)
`نافذة_ترحيب.sf`, `عداد_نقرات.sf` (tkinter) and `مربع_سلحفاة.sf`,
`نجمة_سلحفاة.sf` (turtle) all reach their `mainloop` with **no traceback**
(they block in the event loop — GUI windows; kill with `timeout`).

### Verified build/run recipe (used throughout)
```bash
# Xft-enabled Tcl/Tk built from source into ~/.shifra-tk (see build-env section).
# Prefix order matters: our tk.pc/tcl.pc must come BEFORE the pixi env's noxft ones.
export SHIFRA_TK=$HOME/.shifra-tk
export PKG_CONFIG_PATH=$SHIFRA_TK/lib/pkgconfig:/home/amr/my-mojo-app/my-mojo-app/.pixi/envs/default/lib/pkgconfig
export LD_LIBRARY_PATH=$SHIFRA_TK/lib:/home/amr/my-mojo-app/my-mojo-app/.pixi/envs/default/lib
cargo build --features tkinter --bin rustpython
cargo build --features tkinter --example arabiya_runner
DISPLAY=:0 timeout 7 ./target/debug/examples/arabiya_runner example_projects/shifra_tk/<file>.sf
DISPLAY=:0 ./target/debug/rustpython -c "import tkinter; r=tkinter.Tk(); r.update(); r.destroy()"
```

---

## Files touched (for this task)
- `crates/stdlib/src/tkinter.rs` — `Tk_Init`; fixed `unicode_from_object`
  length-ptr bug; refcount wrappers (`tcl_incr/decr_refcount` via `TclFreeObj`);
  call/flags; single-tuple unwrap; `NewListObj` ownership; `SetObjResult` pacing;
  `SetVar2Ex` ownership; `getboolean`/`eval`/`globalsetvar`/`unsetvar`/
  `globalunsetvar`; 2-arg `setvar`/`globalsetvar`; `str()` fallback; `mainloop`
  `OptionalArg<i32>`.
- `crates/stdlib/src/tkinter.rs` (fonts) — Tk's stock default (9pt "Helvetica")
  is too small even with a good font stack, so after `Tk_Init` we
  `font configure` the named Tcl fonts (`TkDefaultFont`, `TkTextFont`,
  `TkMenuFont`, `TkHeadingFont`, `TkCaptionFont`, `TkTooltipFont`, `TkIconFont`)
  to size 12pt while **keeping the family Tk resolves** (don't request a specific
  fontconfig family — if it's unavailable Tk falls back to the ugly bitmap
  `fixed` font). With the Xft-enabled Tk the default now renders as
  **Noto Sans 12pt** (TrueType, crisp + proper hinting).
- `crates/stdlib/src/tkinter.rs` (**RTL / Arabic shaping**) — Tk on X11 has no
  bidi or Arabic shaping: it would draw Arabic left-to-right in codepoint order
  (reversed + disconnected). Fix is a two-step pre-pass at the Python→Tcl
  boundary (`to_tcl_obj`), applied only to strings that contain RTL/Arabic
  content (`contains_rtl`), and NOT to Tel var names (`varname_converter`):
  1. **Arabic presentation shaping** is implemented directly in Rust
     (`mod rtl` → `shape_arabic`): a joining-class table
     (`RTL_JOIN`/`DUAL_JOIN` per Unicode Joining_Type) picks
     isolated/initial/medial/final presentation forms (U+FE70–FEFE) by joining
     context, and collapses ل+أ/إ/ا/آ into the lam-alef ligatures FEFB/FEFC.
     Transparent diacritics are skipped when scanning neighbours and preserved
     in the output. ZWNJ (0x200C) is dropped, ZWJ (0x200D) forces joins.
  2. **Bidi reordering to visual order** uses fribidi
     (`fribidi_get_bidi_types`, `fribidi_get_par_embedding_levels_ex` with
     `PAR_ON`, `fribidi_reorder_line` with `FRIBIDI_FLAGS_DEFAULT`) — this is
     the analogue of the `bidi` + `arabic_reshaper` Python packages the user
     previously used. fribidi is deliberately used ONLY for reordering, never
     for `join_arabic`/`shape`: its shaping output proved inconsistent across
     optimization levels in testing, whereas reordering is stable and the
     reorder-only call is the canonical simple API.
  - Expected result stored in Tk for `عداد النقرات` (verified via label
    `cget -text` hex in both debug and release):
    `FE96 FE8E FEAE FED8 FEE8 FEE0 FE8D 0020 FEAA FE8E FEAA FECB` (visual,
    presentation forms). Latin strings are passed through untouched.
  - The presentation-form codepoints are a runtime dependency on system
    `libfribidi.so.0` (via `#[link(name = "fribidi")]`).
- **Gotcha: the `target/release/rustpython` binary is easy to leave stale.**
  A weeks-old release build (no `Tk_Init` fix) made `Tk()` fail with
  `can't read "tk_version": no such variable` and rendered broken Arabic.
  ALWAYS rebuild release before judging behavior:
  `cargo build --release --features tkinter --bin rustpython` (with the
  `SHIFRA_TK` env above). The fresh release runs all 4 `.sf` demos with the
  plain command `./target/release/rustpython <file>.sf` (needs `DISPLAY=:0`;
  event loops exit 124 under `timeout`, which is expected).
- `crates/arabiya/src/{keywords.rs,builtins.rs,methods.rs,modules.rs}` — re-synced
  to `SHIFRA.md` (strict; old spellings removed).
- `editors/vscode-shifra/` — tokenizer sets/snippets/docs re-synced to
  `SHIFRA.md` spellings.
- `examples/to_be_deleted/arabiya.rs` — runner prints tracebacks; sets
  `sys.argv`. Registers `[[example]] arabiya_runner` in `Cargo.toml`.
- Temp `examples/to_be_deleted/translate_dump.rs` + its `Cargo.toml` entry were
  removed after the demos were verified.

---

## Out of scope / do not touch unless asked
- The Android app (`android/`), `libshifra.so`, and anything phone-related.
- This is purely about making tkinter/turtle run inside the Shifra interpreter.
