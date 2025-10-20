use std::{
    ffi::OsString, os::windows::ffi::OsStringExt, path::PathBuf, ptr, thread, time::Duration,
};
use windows::Win32::{
    Foundation::{HANDLE, HGLOBAL, POINT},
    System::{
        DataExchange::{
            CloseClipboard, EmptyClipboard, GetClipboardData,
            IsClipboardFormatAvailable, OpenClipboard, RegisterClipboardFormatW, SetClipboardData,
        },
        Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE, GMEM_ZEROINIT},
        Ole::CF_HDROP,
    },
    UI::Shell::{DragQueryFileW, DROPFILES, HDROP},
};
use windows_core::{w, BOOL};

#[derive(Debug)]
pub enum ClipboardOp {
    Copy,
    Move,
    Link,
    Unknown
}

/// Copy real filesystem paths to the Windows clipboard in the same way Explorer does.
/// Explorer will enable "Paste" after this call.
pub fn set_system_clipboard(paths: Vec<String>, op: ClipboardOp) -> Result<(), String> {
    unsafe {
        if paths.is_empty() {
            return Err("No valid paths provided".into());
        }

        let canonical: Vec<std::path::PathBuf> =
            paths.iter().map(std::path::PathBuf::from).collect();

        // Build CF_HDROP (DROPFILES + UTF16 list)
        let mut wide_units: Vec<u16> = Vec::new();
        for p in &canonical {
            let w: Vec<u16> = p
                .display()
                .to_string()
                .encode_utf16()
                .chain(Some(0))
                .collect();
            wide_units.extend_from_slice(&w);
        }
        wide_units.push(0);

        let dropfiles_size = std::mem::size_of::<DROPFILES>();
        let total_size = dropfiles_size + wide_units.len() * 2;

        let hdrop = GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, total_size)
            .map_err(|e| format!("GlobalAlloc failed: {:?}", e))?;
        let ptr = GlobalLock(hdrop) as *mut u8;
        if ptr.is_null() {
            return Err("GlobalLock returned null for CF_HDROP".into());
        }

        let dropfiles = DROPFILES {
            pFiles: dropfiles_size as u32,
            pt: POINT { x: 0, y: 0 },
            fNC: BOOL(0),
            fWide: BOOL(1),
        };
        ptr::copy_nonoverlapping(
            &dropfiles as *const DROPFILES as *const u8,
            ptr,
            dropfiles_size,
        );
        ptr::copy_nonoverlapping(
            wide_units.as_ptr() as *const u8,
            ptr.add(dropfiles_size),
            wide_units.len() * 2,
        );
        GlobalUnlock(hdrop).ok();

        let effect_val: u32 = match op {
            ClipboardOp::Copy | ClipboardOp::Link => 5, // DROPEFFECT_COPY | DROPEFFECT_LINK
            ClipboardOp::Move => 2, // DROPEFFECT_MOVE
            _ => return Err(format!("Unknown DropEffect!"))
        };

        let heffect = GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, 4)
            .map_err(|e| format!("GlobalAlloc (DropEffect) failed: {:?}", e))?;
        let eptr = GlobalLock(heffect) as *mut u32;
        *eptr = effect_val;
        GlobalUnlock(heffect).ok();

        // Try opening clipboard with retries
        let mut opened = false;
        for _ in 0..10 {
            if OpenClipboard(None).is_ok() {
                opened = true;
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }
        if !opened {
            return Err("Failed to open clipboard after retries".into());
        }

        // Ensure cleanup always happens
        let _cleanup_needed = true;
        let result = (|| -> Result<(), String> {
            EmptyClipboard().map_err(|e| format!("EmptyClipboard failed: {:?}", e))?;

            // Register formats
            let _shell_idlist_fmt = RegisterClipboardFormatW(w!("Shell IDList Array"));
            let _fg_fmt = RegisterClipboardFormatW(w!("FileGroupDescriptorW"));
            let _filenamew_fmt = RegisterClipboardFormatW(w!("FileNameW"));
            let preferred_fmt = RegisterClipboardFormatW(w!("Preferred DropEffect"));
            let drop_fmt = RegisterClipboardFormatW(w!("DropEffect"));

            // CF_HDROP
            SetClipboardData(CF_HDROP.0 as u32, Some(HANDLE(hdrop.0)))
                .map_err(|e| format!("SetClipboardData CF_HDROP failed: {:?}", e))?;

            // DropEffect
            SetClipboardData(drop_fmt, Some(HANDLE(heffect.0)))
                .map_err(|e| format!("SetClipboardData DropEffect failed: {:?}", e))?;
            SetClipboardData(preferred_fmt, Some(HANDLE(heffect.0)))
                .map_err(|e| format!("SetClipboardData Preferred DropEffect failed: {:?}", e))?;

            Ok(())
        })();

        // Always close clipboard
        if _cleanup_needed {
            CloseClipboard().ok();
        }

        result
    }
}

/// Retrieve paths and clipboard info for inspection.
/// Returns `(file_list, clipboard_op)`
pub fn get_system_clipboard() -> Result<(Vec<PathBuf>, ClipboardOp), String> {
    unsafe {
        OpenClipboard(None).map_err(|e| format!("OpenClipboard failed: {}", e))?;

        // --- Gather file paths ---
        let mut file_list: Vec<PathBuf> = Vec::new();
        if IsClipboardFormatAvailable(CF_HDROP.0 as u32).is_ok() {
            let handle = GetClipboardData(CF_HDROP.0 as u32)
                .map_err(|e| format!("GetClipboardData failed: {}", e))?;
            let hdrop = HDROP(handle.0);
            let count = DragQueryFileW(hdrop, 0xFFFFFFFF, None);
            for i in 0..count {
                let mut buffer = vec![0u16; 260];
                let len = DragQueryFileW(hdrop, i, Some(&mut buffer));
                let s = OsString::from_wide(&buffer[..len as usize]);
                file_list.push(PathBuf::from(s));
            }
        }

        // --- Try reading "Preferred DropEffect" ---
        let mut op = ClipboardOp::Unknown;
        let fmt = RegisterClipboardFormatW(w!("Preferred DropEffect"));
        if fmt != 0 && IsClipboardFormatAvailable(fmt).is_ok() {
            if let Ok(handle) = GetClipboardData(fmt) {
                let ptr = GlobalLock(HGLOBAL(handle.0)) as *const u32;
                if !ptr.is_null() {
                    let val = *ptr;
                    op = match val {
                        1 | 5 => ClipboardOp::Copy, // 1=Copy, 5=Copy|Link (Explorer)
                        2 => ClipboardOp::Move,
                        4 => ClipboardOp::Link,
                        _ => ClipboardOp::Unknown,
                    };
                    GlobalUnlock(HGLOBAL(handle.0)).ok();
                }
            }
        }

        CloseClipboard().map_err(|e| format!("CloseClipboard failed: {}", e))?;

        Ok((file_list, op))
    }
}