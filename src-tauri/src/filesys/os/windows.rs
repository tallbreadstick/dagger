// use std::{ffi::OsStr, fs, mem, os::windows::ffi::OsStrExt, path::PathBuf, ptr};

// use windows::Win32::{
//     Foundation::{GlobalFree, POINT, HANDLE},
//     System::{
//         DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData},
//         Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE},
//         Ole::CF_HDROP,
//     },
//     UI::Shell::DROPFILES,
// };
// use windows_core::BOOL;

// pub fn set_system_clipboard(paths: Vec<String>) -> Result<(), String> {
//     // 1) Canonicalize paths, filter out invalid ones
//     let canonical_paths: Vec<PathBuf> = paths
//         .iter()
//         .filter_map(|p| fs::canonicalize(p).ok())
//         .collect();
//     if canonical_paths.is_empty() {
//         return Err("No valid paths provided".into());
//     }
//     // 2) Build UTF-16 wide units
//     let mut wide_units: Vec<u16> = Vec::new();
//     for path in &canonical_paths {
//         // Convert each path to UTF-16 units
//         let utf16: Vec<u16> = OsStr::new(path)
//             .encode_wide()
//             .chain(std::iter::once(0)) // Null terminate each string
//             .collect();
//         wide_units.extend(utf16);
//     }
//     // Append final null terminator to mark end of the list (double-null termination)
//     wide_units.push(0);
//     // Compute size
//     let dropfiles_size = mem::size_of::<DROPFILES>();
//     let file_list_bytes = wide_units.len() * mem::size_of::<u16>();
//     let total_bytes = dropfiles_size + file_list_bytes;
//     // 4) Build DROPFILES header
//     let dropfiles = DROPFILES {
//         pFiles: dropfiles_size as u32,
//         pt: POINT { x: 0, y: 0 },
//         fNC: BOOL(0),
//         fWide: BOOL(1), // wide (UTF-16)
//     };
//     // 5) Prepare byte slices for header and wide data
//     // Header as bytes (safe because DROPFILES is repr(C) in the windows bindings)
//     let header_bytes: &[u8] = unsafe {
//         std::slice::from_raw_parts(&dropfiles as *const DROPFILES as *const u8, dropfiles_size)
//     };
//     let wide_bytes: &[u8] =
//         unsafe { std::slice::from_raw_parts(wide_units.as_ptr() as *const u8, file_list_bytes) };
//     // 6) Allocate Global Memory (moveable)
//     // GlobalAlloc returns a Result<HGLOBAL> in the windows crate
//     let hglobal = unsafe {
//         GlobalAlloc(GMEM_MOVEABLE, total_bytes).map_err(|e| format!("GlobalAlloc failed: {}", e))?
//     };
//     // We'll keep a flag to know whether SetClipboardData succeeded (Windows takes ownership)
//     let mut handed_off_to_clipboard = false;
//     // 7) Lock, write, unlock, set clipboard
//     let result: Result<(), String> = unsafe {
//         // Lock the memory to get pointer
//         let mem_ptr = GlobalLock(hglobal) as *mut u8;
//         if mem_ptr.is_null() {
//             // free allocated memory since we haven't handed it off
//             let _ = GlobalFree(Some(hglobal));
//             return Err("GlobalLock returned null".into());
//         }
//         // Copy header
//         ptr::copy_nonoverlapping(header_bytes.as_ptr(), mem_ptr, header_bytes.len());
//         // Copy wide bytes right after header
//         let data_dest = mem_ptr.add(header_bytes.len());
//         ptr::copy_nonoverlapping(wide_bytes.as_ptr(), data_dest, wide_bytes.len());
//         // Unlock
//         GlobalUnlock(hglobal).map_err(|e| format!("GlobalUnlock failed: {}", e))?;
//         // Open clipboard (no owner window)
//         OpenClipboard(None).map_err(|e| format!("OpenClipboard failed: {}", e))?;
//         // Empty clipboard
//         EmptyClipboard().map_err(|e| format!("EmptyClipboard failed: {}", e))?;
//         // SetClipboardData: pass CF_HDROP and the HGLOBAL handle. On success, Windows owns the memory.
//         SetClipboardData(CF_HDROP.0 as u32, Some(HANDLE(hglobal.0)))
//             .map_err(|e| format!("SetClipboardData failed: {}", e))?;
//         // If we reach here, Windows has taken ownership.
//         handed_off_to_clipboard = true;
//         // Close clipboard
//         CloseClipboard().map_err(|e| format!("CloseClipboard failed: {}", e))?;
//         Ok(())
//     };
//     if !handed_off_to_clipboard || result.is_err() {
//         // Attempt to free if we failed before handing off ownership
//         let _ = unsafe { GlobalFree(Some(hglobal)) };
//     }
//     result
// }

use std::{
    ffi::OsStr,
    fs,
    mem,
    os::windows::ffi::OsStrExt,
    path::PathBuf,
    ptr,
};
use windows::Win32::{
    Foundation::{GlobalFree, HANDLE, POINT},
    System::{
        DataExchange::{
            CloseClipboard, EmptyClipboard, OpenClipboard,
            RegisterClipboardFormatW, SetClipboardData,
        },
        Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE, GMEM_ZEROINIT},
        Ole::{CF_HDROP, DROPEFFECT_COPY},
    },
    UI::Shell::DROPFILES,
};
use windows_core::{PCWSTR, BOOL};

pub fn set_system_clipboard(paths: Vec<String>) -> Result<(), String> {
    use windows::Win32::Foundation::{GetLastError, NO_ERROR};

    // 1) Canonicalize paths
    let canonical_paths: Vec<PathBuf> = paths
        .iter()
        .filter_map(|p| fs::canonicalize(p).ok())
        .collect();
    if canonical_paths.is_empty() {
        return Err("No valid paths provided".into());
    }

    // 2) Build UTF-16 wide units (double-null-terminated)
    let mut wide_units: Vec<u16> = Vec::new();
    for path in &canonical_paths {
        let utf16: Vec<u16> = OsStr::new(path)
            .encode_wide()
            .chain(std::iter::once(0)) // null per string
            .collect();
        wide_units.extend(utf16);
    }
    wide_units.push(0); // final null terminator

    // 3) Prepare DROPFILES structure
    let dropfiles_size = std::mem::size_of::<DROPFILES>();
    let file_list_bytes = wide_units.len() * std::mem::size_of::<u16>();
    let total_bytes = dropfiles_size + file_list_bytes;

    let dropfiles = DROPFILES {
        pFiles: dropfiles_size as u32,
        pt: POINT { x: 0, y: 0 },
        fNC: BOOL(0),
        fWide: BOOL(1),
    };

    // 4) Allocate global memory
    let hglobal_files = unsafe {
        GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, total_bytes)
            .map_err(|e| format!("GlobalAlloc failed: {}", e))?
    };

    // 5) Lock memory and write data
    unsafe {
        let mem_ptr = GlobalLock(hglobal_files) as *mut u8;
        if mem_ptr.is_null() {
            let _ = GlobalFree(Some(hglobal_files));
            return Err("GlobalLock returned null".into());
        }

        // Write header
        std::ptr::copy_nonoverlapping(
            &dropfiles as *const DROPFILES as *const u8,
            mem_ptr,
            dropfiles_size,
        );

        // Write UTF-16 file list
        let data_dest = mem_ptr.add(dropfiles_size);
        std::ptr::copy_nonoverlapping(
            wide_units.as_ptr() as *const u8,
            data_dest,
            file_list_bytes,
        );

        // Unlock safely
        let unlock_res = GlobalUnlock(hglobal_files);
        if unlock_res.is_err() {
            let code = GetLastError();
            if code != NO_ERROR {
                return Err(format!("GlobalUnlock failed: 0x{:08X}", code.0));
            }
        }
    }

    // 6) Open clipboard
    unsafe {
        OpenClipboard(None)
            .map_err(|e| format!("OpenClipboard failed: {}", e))?;
        EmptyClipboard()
            .map_err(|e| format!("EmptyClipboard failed: {}", e))?;
    }

    // 7) Set CF_HDROP
    unsafe {
        // Do NOT free or unlock hglobal_files after this
        // Explorer owns it once SetClipboardData succeeds
        SetClipboardData(CF_HDROP.0 as u32, Some(HANDLE(hglobal_files.0)))
            .map_err(|e| format!("SetClipboardData (CF_HDROP) failed: {}", e))?;
    }

    // 8) Register and set CFSTR_PREFERREDDROPEFFECT
    unsafe {
        let name: Vec<u16> = "Preferred DropEffect\0".encode_utf16().collect();
        let fmt = RegisterClipboardFormatW(PCWSTR(name.as_ptr()));

        let hglobal_effect = GlobalAlloc(GMEM_MOVEABLE, std::mem::size_of::<u32>())
            .map_err(|e| format!("GlobalAlloc (DropEffect) failed: {}", e))?;

        let ptr = GlobalLock(hglobal_effect) as *mut u32;
        if ptr.is_null() {
            let _ = GlobalFree(Some(hglobal_effect));
            CloseClipboard().ok();
            return Err("GlobalLock (DropEffect) returned null".into());
        }

        *ptr = DROPEFFECT_COPY.0 as u32;

        let unlock_effect = GlobalUnlock(hglobal_effect);
        if unlock_effect.is_err() {
            let code = GetLastError();
            if code != NO_ERROR {
                CloseClipboard().ok();
                return Err(format!("GlobalUnlock (DropEffect) failed: 0x{:08X}", code.0));
            }
        }

        SetClipboardData(fmt, Some(HANDLE(hglobal_effect.0)))
            .map_err(|e| format!("SetClipboardData (DropEffect) failed: {}", e))?;
    }

    // 9) Close clipboard
    unsafe {
        CloseClipboard()
            .map_err(|e| format!("CloseClipboard failed: {}", e))?;
    }

    Ok(())
}
