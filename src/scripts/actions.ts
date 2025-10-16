import { invoke } from "@tauri-apps/api/core";
import { showCopyToast, showPasteProgressToast } from "../components/Toasts";

/**
 * Copy the currently selected file(s) to the OS clipboard
 * @param paths Absolute paths of all selected files
 * @returns Promise resolving to void if copy is successful
 */
export async function copyItemsToClipboard(paths: string[]): Promise<void> {
    try {
        console.log("attempting...");
        await invoke('copy_items_to_clipboard', { paths });
        showCopyToast(paths.length);
    } catch (err) {
        console.error('copyItemsToClipboard failed:', err);
    }
}

/**
 * Paste file list from clipboard to current directory
 * @param paths Absolute paths of all selected files
 * @returns Promise resolving to void if copy is successful
 */
export async function pasteItemsFromClipboard(workingDir: string | undefined): Promise<void> {
    if (workingDir === undefined) return;
    showPasteProgressToast(workingDir);
}

/**
 * Send the user’s chosen conflict resolution strategy back to Rust
 * after a duplicate file name conflict during paste.
 */
export async function resolveCopyConflict(
    requestId: number,
    strategy: "Ignore" | "Replace" | "Index",
    repeatForAll: boolean = false
): Promise<void> {
    try {
        await invoke("resolve_copy_conflict", {
            payload: {
                request_id: requestId,
                strategy,
                repeat_for_all: repeatForAll,
            },
        });
        console.log(`[resolveCopyConflict] Submitted response for request ${requestId}: ${strategy}${repeatForAll ? " (apply to all)" : ""}`);
    } catch (err) {
        console.error("resolveCopyConflict failed:", err);
    }
}
