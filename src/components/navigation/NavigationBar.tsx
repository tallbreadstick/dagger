import { createSignal } from "solid-js";
import {
    FaSolidChevronLeft,
    FaSolidChevronRight,
    FaSolidArrowUp,
    FaSolidRotateRight,
    FaSolidMagnifyingGlass,
    FaSolidFolder
} from "solid-icons/fa";
import SearchBar from "./SearchBar";

export default function NavigationBar() {
    const [searchMode, setSearchMode] = createSignal(false);

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

            {/* Path or Search bar depending on mode */}
            <div class="flex flex-row items-center grow bg-gray-300 border border-gray-300 rounded px-2 py-1">
                {searchMode() ? (
                    <SearchBar />
                ) : (
                    <div class="flex flex-row items-center w-full gap-2">
                        <FaSolidFolder class="w-4 h-4 text-black" />
                        <input
                            type="text"
                            value="C:\Users\Owner\Documents"
                            class="w-full text-sm outline-none bg-transparent"
                        />
                    </div>
                )}
            </div>

            {/* Toggle button */}
            <button
                onClick={() => setSearchMode(!searchMode())}
                class="p-2 rounded hover:bg-gray-200 active:bg-gray-300 transition"
                title={searchMode() ? "Switch to Path Mode" : "Switch to Search Mode"}
            >
                {searchMode() ? (
                    <FaSolidFolder class="w-4 h-4 text-black" />
                ) : (
                    <FaSolidMagnifyingGlass class="w-4 h-4" />
                )}
            </button>
        </div>
    );
}
