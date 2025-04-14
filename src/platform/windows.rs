// src/platform/windows.rs

// No longer need extern crate with winapi 0.3 and Rust 2018+ module system

// Use winapi 0.3 module structure
use winapi::shared::windef::{HWND, POINT, RECT};
use winapi::um::winuser; // winuser covers most UI functions

use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::ptr;

// Use crate:: prefix for local modules/types
use crate::platform::{Cursor, Handle, Rect, VK};

const SHELLSHOCK_TITLE: &'static str = "ShellShock Live";

#[derive(Debug)]
pub struct WinHandle {
    hwnd: HWND,
}

impl WinHandle {
    fn new(hwnd: HWND) -> Self {
        WinHandle { hwnd }
    }
}

impl Handle for WinHandle {
    // is_key_pressed remains largely the same, just update the function path
    fn is_key_pressed(&self, vk: VK) -> bool {
        // https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
        let key_code = match vk {
            VK::Key1 => 0x31, // '1' key
            VK::Key2 => 0x32, // '2' key
            VK::Key3 => 0x33, // '3' key
            VK::Key4 => 0x34, // '4' key
            VK::Key5 => 0x35, // '5' key
            VK::Key6 => 0x36, // '6' key
            VK::Key7 => 0x37, // '7' key
        };

        // Call functions via winapi::um::winuser::FunctionName
        let state = unsafe { winuser::GetAsyncKeyState(key_code) }; // Returns i16
        // Check the sign bit (MSB) by seeing if the value is negative.
        state < 0
    }

    // Update function path for GetClientRect
    fn get_window_rect(&self) -> Rect {
        let mut win_rect = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };

        // Use winuser::GetClientRect
        let success = unsafe { winuser::GetClientRect(self.hwnd, &mut win_rect) };

        if success == 0 { // BOOL return type, 0 is failure
            eprintln!("[ERROR] Failed to get client rect. Is game window active?");
            return Rect::new(0, 0);
        }

        let width = win_rect.right - win_rect.left;
        let height = win_rect.bottom - win_rect.top;

        // Ensure non-negative dimensions
        Rect::new( if width < 0 { 0 } else { width },
                   if height < 0 { 0 } else { height })
    }

    // Update function paths for GetCursorPos and ScreenToClient
    fn get_mouse_position_in_window(&self) -> Cursor {
        let mut pt = POINT { x: 0, y: 0 };

        unsafe {
            // Use winuser::GetCursorPos
            if winuser::GetCursorPos(&mut pt) == 0 { // Returns BOOL
                eprintln!("[ERROR] Failed to get cursor position.");
                return Cursor::new(0,0); // Return default on error
            }
            // Use winuser::ScreenToClient
            if winuser::ScreenToClient(self.hwnd, &mut pt) == 0 { // Returns BOOL
                eprintln!("[ERROR] Failed to convert screen to client coordinates.");
                return Cursor::new(0,0); // Return default on error
            }
        }
        Cursor::new(pt.x, pt.y)
    }
}

/// Finds the ShellShock Live window handle by its title. Loops until found.
pub fn find_shellshock_handle() -> WinHandle {
    use std::thread;
    use std::time;

    loop {
        thread::sleep(time::Duration::from_millis(100));
        if let Some(handle) = get_handle_by_title(SHELLSHOCK_TITLE) {
            return handle;
        }
    }
}

/// Helper function to find a window by title using Windows API.
// Update function path for FindWindowW
fn get_handle_by_title(title: &str) -> Option<WinHandle> {
    let wide: Vec<u16> = OsStr::new(title).encode_wide().chain(once(0)).collect();
    // Use winuser::FindWindowW
    let hwnd = unsafe { winuser::FindWindowW(ptr::null_mut(), wide.as_ptr()) };
    if hwnd.is_null() {
        return None;
    }
    Some(WinHandle::new(hwnd))
}