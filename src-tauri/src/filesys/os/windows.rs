use std::{
    ffi::{OsStr, OsString},
    os::windows::ffi::{OsStrExt, OsStringExt},
    path::PathBuf,
    ptr,
};
use windows::Win32::{
    Foundation::{HANDLE, HGLOBAL, POINT},
    System::{
        DataExchange::{
            CloseClipboard, EmptyClipboard, EnumClipboardFormats, GetClipboardData,
            GetClipboardFormatNameW, IsClipboardFormatAvailable, OpenClipboard,
            RegisterClipboardFormatW, SetClipboardData,
        },
        Memory::{GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE, GMEM_ZEROINIT},
        Ole::{CF_HDROP, DROPEFFECT_COPY},
    },
    UI::Shell::{
        DragQueryFileW, DROPFILES, FD_ATTRIBUTES, FD_FILESIZE, FD_WRITESTIME, FILEDESCRIPTORW,
        FILEGROUPDESCRIPTORW, HDROP,
    },
};
use windows_core::{BOOL, PCWSTR};

/// Copy real filesystem paths to the Windows clipboard in the same way Explorer does.
/// Explorer will enable "Paste" after this call.
pub fn set_system_clipboard(paths: Vec<String>) -> Result<(), String> {
    unsafe {
        println!("---- set_system_clipboard START ----");

        // 1. Canonicalize paths
        let canonical: Vec<PathBuf> = paths
            .into_iter()
            .filter_map(|p| std::fs::canonicalize(p).ok())
            .collect();
        if canonical.is_empty() {
            return Err("No valid paths provided".into());
        }
        println!("Canonical paths to copy:");
        for (i, p) in canonical.iter().enumerate() {
            println!("  [{}] {}", i, p.display());
        }

        // 2. Build UTF-16 double-null terminated list for CF_HDROP
        let mut wide_units: Vec<u16> = Vec::new();
        for p in &canonical {
            let w: Vec<u16> = OsStr::new(p)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            wide_units.extend_from_slice(&w);
        }
        wide_units.push(0); // final null terminator

        let dropfiles_size = std::mem::size_of::<DROPFILES>();
        let total_size = dropfiles_size + wide_units.len() * std::mem::size_of::<u16>();

        let hdrop = GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, total_size)
            .map_err(|e| format!("GlobalAlloc failed: {e:?}"))?;
        let ptr = GlobalLock(hdrop) as *mut u8;
        if ptr.is_null() {
            return Err("GlobalLock returned null".into());
        }

        // Write DROPFILES header
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

        // Write UTF-16 paths
        let data_ptr = ptr.add(dropfiles_size);
        ptr::copy_nonoverlapping(
            wide_units.as_ptr() as *const u8,
            data_ptr,
            wide_units.len() * std::mem::size_of::<u16>(),
        );

        GlobalUnlock(hdrop).ok();
        println!("CF_HDROP HGLOBAL prepared ({} bytes)", total_size);

        // 3. Prepare Preferred DropEffect
        let effect_val: u32 = DROPEFFECT_COPY.0;
        let heffect = GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, std::mem::size_of::<u32>())
            .map_err(|e| format!("GlobalAlloc (DropEffect) failed: {e:?}"))?;
        let effect_ptr = GlobalLock(heffect) as *mut u32;
        if effect_ptr.is_null() {
            return Err("GlobalLock (DropEffect) returned null".into());
        }
        *effect_ptr = effect_val;
        GlobalUnlock(heffect).ok();
        println!("Preferred DropEffect HGLOBAL prepared (value = {effect_val})");

        // 4. Open clipboard
        OpenClipboard(None).map_err(|e| format!("OpenClipboard failed: {e:?}"))?;
        EmptyClipboard().map_err(|e| format!("EmptyClipboard failed: {e:?}"))?;
        println!("Clipboard opened and emptied");

        // 5. Set CF_HDROP
        SetClipboardData(CF_HDROP.0 as u32, Some(HANDLE(hdrop.0)))
            .map_err(|e| format!("SetClipboardData (CF_HDROP) failed: {e:?}"))?;
        println!("CF_HDROP set");

        // 6. Set Preferred DropEffect
        let name_utf16: Vec<u16> = "Preferred DropEffect\0".encode_utf16().collect();
        let fmt = RegisterClipboardFormatW(PCWSTR(name_utf16.as_ptr()));
        SetClipboardData(fmt, Some(HANDLE(heffect.0)))
            .map_err(|e| format!("SetClipboardData (DropEffect) failed: {e:?}"))?;
        println!("Preferred DropEffect set");

        // 7. Set "FileNameW" for each file
        // Explorer writes the full Unicode path of the first file under the "FileNameW" clipboard format.
        // This helps apps like Outlook or WinRAR recognize the file names directly.
        // TODO: allocate HGLOBAL with UTF-16 of the first file and call SetClipboardData with FileNameW format.
        if let Some(first_path) = canonical.get(0) {
            // Convert the first file path to UTF-16 with null terminator
            let wide_name: Vec<u16> = first_path
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0)) // null-terminate
                .collect();

            // Allocate global memory for the clipboard
            let h_name = GlobalAlloc(
                GMEM_MOVEABLE | GMEM_ZEROINIT,
                wide_name.len() * std::mem::size_of::<u16>(),
            )
            .map_err(|e| format!("GlobalAlloc (FileNameW) failed: {e:?}"))?;

            let ptr = GlobalLock(h_name) as *mut u16;
            if ptr.is_null() {
                return Err("GlobalLock (FileNameW) returned null".into());
            }

            // Copy UTF-16 bytes into HGLOBAL
            ptr::copy_nonoverlapping(wide_name.as_ptr(), ptr, wide_name.len());

            GlobalUnlock(h_name).ok();

            // Register "FileNameW" format and set clipboard data
            let fmt_name_utf16: Vec<u16> = "FileNameW\0".encode_utf16().collect();
            let fmt = RegisterClipboardFormatW(PCWSTR(fmt_name_utf16.as_ptr()));
            SetClipboardData(fmt, Some(HANDLE(h_name.0)))
                .map_err(|e| format!("SetClipboardData (FileNameW) failed: {e:?}"))?;

            println!("FileNameW set for first file: {}", first_path.display());
        }

        // 8. Set "FileGroupDescriptorW" if multiple files
        // This is a structured descriptor containing the display names of each file.
        // It is used by apps like Outlook to know how many files are being copied.
        // TODO: define FILEGROUPDESCRIPTORW struct, fill in file names, allocate HGLOBAL, SetClipboardData.
        let descriptor_size = std::mem::size_of::<FILEGROUPDESCRIPTORW>()
            + canonical.len().saturating_sub(1) * std::mem::size_of::<FILEDESCRIPTORW>();

        let hfgd = GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, descriptor_size)
            .map_err(|e| format!("GlobalAlloc (FileGroupDescriptorW) failed: {e:?}"))?;
        let ptr = GlobalLock(hfgd) as *mut u8;
        if ptr.is_null() {
            return Err("GlobalLock (FileGroupDescriptorW) returned null".into());
        }

        // Fill FILEGROUPDESCRIPTORW
        let fg: *mut FILEGROUPDESCRIPTORW = ptr as *mut FILEGROUPDESCRIPTORW;
        (*fg).cItems = canonical.len() as u32;

        let files_ptr = &mut (*fg).fgd as *mut FILEDESCRIPTORW;

        for (i, path) in canonical.iter().enumerate() {
            let fd: *mut FILEDESCRIPTORW = files_ptr.add(i);
            (*fd).dwFlags = (FD_ATTRIBUTES.0 | FD_FILESIZE.0 | FD_WRITESTIME.0) as u32;
            (*fd).cFileName = [0u16; 260];

            let name = path.file_name().unwrap_or_default();
            let name_w: Vec<u16> = name.encode_wide().chain(std::iter::once(0)).collect();
            for (j, c) in name_w.iter().enumerate().take(260) {
                (*fd).cFileName[j] = *c;
            }

            // Optional: fill file attributes or size
            (*fd).dwFileAttributes = 0;
            (*fd).nFileSizeHigh = 0;
            (*fd).nFileSizeLow = 0;
        }

        GlobalUnlock(hfgd).ok();

        // Register FileGroupDescriptorW format
        let fg_format_utf16: Vec<u16> = "FileGroupDescriptorW\0".encode_utf16().collect();
        let fg_fmt = RegisterClipboardFormatW(PCWSTR(fg_format_utf16.as_ptr()));

        SetClipboardData(fg_fmt, Some(HANDLE(hfgd.0)))
            .map_err(|e| format!("SetClipboardData (FileGroupDescriptorW) failed: {e:?}"))?;
        println!("FileGroupDescriptorW set with {} files", canonical.len());

        // 9. Optional: Set other Explorer formats for compatibility
        // - "Shell IDList Array" (format 0x000C) → used for drag-drop in Explorer windows.
        // - "DataObject" → OLE internal format, usually optional for simple copy-paste.
        // 9. Optional: Set other Explorer formats for compatibility

        // 9a. Shell IDList Array (CFSTR_SHELLIDLIST, typically 0x000C)
        let shell_idlist_format_utf16: Vec<u16> = "Shell IDList Array\0".encode_utf16().collect();
        let shell_idlist_fmt = RegisterClipboardFormatW(PCWSTR(shell_idlist_format_utf16.as_ptr()));

        // We'll just set an empty HGLOBAL; Explorer fills this properly for full drag-drop
        let h_shell_idlist = GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, 1)
            .map_err(|e| format!("GlobalAlloc (Shell IDList) failed: {e:?}"))?;
        SetClipboardData(shell_idlist_fmt, Some(HANDLE(h_shell_idlist.0)))
            .map_err(|e| format!("SetClipboardData (Shell IDList) failed: {e:?}"))?;
        println!("Shell IDList Array set (placeholder)");

        // 9b. DataObject (OLE internal format)
        let dataobject_format_utf16: Vec<u16> = "DataObject\0".encode_utf16().collect();
        let dataobject_fmt = RegisterClipboardFormatW(PCWSTR(dataobject_format_utf16.as_ptr()));
        let h_dataobject = GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, 1)
            .map_err(|e| format!("GlobalAlloc (DataObject) failed: {e:?}"))?;
        SetClipboardData(dataobject_fmt, Some(HANDLE(h_dataobject.0)))
            .map_err(|e| format!("SetClipboardData (DataObject) failed: {e:?}"))?;
        println!("DataObject set (placeholder)");

        // 10. Close clipboard
        CloseClipboard().map_err(|e| format!("CloseClipboard failed: {e:?}"))?;
        println!("Clipboard closed");
        println!("---- set_system_clipboard END ----");

        Ok(())
    }
}

/// Retrieve paths and clipboard info for inspection.
///
/// Returns all file paths if CF_HDROP exists, or an empty Vec otherwise.
pub fn get_system_clipboard() -> Result<Vec<PathBuf>, String> {
    unsafe {
        // 1. Open clipboard
        OpenClipboard(None).map_err(|e| format!("OpenClipboard failed: {}", e))?;

        println!("---- Clipboard Formats ----");
        // Enumerate all formats
        let mut fmt: u32 = 0;
        loop {
            match EnumClipboardFormats(fmt) {
                next if next != 0 => {
                    fmt = next;

                    // Get format name
                    let mut name_buf = [0u16; 256];
                    let len = GetClipboardFormatNameW(fmt, &mut name_buf);
                    let name = if len > 0 {
                        String::from_utf16_lossy(&name_buf[..len as usize])
                    } else {
                        String::new()
                    };

                    println!(
                        "format {}{}",
                        fmt,
                        if name.is_empty() {
                            "".into()
                        } else {
                            format!(": {}", name)
                        }
                    );

                    // Peek at HGLOBAL contents if available
                    if let Ok(_) = IsClipboardFormatAvailable(fmt) {
                        if let Ok(handle) = GetClipboardData(fmt) {
                            let hglobal = HGLOBAL(handle.0);
                            let ptr = GlobalLock(hglobal);
                            if !ptr.is_null() {
                                let bytes: &[u8] = std::slice::from_raw_parts(
                                    ptr as *const u8,
                                    64.min(GlobalSize(hglobal) as usize),
                                );
                                println!(
                                    "  -> HGLOBAL size: {} bytes, preview: {:02X?}",
                                    GlobalSize(hglobal),
                                    bytes
                                );
                                GlobalUnlock(hglobal).ok();
                            }
                        }
                    }
                }
                _ => break,
            }
        }

        // 2. Try to read CF_HDROP (file list)
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
            println!("CF_HDROP -> {} files", file_list.len());
            for (i, p) in file_list.iter().enumerate() {
                println!("  [{}] {}", i, p.display());
            }
        } else {
            println!("CF_HDROP not available");
        }

        // 3. Try to read "Preferred DropEffect"
        let name_utf16: Vec<u16> = "Preferred DropEffect\0".encode_utf16().collect();
        let fmt = RegisterClipboardFormatW(PCWSTR(name_utf16.as_ptr()));
        if fmt != 0 && IsClipboardFormatAvailable(fmt).is_ok() {
            let handle = GetClipboardData(fmt)
                .map_err(|e| format!("GetClipboardData (DropEffect): {}", e))?;
            let ptr = GlobalLock(HGLOBAL(handle.0)) as *const u32;
            if !ptr.is_null() {
                let val = *ptr;
                let _ = GlobalUnlock(HGLOBAL(handle.0));
                let meaning = match val {
                    1 => "COPY",
                    2 => "MOVE",
                    5 => "LINK",
                    _ => "UNKNOWN",
                };
                println!("Preferred DropEffect: {val:#X} ({meaning})");
            }
        } else {
            println!("Preferred DropEffect not available");
        }

        // 4. Close clipboard
        CloseClipboard().map_err(|e| format!("CloseClipboard failed: {}", e))?;

        Ok(file_list)
    }
}
