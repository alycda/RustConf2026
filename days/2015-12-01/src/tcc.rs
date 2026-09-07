//! Hand-written FFI bindings to libtcc, the Tiny C Compiler's JIT backend.
//!
//! There is no practical reason to solve "sum a list of +1/-1" by generating
//! C source, JIT-compiling it at runtime, and calling the result through a
//! raw function pointer — that's the point. It stress-tests the boundary
//! harder than a `#[no_mangle] extern "C" fn` in a static library would:
//! codegen, a JIT compile step, `TCCState` as an opaque C struct, an
//! error callback bridging into a `String`, and a `*mut c_void` transmuted
//! into a callable `extern "C" fn` pointer, all before the numbers come back.

use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::sync::{Mutex, PoisonError};

/// One JIT at a time, enforced on this side of the boundary.
///
/// libtcc serialises compilation behind a semaphore of its own, but on macOS
/// that semaphore is created lazily on first use with no synchronisation
/// (tcc.h, `wait_sem`: `if (!p->init) p->sem = dispatch_semaphore_create(1)`).
/// Two threads compiling for the first time in a process can both see the
/// uninitialised flag, each create a semaphore, and both walk into the
/// preprocessor at once — `tccpp_new` then dereferences state the other
/// thread is still building. On macos-latest that was a SIGSEGV or SIGTRAP
/// in this crate's parallel tests, in the first `cargo test` after a build
/// and not in hundreds of loops of the built binary, which is what a
/// first-use race looks like. Nothing here needs two JITs in flight, so the
/// wrapper owns the exclusion rather than trusting the library's.
static JIT: Mutex<()> = Mutex::new(());

#[repr(C)]
struct TCCState {
    _private: [u8; 0],
}

const TCC_OUTPUT_MEMORY: c_int = 1;

unsafe extern "C" {
    fn tcc_new() -> *mut TCCState;
    fn tcc_delete(s: *mut TCCState);
    fn tcc_set_error_func(
        s: *mut TCCState,
        error_opaque: *mut c_void,
        error_func: Option<unsafe extern "C" fn(opaque: *mut c_void, msg: *const c_char)>,
    );
    fn tcc_set_output_type(s: *mut TCCState, output_type: c_int) -> c_int;
    fn tcc_compile_string(s: *mut TCCState, buf: *const c_char) -> c_int;
    fn tcc_relocate(s: *mut TCCState) -> c_int;
    fn tcc_get_symbol(s: *mut TCCState, name: *const c_char) -> *mut c_void;
}

/// Appends tcc's error/warning messages to the `String` behind `opaque`.
unsafe extern "C" fn collect_errors(opaque: *mut c_void, msg: *const c_char) {
    unsafe {
        let errors = &mut *opaque.cast::<String>();
        errors.push_str(&CStr::from_ptr(msg).to_string_lossy());
        errors.push('\n');
    }
}

/// JIT-compiles `source` with libtcc, then calls the `int symbol(void)`
/// function it defines and returns its result.
pub fn call_i32_fn(source: &str, symbol: &str) -> miette::Result<i32> {
    // A panic while holding the lock poisons it; the C state it guarded is
    // gone with that thread, so the next caller can safely take over.
    let _jit = JIT.lock().unwrap_or_else(PoisonError::into_inner);
    let mut errors = String::new();

    unsafe {
        let state = tcc_new();
        if state.is_null() {
            return Err(miette::miette!("tcc_new failed to allocate a TCCState"));
        }

        tcc_set_error_func(
            state,
            std::ptr::addr_of_mut!(errors).cast(),
            Some(collect_errors),
        );

        // Output type must be set before any compilation happens.
        tcc_set_output_type(state, TCC_OUTPUT_MEMORY);

        let c_source =
            CString::new(source).map_err(|e| miette::miette!("source has a NUL byte: {e}"))?;
        if tcc_compile_string(state, c_source.as_ptr()) < 0 {
            tcc_delete(state);
            return Err(miette::miette!("tcc_compile_string failed:\n{errors}"));
        }

        if tcc_relocate(state) < 0 {
            tcc_delete(state);
            return Err(miette::miette!("tcc_relocate failed:\n{errors}"));
        }

        let c_symbol = CString::new(symbol).expect("symbol name has no NUL byte");
        let func_ptr = tcc_get_symbol(state, c_symbol.as_ptr());
        if func_ptr.is_null() {
            tcc_delete(state);
            return Err(miette::miette!(
                "symbol `{symbol}` not found in JIT-compiled code"
            ));
        }

        let func: extern "C" fn() -> i32 = std::mem::transmute(func_ptr);
        let result = func();

        tcc_delete(state);
        Ok(result)
    }
}
