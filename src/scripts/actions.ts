import { invoke } from "@tauri-apps/api/core";

/**
 * Copy the currently selected file(s) to the OS clipboard
 * @param paths Absolute paths of all selected files
 * @returns Promise resolving to void if copy is successful
 */
export async function copyItemsToClipboard(paths: string[]): Promise<void> {
    try {
        console.log("attempting...");
        await invoke('copy_items_to_clipboard', { paths });
    } catch (err) {
        console.error('copyItemsToClipboard failed:', err);
    }
}

/**
 * Paste file list from clipboard to current directory
 * @param paths Absolute paths of all selected files
 * @returns Promise resolving to void if copy is successful
 */
export async function pasteItemsFromClipboard(workingDir: string): Promise<void> {
    try {
        console.log("attempting...");
        await invoke('paste_items_from_clipboard', { workingDir });
    } catch (err) {
        console.error('pasteItemsFromClipboard failed:', err);
    }
}
