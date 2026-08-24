//! Hand-written FFI bindings to libcaca, used only for its built-in FIGlet
//! font engine: `caca_canvas_set_figfont` loads a classic `.flf` font file
//! (see `fonts/standard.flf`) and `caca_put_figchar` renders text through it
//! onto a canvas that grows to fit. Not part of solving the puzzle — this
//! just banners the answer, the same kind of "no practical reason, but a
//! real C library on the other side of the boundary" exercise as the
//! libtcc-JIT variant of this day.

use std::ffi::{CString, c_char, c_int};
use std::path::Path;

#[repr(C)]
struct CacaCanvas {
    _private: [u8; 0],
}

unsafe extern "C" {
    fn caca_create_canvas(width: c_int, height: c_int) -> *mut CacaCanvas;
    fn caca_free_canvas(cv: *mut CacaCanvas) -> c_int;
    fn caca_canvas_set_figfont(cv: *mut CacaCanvas, path: *const c_char) -> c_int;
    fn caca_put_figchar(cv: *mut CacaCanvas, ch: u32) -> c_int;
    fn caca_flush_figlet(cv: *mut CacaCanvas) -> c_int;
    fn caca_get_canvas_width(cv: *const CacaCanvas) -> c_int;
    fn caca_get_canvas_height(cv: *const CacaCanvas) -> c_int;
    fn caca_get_canvas_chars(cv: *const CacaCanvas) -> *const u32;
}

/// Renders `text` as a block-letter banner using the FIGlet font at
/// `font_path`, via libcaca's FIGfont engine.
pub fn figlet_banner(font_path: &Path, text: &str) -> miette::Result<String> {
    let c_path = CString::new(font_path.to_string_lossy().as_bytes())
        .map_err(|e| miette::miette!("font path has a NUL byte: {e}"))?;

    unsafe {
        let cv = caca_create_canvas(0, 0);
        if cv.is_null() {
            return Err(miette::miette!("caca_create_canvas failed"));
        }

        if caca_canvas_set_figfont(cv, c_path.as_ptr()) < 0 {
            caca_free_canvas(cv);
            return Err(miette::miette!(
                "caca_canvas_set_figfont failed to load {}",
                font_path.display()
            ));
        }

        for ch in text.chars() {
            caca_put_figchar(cv, ch as u32);
        }
        caca_flush_figlet(cv);

        let width = caca_get_canvas_width(cv) as usize;
        let height = caca_get_canvas_height(cv) as usize;
        let chars = caca_get_canvas_chars(cv);

        let mut banner = String::with_capacity(width * (height + 1));
        for y in 0..height {
            for x in 0..width {
                let cp = *chars.add(y * width + x);
                let c = char::from_u32(cp).filter(|c| *c != '\0').unwrap_or(' ');
                banner.push(c);
            }
            banner.push('\n');
        }

        caca_free_canvas(cv);
        Ok(banner)
    }
}
