/**
 * App.tsx
 * ----------
 * Main application root for the Tauri + SolidJS file explorer.
 * Handles:
 *  - Tab management (open/close/duplicate)
 *  - Search mode toggling and global keyboard shortcuts
 *  - Integration between NavigationBar and SearchBar focus behavior
 *  - TitleBar, Sidebar, and other UI layout structure
 */

import { createStore } from "solid-js/store";
import { createSignal, For, onCleanup } from "solid-js";
import "./index.css";
import Tab from "./classes/Tab";
import TabHeading from "./components/tabbing/TabHeading";

import { FaSolidPlus, FaSolidChevronLeft, FaSolidChevronRight } from "solid-icons/fa";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Sidebar from "./components/sidebar/Sidebar";
import NavigationBar from "./components/navigation/NavigationBar";
import TitleBar from "./components/window/TitleBar";
import { useWindowFocusEvents } from "./scripts/events";
import { useGlobalShortcuts } from "./scripts/shortcuts";

export default function App() {
    // 🔸 Syncs window focus events (blur/focus effects)
    useWindowFocusEvents();
    const appWindow = getCurrentWindow();

    // 🔹 State: List of open tabs
    const [tabs, setTabs] = createStore([
        new Tab("C:\\Program Files"),
        new Tab("C:\\Program Files (x86)"),
        new Tab("C:\\Users\\Owner"),
        new Tab("C:\\CSIT327 - Information Management 2"),
        new Tab("C:\\Temp"),
    ]);

    // 🔹 Currently active tab
    const [currentTab, setCurrentTab] = createSignal<Tab | null>(tabs[0] || null);

    // 🔹 Search state
    const [searchMode, setSearchMode] = createSignal(false);
    const [searchBarMode, setSearchBarMode] = createSignal<
        "text" | "image" | "audio" | "document"
    >("text");

    /**
     * Reference to the SearchBar focus function.
     * Assigned dynamically by NavigationBar → SearchBar on mount.
     */
    let focusSearchInput: (() => void) | null = null;

    /**
     * 🔹 Global keyboard shortcuts
     * Ctrl+F → Toggle search mode
     * Ctrl+1–4 → Set specific search modes
     * Automatically focuses input field after mode change
     */
    useGlobalShortcuts({
        toggleSearchMode: () => {
            setSearchMode((prev) => !prev);
            queueMicrotask(() => focusSearchInput?.());
        },
        setSearchMode: (m) => {
            setSearchBarMode(m);
            setSearchMode(true);
            queueMicrotask(() => focusSearchInput?.());
        },
    });

    // 🔹 Tab scroll area handling
    let scrollContainer!: HTMLDivElement;
    let scrollInterval: number | null = null;

    /** Opens a new tab and auto-scrolls to the end */
    function addTab() {
        setTabs((tabs) => [...tabs, new Tab("C:")]);
        queueMicrotask(() => {
            scrollContainer.scrollTo({
                left: scrollContainer.scrollWidth,
                behavior: "smooth",
            });
        });
    }

    /** Closes a tab and exits the app if last tab is closed */
    async function removeTab(id: number) {
        setTabs((prev) => {
            const updated = prev.filter((tab) => tab.id !== id);
            if (updated.length === 0) {
                appWindow.close();
            } else if (currentTab()?.id === id) {
                setCurrentTab(updated[0]);
            }
            return updated;
        });
    }

    /** Duplicates a tab */
    function duplicateTab(id: number) {
        const tab = tabs.find((tab) => tab.id === id);
        if (tab) {
            setTabs((tabs) => [...tabs, new Tab(tab.workingDir)]);
        }
    }

    /** Scrolls the tab bar continuously when arrow buttons are held */
    function startScroll(direction: "left" | "right") {
        stopScroll();
        scrollInterval = window.setInterval(() => {
            scrollContainer.scrollBy({
                left: direction === "left" ? -15 : 15,
                behavior: "auto",
            });
        }, 16);
    }

    /** Stops tab bar auto-scroll */
    function stopScroll() {
        if (scrollInterval !== null) {
            clearInterval(scrollInterval);
            scrollInterval = null;
        }
    }

    onCleanup(stopScroll);

    return (
        <div class="w-full h-full flex flex-col overflow-hidden">
            {/* Top-level title bar (window controls) */}
            <TitleBar />

            {/* Tabs bar */}
            <div class="flex flex-row items-center w-full h-fit px-2 box-border border-gray-300">
                {/* Left scroll */}
                <button
                    onMouseDown={() => startScroll("left")}
                    onMouseUp={stopScroll}
                    onMouseLeave={stopScroll}
                    class="p-1 rounded hover:bg-gray-200/70 active:bg-gray-300 transition"
                >
                    <FaSolidChevronLeft class="w-3 h-3" />
                </button>

                {/* Scrollable tab container */}
                <div ref={scrollContainer} class="flex flex-row overflow-hidden grow items-center">
                    <For each={tabs}>
                        {(tab) => (
                            <TabHeading
                                currentTab={currentTab}
                                setCurrentTab={setCurrentTab}
                                removeTab={removeTab}
                                tab={tab}
                            />
                        )}
                    </For>
                </div>

                {/* Add new tab */}
                <button
                    onClick={addTab}
                    class="ml-1 px-2 py-1 rounded hover:bg-gray-200/70 active:bg-gray-300 transition flex items-center justify-center"
                    title="New Tab"
                >
                    <FaSolidPlus class="w-4 h-4" />
                </button>

                {/* Right scroll */}
                <button
                    onMouseDown={() => startScroll("right")}
                    onMouseUp={stopScroll}
                    onMouseLeave={stopScroll}
                    class="p-1 rounded hover:bg-gray-200/70 active:bg-gray-300 transition"
                >
                    <FaSolidChevronRight class="w-3 h-3" />
                </button>
            </div>

            {/* Main content area */}
            <div class="w-full h-full grow flex flex-col bg-gray-200/40 z-1">
                <NavigationBar
                    searchMode={searchMode()}
                    setSearchMode={setSearchMode}
                    searchBarMode={searchBarMode()}
                    setSearchBarMode={setSearchBarMode}
                    registerFocusHandler={(fn) => (focusSearchInput = fn)} // 👈 Link focus control
                />
            </div>
        </div>
    );
}
