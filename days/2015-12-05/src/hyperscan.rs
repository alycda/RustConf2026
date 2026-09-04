//! Hand-written FFI bindings to vectorscan (the maintained fork of Intel's
//! Hyperscan) — a SIMD-vectorized, multi-pattern regex engine built to scan
//! gigabits/sec of network traffic against thousands of signatures at once
//! (it's the matching core inside Suricata/Snort-style intrusion detection).
//! Day 5 is a string-validation puzzle over a few hundred 16-character
//! lines. The *shape* is genuinely right — "does this text contain any of
//! these patterns" is exactly what Hyperscan is for — the *scale* is
//! wrong by several orders of magnitude. See days/2015-12-05/README.md.
//!
//! Two compiled databases, each built from many small literal/near-literal
//! patterns rather than one clever regex, because Hyperscan does not
//! support backreferences — its vectorized model can't represent "whatever
//! character matched earlier" — so "any repeated letter" becomes 26
//! literal patterns (`"aa"`..`"zz"`) instead of one `(.)\1`. That is not a
//! workaround so much as how real Hyperscan signature sets are actually
//! built: many literals over one clever pattern is the library's whole
//! reason to exist.
//!
//! Compiled once and cached (`OnceLock`), not per call: unlike the
//! `tcc` module elsewhere in this workshop (which deliberately
//! recompiles on every call to make a point about JIT overhead),
//! Hyperscan's own docs assume "compile the signature set once at
//! startup, scan every packet against it forever after" — so this module
//! does the same, which is also the realistic-performance version of the
//! same idea.
//!
//! The database is immutable once compiled and safe to share across
//! threads, but the scratch space `hs_scan` writes into as scan-time
//! working memory is not — sharing one scratch buffer across concurrently
//! running `cargo test` threads produced real, intermittent wrong answers
//! (caught by the test suite, not by reasoning about it), so each thread
//! gets its own scratch, lazily allocated from the shared database on
//! first use (`thread_local!`) — the pattern Hyperscan's own docs
//! prescribe for multi-threaded scanning.

use std::ffi::{c_char, c_int, c_uint, c_void};
use std::sync::OnceLock;

#[repr(C)]
struct HsDatabase {
    _private: [u8; 0],
}

#[repr(C)]
struct HsScratch {
    _private: [u8; 0],
}

#[repr(C)]
struct HsCompileError {
    message: *mut c_char,
    expression: c_int,
}

const HS_SUCCESS: c_int = 0;
const HS_MODE_BLOCK: c_uint = 1;

type MatchEventHandler = unsafe extern "C" fn(
    id: c_uint,
    from: u64,
    to: u64,
    flags: c_uint,
    context: *mut c_void,
) -> c_int;

unsafe extern "C" {
    fn hs_compile_multi(
        expressions: *const *const c_char,
        flags: *const c_uint,
        ids: *const c_uint,
        elements: c_uint,
        mode: c_uint,
        platform: *const c_void,
        db: *mut *mut HsDatabase,
        error: *mut *mut HsCompileError,
    ) -> c_int;
    fn hs_free_compile_error(error: *mut HsCompileError) -> c_int;
    fn hs_alloc_scratch(db: *const HsDatabase, scratch: *mut *mut HsScratch) -> c_int;
    fn hs_scan(
        db: *const HsDatabase,
        data: *const c_char,
        length: c_uint,
        flags: c_uint,
        scratch: *mut HsScratch,
        on_event: MatchEventHandler,
        context: *mut c_void,
    ) -> c_int;
}

/// An `hs_database_t` is immutable and safe to scan from many callers once
/// compiled (Hyperscan's own contract); a raw pointer just isn't `Send`/
/// `Sync` on its own, so this asserts what's already true.
struct Database(*mut HsDatabase);
unsafe impl Send for Database {}
unsafe impl Sync for Database {}

/// One scratch buffer belongs to one thread at a time — see the module
/// docs. `Send` so a `thread_local!` can move an instance in, but
/// deliberately not `Sync`: nothing here should be reading the same
/// `Scratch` from two threads at once.
struct Scratch(*mut HsScratch);
unsafe impl Send for Scratch {}

fn compile(patterns: &[String]) -> Database {
    let c_patterns: Vec<std::ffi::CString> = patterns
        .iter()
        .map(|p| std::ffi::CString::new(p.as_str()).expect("pattern has no NUL byte"))
        .collect();
    let pattern_ptrs: Vec<*const c_char> = c_patterns.iter().map(|p| p.as_ptr()).collect();
    let flags = vec![0u32; patterns.len()];
    let ids: Vec<u32> = (0..patterns.len() as u32).collect();

    let mut db: *mut HsDatabase = std::ptr::null_mut();
    let mut error: *mut HsCompileError = std::ptr::null_mut();

    let rc = unsafe {
        hs_compile_multi(
            pattern_ptrs.as_ptr(),
            flags.as_ptr(),
            ids.as_ptr(),
            patterns.len() as c_uint,
            HS_MODE_BLOCK,
            std::ptr::null(),
            &mut db,
            &mut error,
        )
    };

    if rc != HS_SUCCESS {
        let msg = unsafe { std::ffi::CStr::from_ptr((*error).message) }
            .to_string_lossy()
            .into_owned();
        unsafe { hs_free_compile_error(error) };
        panic!("hs_compile_multi failed: {msg}");
    }

    Database(db)
}

fn alloc_scratch(db: &Database) -> Scratch {
    let mut scratch: *mut HsScratch = std::ptr::null_mut();
    unsafe { hs_alloc_scratch(db.0, &mut scratch) };
    Scratch(scratch)
}

fn scan(
    db: &Database,
    scratch: &Scratch,
    line: &str,
    on_event: MatchEventHandler,
    context: *mut c_void,
) {
    unsafe {
        hs_scan(
            db.0,
            line.as_ptr().cast(),
            line.len() as c_uint,
            0,
            scratch.0,
            on_event,
            context,
        );
    }
}

// ---- Part 1: forbidden pairs (ids 0-3), double letters (ids 4-29), vowels (id 30) ----

#[derive(Default)]
struct Part1Match {
    forbidden: bool,
    double_letter: bool,
    vowel_count: u32,
}

unsafe extern "C" fn on_part1_match(
    id: c_uint,
    _from: u64,
    _to: u64,
    _flags: c_uint,
    context: *mut c_void,
) -> c_int {
    let ctx = unsafe { &mut *context.cast::<Part1Match>() };
    match id {
        0..=3 => ctx.forbidden = true,
        4..=29 => ctx.double_letter = true,
        30 => ctx.vowel_count += 1,
        _ => {}
    }
    0
}

fn part1_patterns() -> Vec<String> {
    let mut patterns: Vec<String> = vec!["ab", "cd", "pq", "xy"]
        .into_iter()
        .map(String::from)
        .collect();
    patterns.extend((b'a'..=b'z').map(|c| {
        let c = c as char;
        format!("{c}{c}")
    }));
    patterns.push("[aeiou]".to_string());
    patterns
}

fn part1_db() -> &'static Database {
    static DB: OnceLock<Database> = OnceLock::new();
    DB.get_or_init(|| compile(&part1_patterns()))
}

thread_local! {
    static PART1_SCRATCH: Scratch = alloc_scratch(part1_db());
}

/// Check if a line is nice — the original rules — by scanning it with a
/// Hyperscan database compiled from the puzzle's rules as literal/simple
/// patterns instead of evaluating them in Rust. See the module docs.
pub fn is_nice_via_hyperscan(line: &str) -> bool {
    let mut ctx = Part1Match::default();
    PART1_SCRATCH.with(|scratch| {
        scan(
            part1_db(),
            scratch,
            line,
            on_part1_match,
            std::ptr::addr_of_mut!(ctx).cast(),
        );
    });
    !ctx.forbidden && ctx.double_letter && ctx.vowel_count >= 3
}

// ---- Part 2: pair-repeat (ids 0-675, id = 26*first + second), sandwich (ids 676-701) ----

struct Part2Match {
    /// Earliest/latest end-offset seen for each of the 676 two-letter
    /// pairs. Two occurrences of the same pair are non-overlapping iff
    /// they're at least 2 bytes apart — checking the extremes is enough:
    /// if the earliest and latest are that far apart, a non-overlapping
    /// pair exists, regardless of what's between them.
    pair_min: [i64; 676],
    pair_max: [i64; 676],
    sandwich: bool,
}

impl Default for Part2Match {
    fn default() -> Self {
        Self {
            pair_min: [-1; 676],
            pair_max: [-1; 676],
            sandwich: false,
        }
    }
}

unsafe extern "C" fn on_part2_match(
    id: c_uint,
    _from: u64,
    to: u64,
    _flags: c_uint,
    context: *mut c_void,
) -> c_int {
    let ctx = unsafe { &mut *context.cast::<Part2Match>() };
    let id = id as usize;
    if id < 676 {
        let end = to as i64;
        if ctx.pair_min[id] < 0 {
            ctx.pair_min[id] = end;
        }
        ctx.pair_max[id] = end;
    } else {
        ctx.sandwich = true;
    }
    0
}

fn part2_patterns() -> Vec<String> {
    let mut patterns = Vec::with_capacity(702);
    for a in b'a'..=b'z' {
        for b in b'a'..=b'z' {
            patterns.push(format!("{}{}", a as char, b as char));
        }
    }
    for c in b'a'..=b'z' {
        let c = c as char;
        patterns.push(format!("{c}.{c}"));
    }
    patterns
}

fn part2_db() -> &'static Database {
    static DB: OnceLock<Database> = OnceLock::new();
    DB.get_or_init(|| compile(&part2_patterns()))
}

thread_local! {
    static PART2_SCRATCH: Scratch = alloc_scratch(part2_db());
}

/// Check if a line is nice under the part-2 rules, via Hyperscan. See the
/// module docs and [`is_nice_via_hyperscan`].
pub fn is_nice_v2_via_hyperscan(line: &str) -> bool {
    let mut ctx = Part2Match::default();
    PART2_SCRATCH.with(|scratch| {
        scan(
            part2_db(),
            scratch,
            line,
            on_part2_match,
            std::ptr::addr_of_mut!(ctx).cast(),
        );
    });

    let repeated_pair = ctx
        .pair_min
        .iter()
        .zip(ctx.pair_max.iter())
        .any(|(&min, &max)| min >= 0 && max - min >= 2);

    repeated_pair && ctx.sandwich
}
