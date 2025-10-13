import { createSignal, createEffect } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

// --- Types matching Rust enums/struct ---

export type SortKey = "name" | "size" | "filetype" | "date_modified";
export type ViewMode = "grid" | "list";
export type IconSize = "small" | "medium" | "large";

export interface LayoutCache {
    // Sorting
    sort_key: SortKey;
    ascending: boolean;

    // Viewing
    view_mode: ViewMode;
    show_hidden: boolean;
    show_extensions: boolean;
    icon_size: IconSize;
}

// --- Defaults (matches Rust Default impl) ---

export const defaultLayoutCache: LayoutCache = {
    sort_key: "name",
    ascending: true,
    view_mode: "grid",
    show_hidden: false,
    show_extensions: true,
    icon_size: "small",
};

// --- Backend bridge (Tauri commands) ---

export async function fetchLayoutSettings(): Promise<LayoutCache> {
    try {
        const result = await invoke<LayoutCache>("fetch_layout_settings");
        return { ...defaultLayoutCache, ...result }; // merge missing fields
    } catch (err) {
        console.error("Failed to fetch layout settings:", err);
        return defaultLayoutCache;
    }
}

export async function updateLayoutSettings(newSettings: LayoutCache): Promise<void> {
    try {
        await invoke("update_layout_settings", { newSettings });
    } catch (err) {
        console.error("Failed to update layout settings:", err);
    }
}

// --- Reactive layout cache store ---

export function useLayoutCache() {
    // Sorting
    const [sortKey, setSortKey] = createSignal<SortKey>("name");
    const [ascending, setAscending] = createSignal(true);

    // Viewing
    const [viewMode, setViewMode] = createSignal<ViewMode>("grid");
    const [showHidden, setShowHidden] = createSignal(false);
    const [showExtensions, setShowExtensions] = createSignal(true);
    const [iconSize, setIconSize] = createSignal<IconSize>("small");

    // --- Load from backend cache on mount ---
    fetchLayoutSettings().then(cache => {
        setSortKey(cache.sort_key);
        setAscending(cache.ascending);
        setViewMode(cache.view_mode);
        setShowHidden(cache.show_hidden);
        setShowExtensions(cache.show_extensions);
        setIconSize(cache.icon_size);
    });

    // --- Auto-save whenever settings change ---
    createEffect(() => {
        const cache: LayoutCache = {
            sort_key: sortKey(),
            ascending: ascending(),
            view_mode: viewMode(),
            show_hidden: showHidden(),
            show_extensions: showExtensions(),
            icon_size: iconSize(),
        };
        updateLayoutSettings(cache);
    });

    // --- Expose state and setters ---
    return {
        sortKey, setSortKey,
        ascending, setAscending,
        viewMode, setViewMode,
        showHidden, setShowHidden,
        showExtensions, setShowExtensions,
        iconSize, setIconSize,
    };
}
