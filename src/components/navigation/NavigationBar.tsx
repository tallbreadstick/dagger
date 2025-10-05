/**
 * NavigationBar.tsx
 * -----------------
 * Toolbar containing navigation buttons, path display, and the SearchBar.
 * Switches dynamically between:
 *  - Path input mode
 *  - Search mode with selectable search type (text/image/audio/document)
 *
 * Exposes a `registerFocusHandler` to the parent App component
 * to allow keyboard shortcuts (Ctrl+F / Ctrl+1–4) to trigger input focus.
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

export default function NavigationBar(props: {
    searchMode: boolean;
    setSearchMode: (val: boolean) => void;
    searchBarMode: string;
    setSearchBarMode: (val: "text" | "image" | "audio" | "document") => void;
    registerFocusHandler?: (handler: () => void) => void;
}) {
    let searchInputRef: HTMLInputElement | undefined;

    onMount(() => {
        // 🔹 Expose the focus handler to the parent (App)
        if (props.registerFocusHandler && searchInputRef) {
            props.registerFocusHandler(() => {
                searchInputRef?.focus();
            });
        }
    });

    return (
        <div class="w-full h-15 flex flex-row items-center px-2 gap-2 border-b border-gray-500">
            {/* Navigation buttons */}
            <div class="flex flex-row items-center gap-1">
                <button class="p-2 rounded hover:bg-gray-200 active:bg-gray-300 transition" title="Back">
                    <FaSolidChevronLeft class="w-4 h-4" />
                </button>
                <button class="p-2 rounded hover:bg-gray-200 active:bg-gray-300 transition" title="Forward">
                    <FaSolidChevronRight class="w-4 h-4" />
                </button>
                <button class="p-2 rounded hover:bg-gray-200 active:bg-gray-300 transition" title="Up one level">
                    <FaSolidArrowUp class="w-4 h-4" />
                </button>
                <button class="p-2 rounded hover:bg-gray-200 active:bg-gray-300 transition" title="Refresh">
                    <FaSolidRotateRight class="w-4 h-4" />
                </button>
            </div>

            {/* Central area — switches between PathBar and SearchBar */}
            <div class="flex flex-row items-center grow bg-gray-300 border border-gray-300 rounded px-2 py-1">
                {props.searchMode ? (
                    <SearchBar
                        mode={props.searchBarMode}
                        setMode={props.setSearchBarMode}
                        inputRef={(el) => (searchInputRef = el)}
                    />
                ) : (
                    <div class="flex flex-row items-center w-full gap-2">
                        <FaSolidFolder class="w-4 h-4 text-black" />
                        <input
                            type="text"
                            value="C:\\Users\\Owner\\Documents"
                            class="w-full text-sm outline-none bg-transparent"
                        />
                    </div>
                )}
            </div>

            {/* Mode toggle button */}
            <button
                onClick={() => props.setSearchMode(!props.searchMode)}
                class="p-2 rounded hover:bg-gray-200 active:bg-gray-300 transition"
                title={props.searchMode ? "Switch to Path Mode" : "Switch to Search Mode"}
            >
                {props.searchMode ? (
                    <FaSolidFolder class="w-4 h-4 text-black" />
                ) : (
                    <FaSolidMagnifyingGlass class="w-4 h-4" />
                )}
            </button>
        </div>
    );
}
