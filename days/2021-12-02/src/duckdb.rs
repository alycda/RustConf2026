//! Hand-written FFI bindings to DuckDB's C API.
//!
//! DuckDB is an in-process analytical database — columnar storage, a
//! vectorized execution engine, a query optimizer, the whole apparatus built
//! for scanning billions of rows. This module points it at a thousand rows of
//! `forward 5` and asks it to add them up.
//!
//! The reason it is *almost* useful: part two of this puzzle is a running
//! total, and a running total is what a SQL window function is. The aim at any
//! command is `SUM(...) OVER (ORDER BY idx)`, which is arguably a more direct
//! statement of the rule than the fold in `lib.rs` is. The puzzle is not being
//! bent to fit the tool here; the tool genuinely says the thing.
//!
//! What the scratchpad turned up before any of this was written:
//!
//! 1. **A connection is not an isolation boundary — the database is.** Two
//!    connections to one `duckdb_database` share a catalog, so a second
//!    `CREATE TABLE course` fails as "already exists" and the next query
//!    silently reads the *first* course's rows. The first version of the
//!    scratch program did exactly that and cheerfully reported 150/900 for an
//!    empty course. Every solve here therefore opens its own in-memory
//!    database, not just its own connection.
//! 2. **`SUM` over no rows is `NULL`, not `0`.** An empty course is a real
//!    input (`Day::from_str("")`), so both queries wrap their sums in
//!    `COALESCE(..., 0)`. Without it the answer comes back as a valid result
//!    containing a NULL, which is a different failure from an error and is
//!    easy to read as a zero.
//! 3. **The `duckdb_value_*` result accessors are deprecated** and documented
//!    as scheduled for removal. The supported path is columnar:
//!    `duckdb_fetch_chunk` → `duckdb_data_chunk_get_vector` →
//!    `duckdb_vector_get_data`, plus a separate validity bitmask. That is more
//!    binding surface, and it is also the honest one — reading a result out of
//!    DuckDB means meeting its column vectors.

use std::ffi::{CStr, CString, c_char, c_void};

use crate::Command;

/// `idx_t` is `uint64_t`.
type IdxT = u64;

/// `duckdb_state` is an enum with `DuckDBSuccess = 0`, `DuckDBError = 1`.
type DuckDbState = i32;
const DUCKDB_SUCCESS: DuckDbState = 0;

/// The handle types (`duckdb_database`, `duckdb_connection`, `duckdb_appender`,
/// `duckdb_data_chunk`, `duckdb_vector`) are all `struct { void *internal_ptr; } *`
/// — a pointer to a one-pointer struct. We only ever hold and pass them, so an
/// opaque pointer is the whole declaration needed, and it keeps us from
/// dereferencing one by accident.
#[repr(C)]
struct Opaque {
    _private: [u8; 0],
}

type DuckDbDatabase = *mut Opaque;
type DuckDbConnection = *mut Opaque;
type DuckDbAppender = *mut Opaque;
type DuckDbDataChunk = *mut Opaque;
type DuckDbVector = *mut Opaque;

/// `duckdb_result`, unlike the handles above, is a real by-value struct — and
/// `duckdb_fetch_chunk` takes one **by value**, so the layout has to be right
/// rather than merely pointer-shaped. Six machine words: three `idx_t` and
/// three pointers, all of them the deprecated accessors' business and none of
/// them ours. `#[repr(C)]` is what makes the by-value pass match the C ABI.
#[repr(C)]
#[derive(Clone, Copy)]
struct DuckDbResult {
    deprecated_column_count: IdxT,
    deprecated_row_count: IdxT,
    deprecated_rows_changed: IdxT,
    deprecated_columns: *mut c_void,
    deprecated_error_message: *mut c_char,
    internal_data: *mut c_void,
}

unsafe extern "C" {
    fn duckdb_open(path: *const c_char, out_database: *mut DuckDbDatabase) -> DuckDbState;
    fn duckdb_close(database: *mut DuckDbDatabase);
    fn duckdb_connect(
        database: DuckDbDatabase,
        out_connection: *mut DuckDbConnection,
    ) -> DuckDbState;
    fn duckdb_disconnect(connection: *mut DuckDbConnection);

    fn duckdb_query(
        connection: DuckDbConnection,
        query: *const c_char,
        out_result: *mut DuckDbResult,
    ) -> DuckDbState;
    fn duckdb_destroy_result(result: *mut DuckDbResult);
    fn duckdb_result_error(result: *mut DuckDbResult) -> *const c_char;

    fn duckdb_fetch_chunk(result: DuckDbResult) -> DuckDbDataChunk;
    fn duckdb_destroy_data_chunk(chunk: *mut DuckDbDataChunk);
    fn duckdb_data_chunk_get_size(chunk: DuckDbDataChunk) -> IdxT;
    fn duckdb_data_chunk_get_vector(chunk: DuckDbDataChunk, col_idx: IdxT) -> DuckDbVector;
    fn duckdb_vector_get_data(vector: DuckDbVector) -> *mut c_void;
    fn duckdb_vector_get_validity(vector: DuckDbVector) -> *mut u64;
    fn duckdb_validity_row_is_valid(validity: *mut u64, row: IdxT) -> bool;

    fn duckdb_appender_create(
        connection: DuckDbConnection,
        schema: *const c_char,
        table: *const c_char,
        out_appender: *mut DuckDbAppender,
    ) -> DuckDbState;
    fn duckdb_appender_error(appender: DuckDbAppender) -> *const c_char;
    fn duckdb_appender_close(appender: DuckDbAppender) -> DuckDbState;
    fn duckdb_appender_destroy(appender: *mut DuckDbAppender) -> DuckDbState;
    fn duckdb_appender_end_row(appender: DuckDbAppender) -> DuckDbState;
    fn duckdb_append_int32(appender: DuckDbAppender, value: i32) -> DuckDbState;
    fn duckdb_append_int64(appender: DuckDbAppender, value: i64) -> DuckDbState;
    fn duckdb_append_varchar(appender: DuckDbAppender, val: *const c_char) -> DuckDbState;
}

/// Part one: horizontal is every `forward`, depth is `down` minus `up`.
pub const PART1_SQL: &str = "\
SELECT (COALESCE(SUM(CASE WHEN cmd = 'forward' THEN x END), 0)
      * COALESCE(SUM(CASE cmd WHEN 'down' THEN x WHEN 'up' THEN -x END), 0))::BIGINT
FROM course";

/// Part two: `aim` is a running total, so it is a window function. Every
/// `forward` contributes 0 to that sum, which is why the frame can end at
/// `CURRENT ROW` and still give each `forward` the aim as it stood *before*
/// it — the inclusive and exclusive totals are equal exactly at the rows that
/// read them.
///
/// `ROWS`, not the default `RANGE`: `RANGE` frames are defined over peer
/// groups of the `ORDER BY` value, so the two agree only as long as `idx` has
/// no duplicates. Saying `ROWS` makes the intent independent of that.
pub const PART2_SQL: &str = "\
SELECT (COALESCE(SUM(x) FILTER (WHERE cmd = 'forward'), 0)
      * COALESCE(SUM(x * aim) FILTER (WHERE cmd = 'forward'), 0))::BIGINT
FROM (
  SELECT cmd,
         x,
         SUM(CASE cmd WHEN 'down' THEN x WHEN 'up' THEN -x ELSE 0 END)
           OVER (ORDER BY idx ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW)
           AS aim
  FROM course
)";

impl DuckDbResult {
    /// The zeroed struct `duckdb_query` expects as its out-parameter. Every
    /// field belongs to the deprecated accessors or to DuckDB's internals;
    /// nothing here reads one.
    fn empty() -> Self {
        Self {
            deprecated_column_count: 0,
            deprecated_row_count: 0,
            deprecated_rows_changed: 0,
            deprecated_columns: std::ptr::null_mut(),
            deprecated_error_message: std::ptr::null_mut(),
            internal_data: std::ptr::null_mut(),
        }
    }
}

const CREATE_SQL: &str = "CREATE TABLE course (idx BIGINT, cmd VARCHAR, x INTEGER)";

/// An in-memory DuckDB holding one `course` table.
///
/// Owns the database *and* the connection, and drops them in that order.
/// Isolation is per-database on purpose — see this module's docs.
pub struct Course {
    database: DuckDbDatabase,
    connection: DuckDbConnection,
}

impl Course {
    /// Opens a fresh in-memory database, connects, and loads `commands`.
    ///
    /// A null path is DuckDB's in-memory database. Rows go in through the
    /// appender — DuckDB's bulk-load API — rather than a generated `INSERT
    /// ... VALUES` string: with a thousand rows the latter would build tens of
    /// kilobytes of SQL and end up timing the parser instead of the database.
    pub fn load(commands: &[Command]) -> miette::Result<Self> {
        let mut database: DuckDbDatabase = std::ptr::null_mut();
        let mut connection: DuckDbConnection = std::ptr::null_mut();

        // SAFETY: out-params are valid, initialized pointers; every handle is
        // checked for success before the next call uses it.
        unsafe {
            if duckdb_open(std::ptr::null(), &mut database) != DUCKDB_SUCCESS {
                return Err(miette::miette!("duckdb_open failed"));
            }
            if duckdb_connect(database, &mut connection) != DUCKDB_SUCCESS {
                duckdb_close(&mut database);
                return Err(miette::miette!("duckdb_connect failed"));
            }
        }

        // From here on `self` owns both handles, so `?` can unwind through
        // Drop rather than leaking them on every early return.
        let course = Self {
            database,
            connection,
        };

        course.execute(CREATE_SQL)?;
        course.append(commands)?;
        Ok(course)
    }

    /// Runs a statement whose result is discarded, surfacing DuckDB's own
    /// error text on failure.
    fn execute(&self, sql: &str) -> miette::Result<()> {
        let sql = CString::new(sql).map_err(|e| miette::miette!("SQL has a NUL byte: {e}"))?;
        let mut result = DuckDbResult::empty();

        // SAFETY: `sql` is NUL-terminated and outlives the call; `result` is
        // owned here and destroyed on both paths.
        unsafe {
            let state = duckdb_query(self.connection, sql.as_ptr(), &mut result);
            if state != DUCKDB_SUCCESS {
                let message = error_text(duckdb_result_error(&mut result));
                duckdb_destroy_result(&mut result);
                return Err(miette::miette!("query failed: {message}"));
            }
            duckdb_destroy_result(&mut result);
        }

        Ok(())
    }

    /// Bulk-loads the parsed course through DuckDB's appender.
    fn append(&self, commands: &[Command]) -> miette::Result<()> {
        let table = CString::new("course").expect("no NUL byte");
        let mut appender: DuckDbAppender = std::ptr::null_mut();

        // The three command words, as C strings, allocated once rather than
        // per row — `duckdb_append_varchar` copies what it is given.
        let forward = CString::new("forward").expect("no NUL byte");
        let down = CString::new("down").expect("no NUL byte");
        let up = CString::new("up").expect("no NUL byte");

        // SAFETY: `self.connection` is live; the appender is closed and
        // destroyed on every path out, including the error ones.
        unsafe {
            if duckdb_appender_create(
                self.connection,
                std::ptr::null(),
                table.as_ptr(),
                &mut appender,
            ) != DUCKDB_SUCCESS
            {
                return Err(miette::miette!("duckdb_appender_create failed"));
            }

            for (index, command) in commands.iter().enumerate() {
                let (word, amount) = match *command {
                    Command::Forward(x) => (&forward, x),
                    Command::Down(x) => (&down, x),
                    Command::Up(x) => (&up, x),
                };

                let ok = duckdb_append_int64(appender, index as i64) == DUCKDB_SUCCESS
                    && duckdb_append_varchar(appender, word.as_ptr()) == DUCKDB_SUCCESS
                    && duckdb_append_int32(appender, amount) == DUCKDB_SUCCESS
                    && duckdb_appender_end_row(appender) == DUCKDB_SUCCESS;

                if !ok {
                    let message = error_text(duckdb_appender_error(appender));
                    duckdb_appender_destroy(&mut appender);
                    return Err(miette::miette!("appending row {index} failed: {message}"));
                }
            }

            // close() flushes and is the only call that reports a flush
            // failure; destroy() alone would free the appender and swallow it.
            if duckdb_appender_close(appender) != DUCKDB_SUCCESS {
                let message = error_text(duckdb_appender_error(appender));
                duckdb_appender_destroy(&mut appender);
                return Err(miette::miette!("flushing the course failed: {message}"));
            }
            duckdb_appender_destroy(&mut appender);
        }

        Ok(())
    }

    /// Runs `sql` and reads a single `BIGINT` out of the first row.
    ///
    /// The result is read the columnar way, because the scalar accessors are
    /// deprecated: fetch one chunk, take column 0's vector, read its data
    /// pointer as `i64`. The validity mask is separate from the data and may
    /// be null — DuckDB returns null to mean "every row in this vector is
    /// valid", so a null check here is not a defensive nicety, it is the
    /// common case.
    ///
    /// `BIGINT` in that first sentence is a contract the *caller* has to keep,
    /// and it is easy to break by accident: `SUM` over an `INTEGER` column
    /// widens to `HUGEINT`, so the obvious query hands back a 16-byte value
    /// and this `cast::<i64>()` reads its low half. There is no type tag in
    /// the pointer to catch that — the wrong answer is simply the right
    /// answer modulo 2^64. Both queries here end in `::BIGINT` for exactly
    /// this reason, which also puts the range check in DuckDB, where an
    /// out-of-range value fails the query instead of being silently trimmed.
    pub fn scalar(&self, sql: &str) -> miette::Result<i64> {
        let sql = CString::new(sql).map_err(|e| miette::miette!("SQL has a NUL byte: {e}"))?;
        let mut result = DuckDbResult::empty();

        // SAFETY: `sql` outlives the call; `result` and `chunk` are owned here
        // and destroyed on every path; the vector borrows from `chunk` and is
        // read before it is destroyed.
        unsafe {
            if duckdb_query(self.connection, sql.as_ptr(), &mut result) != DUCKDB_SUCCESS {
                let message = error_text(duckdb_result_error(&mut result));
                duckdb_destroy_result(&mut result);
                return Err(miette::miette!("query failed: {message}"));
            }

            // fetch_chunk takes the result *by value*. C hands it a copy and
            // the caller still owns — and still has to destroy — the original,
            // which is exactly what `Copy` on DuckDbResult expresses.
            let mut chunk = duckdb_fetch_chunk(result);
            duckdb_destroy_result(&mut result);

            if chunk.is_null() {
                return Err(miette::miette!("query returned no chunks"));
            }

            let rows = duckdb_data_chunk_get_size(chunk);
            if rows != 1 {
                duckdb_destroy_data_chunk(&mut chunk);
                return Err(miette::miette!("expected exactly one row, got {rows}"));
            }

            let vector = duckdb_data_chunk_get_vector(chunk, 0);
            let validity = duckdb_vector_get_validity(vector);
            if !duckdb_validity_row_is_valid(validity, 0) {
                duckdb_destroy_data_chunk(&mut chunk);
                return Err(miette::miette!(
                    "the answer came back NULL — a SUM over no rows without COALESCE"
                ));
            }

            let value = *duckdb_vector_get_data(vector).cast::<i64>();
            duckdb_destroy_data_chunk(&mut chunk);
            Ok(value)
        }
    }
}

impl Drop for Course {
    fn drop(&mut self) {
        // SAFETY: both handles were produced by duckdb_open/duckdb_connect and
        // have not been released. The connection goes first: it borrows from
        // the database.
        unsafe {
            duckdb_disconnect(&mut self.connection);
            duckdb_close(&mut self.database);
        }
    }
}

/// DuckDB's error strings are borrowed from the object that produced them, so
/// this copies before that object is destroyed.
fn error_text(message: *const c_char) -> String {
    if message.is_null() {
        return "(no error message)".to_string();
    }
    // SAFETY: non-null and NUL-terminated per the C API's contract; the
    // pointee outlives this call because the owning object is destroyed after.
    unsafe { CStr::from_ptr(message) }
        .to_string_lossy()
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `duckdb_fetch_chunk` takes `duckdb_result` by value, so a wrong layout
    /// here would corrupt the call rather than fail to link.
    #[test]
    fn result_layout_is_six_machine_words() {
        assert_eq!(size_of::<DuckDbResult>(), 6 * size_of::<usize>());
        assert_eq!(size_of::<IdxT>(), 8);
    }

    /// The isolation finding from the scratchpad, pinned: two courses loaded
    /// back to back must not see each other's rows. This fails — loudly, as a
    /// "table already exists" error — the moment `load` reuses one database
    /// across calls.
    #[test]
    fn each_course_gets_its_own_database() -> miette::Result<()> {
        let first = Course::load(&[Command::Forward(5), Command::Down(5)])?;
        let second = Course::load(&[Command::Forward(1)])?;

        assert_eq!(first.scalar(PART1_SQL)?, 25);
        assert_eq!(second.scalar(PART1_SQL)?, 0);
        Ok(())
    }

    /// `SUM` over no rows is NULL, and a NULL is a *valid* result, not an
    /// error — without COALESCE this returns the "came back NULL" message
    /// rather than 0.
    #[test]
    fn an_empty_course_sums_to_zero_not_null() -> miette::Result<()> {
        let course = Course::load(&[])?;
        assert_eq!(course.scalar(PART1_SQL)?, 0);
        assert_eq!(course.scalar(PART2_SQL)?, 0);
        Ok(())
    }

    /// DuckDB computes in BIGINT, so a course whose product overflows an i32
    /// is answered correctly here rather than wrapping — the narrowing back to
    /// the puzzle's i32 is `lib.rs`'s job, and this is what it is handed.
    #[test]
    fn the_database_computes_in_64_bits() -> miette::Result<()> {
        let course = Course::load(&[Command::Forward(100_000), Command::Down(100_000)])?;
        assert_eq!(course.scalar(PART1_SQL)?, 10_000_000_000);
        Ok(())
    }

    /// The one case the test above cannot reach: a product too large for
    /// `BIGINT` itself. `SUM` widens to `HUGEINT`, so the multiply really does
    /// produce 2^64 here rather than wrapping in the database — and 2^64 is
    /// congruent to 0 mod 2^64, which is precisely the value an unchecked
    /// `cast::<i64>()` of the low half would return. A regression shows up as
    /// `Ok(0)`, the most plausible-looking wrong answer available, so the
    /// assertion is on the error and not merely on `is_err()`.
    #[test]
    fn a_product_too_large_for_bigint_fails_the_query() {
        let quarter = 1 << 30;
        let course = Course::load(&[
            Command::Forward(quarter),
            Command::Forward(quarter),
            Command::Forward(quarter),
            Command::Forward(quarter),
            Command::Down(quarter),
            Command::Down(quarter),
            Command::Down(quarter),
            Command::Down(quarter),
        ])
        .expect("the course itself is well within INTEGER");

        let error = course
            .scalar(PART1_SQL)
            .expect_err("2^64 does not fit in a BIGINT")
            .to_string();
        assert!(
            error.contains("out of range"),
            "expected a conversion error, got: {error}"
        );
    }
}
