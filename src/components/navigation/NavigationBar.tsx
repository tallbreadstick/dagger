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
import Tab from "../../classes/Tab";
import PathBar from "./PathBar";

export default function NavigationBar(props: {
    currentTabEntry: Accessor<TabEntry | null>;
    setCurrentTabEntry: Setter<TabEntry | null>;
    searchMode: boolean;
    setSearchMode: (val: boolean) => void;
    searchBarMode: string;
    setSearchBarMode: (val: "text" | "image" | "audio" | "document") => void;
    registerFocusHandler?: (handler: () => void) => void;
    refresh?: Accessor<number>;
    setRefresh?: Setter<number>;
}) {
    let searchInputRef: HTMLInputElement | undefined;

    onMount(() => {
        if (props.registerFocusHandler && searchInputRef) {
            props.registerFocusHandler(() => searchInputRef?.focus());
        }
    });

    // helpers to force a store update after a Tab method mutates internal fields.
    // function triggerEntryUpdate(entry: TabEntry | null) {
    //     if (!entry) return;
    //     // Call setTab with an identity updater to notify Solid the object changed.
    //     // This is needed because class method mutations are not always tracked automatically.
    //     entry.setTab((prev) => prev as any);
    // }

    // helpers to force a store update safely
    function updateTab(entry: TabEntry, updater: (tab: Tab) => Tab) {
        if (!entry) return;
        const newTab = updater(entry.tab);
        entry.setTab(newTab); // Solid sees replacement, no store mutation
    }

    const goBack = () => {
        const entry = props.currentTabEntry();
        if (!entry || !entry.tab.canGoBack()) return;

        updateTab(entry, (tab) => {
            const newTab = tab.clone();
            newTab.goBack(); // mutate clone
            return newTab;
        });
    };

    const goForward = () => {
        const entry = props.currentTabEntry();
        if (!entry || !entry.tab.canGoForward()) return;

        updateTab(entry, (tab) => {
            const newTab = tab.clone();
            newTab.goForward();
            return newTab;
        });
    };

    const goUp = () => {
        const entry = props.currentTabEntry();
        if (!entry || !entry.tab.canGoUp()) return;

        updateTab(entry, (tab) => {
            const newTab = tab.clone();
            newTab.goUp();
            return newTab;
        });
    };

    const refresh = () => {
        props.setRefresh?.(props.refresh ? props.refresh() + 1 : 1);
    };

    // Make currentTab reactive by calling it as a function
    const currentTab = () => props.currentTabEntry()?.tab ?? null;

    return (
        <div class="w-full h-15 flex flex-row items-center px-2 gap-2 border-b border-gray-500">
            <div class="flex flex-row items-center gap-1">
                <button
                    disabled={!currentTab()?.canGoBack()}
                    onClick={goBack}
                    class={`p-2 rounded transition ${currentTab()?.canGoBack() ? "hover:bg-gray-200 active:bg-gray-300" : "opacity-50 cursor-not-allowed"}`}
                    title="Back"
                >
                    <FaSolidChevronLeft class="w-4 h-4" />
                </button>

                <button
                    disabled={!currentTab()?.canGoForward()}
                    onClick={goForward}
                    class={`p-2 rounded transition ${currentTab()?.canGoForward() ? "hover:bg-gray-200 active:bg-gray-300" : "opacity-50 cursor-not-allowed"}`}
                    title="Forward"
                >
                    <FaSolidChevronRight class="w-4 h-4" />
                </button>

                <button
                    disabled={!currentTab()?.canGoUp()}
                    onClick={goUp}
                    class={`p-2 rounded transition ${currentTab()?.canGoUp() ? "hover:bg-gray-200 active:bg-gray-300" : "opacity-50 cursor-not-allowed"}`}
                    title="Up one level"
                >
                    <FaSolidArrowUp class="w-4 h-4" />
                </button>

                <button onClick={refresh} class="p-2 rounded hover:bg-gray-200 active:bg-gray-300 transition" title="Refresh">
                    <FaSolidRotateRight class="w-4 h-4" />
                </button>
            </div>

            <div class="flex flex-row items-center grow min-w-0 bg-gray-300 border border-gray-300 rounded px-2 py-1 overflow-hidden">
                <div class="flex-1 min-w-0 overflow-x-auto no-scrollbar">
                    {props.searchMode ? (
                        <SearchBar
                            mode={props.searchBarMode}
                            setMode={props.setSearchBarMode}
                            inputRef={(el) => (searchInputRef = el)}
                        />
                    ) : (
                        <PathBar
                            currentPath={currentTab()?.workingDir ?? ""}
                            onNavigate={(newPath) => {
                                const entry = props.currentTabEntry();
                                if (!entry) return;

                                const normalizedPath = newPath.replace(/\//g, "\\");
                                updateTab(entry, (tab) => {
                                    const newTab = tab.clone();
                                    newTab.navigateTo(normalizedPath);
                                    return newTab;
                                });
                            }}
                        />
                    )}
                </div>
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
