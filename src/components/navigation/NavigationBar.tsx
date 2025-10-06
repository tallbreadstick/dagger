/**
 * NavigationBar.tsx
 * -----------------
 * Uses the active TabEntry to operate Back/Forward/Up/Refresh.
 */

import { onMount } from "solid-js";
import SearchBar from "./SearchBar";
import {
    FaSolidChevronLeft,
    FaSolidChevronRight,
    FaSolidArrowUp,
    FaSolidRotateRight,
    FaSolidMagnifyingGlass,
    FaSolidFolder,
} from "solid-icons/fa";
import type { Accessor, Setter } from "solid-js";
import type { TabEntry } from "../../App";

export default function NavigationBar(props: {
    currentTabEntry: Accessor<TabEntry | null>;
    setCurrentTabEntry: Setter<TabEntry | null>;
    searchMode: boolean;
    setSearchMode: (val: boolean) => void;
    searchBarMode: string;
    setSearchBarMode: (val: "text" | "image" | "audio" | "document") => void;
    registerFocusHandler?: (handler: () => void) => void;
}) {
    let searchInputRef: HTMLInputElement | undefined;

    onMount(() => {
        if (props.registerFocusHandler && searchInputRef) {
            props.registerFocusHandler(() => searchInputRef?.focus());
        }
    });

    // helpers to force a store update after a Tab method mutates internal fields.
    function triggerEntryUpdate(entry: TabEntry | null) {
        if (!entry) return;
        // Call setTab with an identity updater to notify Solid the object changed.
        // This is needed because class method mutations are not always tracked automatically.
        entry.setTab((prev) => prev as any);
    }

    const goBack = () => {
        const entry = props.currentTabEntry();
        const tab = entry?.tab ?? null;
        if (!tab || !tab.canGoBack()) return;
        tab.goBack();
        triggerEntryUpdate(entry);
    };

    const goForward = () => {
        const entry = props.currentTabEntry();
        const tab = entry?.tab ?? null;
        if (!tab || !tab.canGoForward()) return;
        tab.goForward();
        triggerEntryUpdate(entry);
    };

    const goUp = () => {
        const entry = props.currentTabEntry();
        const tab = entry?.tab ?? null;
        if (!tab || !tab.canGoUp()) return;
        tab.goUp();
        triggerEntryUpdate(entry);
    };

    const refresh = () => {
        triggerEntryUpdate(props.currentTabEntry());
    };

    const currentTab = props.currentTabEntry()?.tab ?? null;

    return (
        <div class="w-full h-15 flex flex-row items-center px-2 gap-2 border-b border-gray-500">
            <div class="flex flex-row items-center gap-1">
                <button
                    disabled={!currentTab?.canGoBack()}
                    onClick={goBack}
                    class={`p-2 rounded transition ${currentTab?.canGoBack() ? "hover:bg-gray-200 active:bg-gray-300" : "opacity-50 cursor-not-allowed"}`}
                    title="Back"
                >
                    <FaSolidChevronLeft class="w-4 h-4" />
                </button>

                <button
                    disabled={!currentTab?.canGoForward()}
                    onClick={goForward}
                    class={`p-2 rounded transition ${currentTab?.canGoForward() ? "hover:bg-gray-200 active:bg-gray-300" : "opacity-50 cursor-not-allowed"}`}
                    title="Forward"
                >
                    <FaSolidChevronRight class="w-4 h-4" />
                </button>

                <button
                    disabled={!currentTab?.canGoUp()}
                    onClick={goUp}
                    class={`p-2 rounded transition ${currentTab?.canGoUp() ? "hover:bg-gray-200 active:bg-gray-300" : "opacity-50 cursor-not-allowed"}`}
                    title="Up one level"
                >
                    <FaSolidArrowUp class="w-4 h-4" />
                </button>

                <button onClick={refresh} class="p-2 rounded hover:bg-gray-200 active:bg-gray-300 transition" title="Refresh">
                    <FaSolidRotateRight class="w-4 h-4" />
                </button>
            </div>

            <div class="flex flex-row items-center grow bg-gray-300 border border-gray-300 rounded px-2 py-1">
                {/* searchMode vs path display */}
                {props.searchMode ? (
                    <SearchBar mode={props.searchBarMode} setMode={props.setSearchBarMode} inputRef={(el) => (searchInputRef = el)} />
                ) : (
                    <div class="flex flex-row items-center w-full gap-2">
                        <FaSolidFolder class="w-4 h-4 text-black" />
                        <input type="text" value={currentTab?.workingDir ?? ""} class="w-full text-sm outline-none bg-transparent" readOnly />
                    </div>
                )}
            </div>

            <button
                onClick={() => props.setSearchMode(!props.searchMode)}
                class="p-2 rounded hover:bg-gray-200 active:bg-gray-300 transition"
                title={props.searchMode ? "Switch to Path Mode" : "Switch to Search Mode"}
            >
                {props.searchMode ? <FaSolidFolder class="w-4 h-4 text-black" /> : <FaSolidMagnifyingGlass class="w-4 h-4" />}
            </button>
        </div>
    );
}
