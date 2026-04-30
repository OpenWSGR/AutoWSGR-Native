mod image;
mod interface;
mod locator;
mod recognize_enemy;
mod recognize_map;
#[cfg(test)]
mod test;

use image::BGRImage;
use interface::{Interface, RecognizeEnemyInput, WrappedPixels};

use std::ffi::{c_char, c_int, c_void};
use std::panic;

/// Helper function to catch panics and log stack trace
fn catch_panic<F, R>(f: F) -> Result<R, ()>
where
    F: FnOnce() -> R + panic::UnwindSafe,
{
    panic::catch_unwind(f).map_err(|panic_info| {
        eprintln!("Panic occurred in image-autowsgrs library:");

        // Print panic message
        if let Some(s) = panic_info.downcast_ref::<&str>() {
            eprintln!("Panic message: {s}");
        } else if let Some(s) = panic_info.downcast_ref::<String>() {
            eprintln!("Panic message: {s}");
        } else {
            eprintln!("Panic message: <unknown>");
        }

        // Print backtrace if available
        eprintln!("Stack backtrace:");
        eprintln!("{}", std::backtrace::Backtrace::capture());
    })
}

/// # Safety
/// The input pointer must be freed by caller
///
/// return -1 if failed
///
/// # Usage
///
/// locate the blue textbox and return a array contains the position of the textboxes
///
/// e.g. [[0, 1], [2, 3], [4, 5]]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn locate(input: *const c_void, output: *mut c_int) -> c_int {
    catch_panic(|| {
        let image = BGRImage::from_wrapped_pixels(unsafe { WrappedPixels::from_raw(input) });
        let result = locator::locate(&image);
        if result.is_empty() || result.len() >= 32 {
            return -1;
        }
        let len = result.len();
        unsafe { output.copy_from_nonoverlapping(result.as_ptr() as *const c_int, len) };
        len as c_int
    })
    .unwrap_or(-1)
}

/// # Safety
/// The input pointer must be freed by caller
///
/// return -1 if failed
///
/// # Usage
///
/// recognize them when enemys were spotted and return a string
///
/// e.g. "DD SS NO NO NO NO"
#[unsafe(no_mangle)]
pub unsafe extern "C" fn recognize_enemy(input: *const c_void, output: *mut c_char) -> c_int {
    catch_panic(|| {
        let input = unsafe { RecognizeEnemyInput::from_raw(input) };
        let result = recognize_enemy::recognize_enemy(&input.images);

        unsafe { output.copy_from(result.as_ptr() as *const c_char, result.len()) };
        0
    })
    .unwrap_or(-1)
}

/// # Safety
/// The input pointer must be freed by caller
///
/// return '\0' if failed (panic occurred), otherwise return the recognized character
///
/// # Usage
///
/// process decisive_battle map image for recognize
///
/// e.g. 'J'
#[unsafe(no_mangle)]
pub unsafe extern "C" fn recognize_map(input: *const c_void) -> c_char {
    catch_panic(|| {
        let image = BGRImage::from_wrapped_pixels(unsafe { WrappedPixels::from_raw(input) });
        let result = recognize_map::recognize_map(&image);
        result as c_char
    })
    .unwrap_or_default()
}
