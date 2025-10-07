/**
 * App.tsx
 * ----------
 * Main application root for the Tauri + SolidJS file explorer.
 */

import { createSignal, onCleanup, onMount } from "solid-js";
import "./index.css";

import TitleBar from "./components/window/TitleBar";
import Sidebar from "./components/sidebar/Sidebar";
import NavigationBar from "./components/navigation/NavigationBar";
import TabMenu from "./components/tabbing/TabMenu";
import { useWindowFocusEvents } from "./scripts/events";
import { useGlobalShortcuts } from "./scripts/shortcuts";
import { createStore, SetStoreFunction } from "solid-js/store";
import Tab from "./classes/Tab";
import ActionBar from "./components/content/ActionBar";

/** TabEntry: a store-proxied Tab plus its setTab setter */
export type TabEntry = {
    tab: Tab;                      // proxied store object returned by createStore(new Tab(...))
    setTab: SetStoreFunction<Tab>;  // setter returned by createStore
};

export default function App() {

    useWindowFocusEvents();

    // 🧱 Sidebar width + constraints
    const [sidebarWidth, setSidebarWidth] = createSignal(288); // default 72 * 4
    const minWidth = 180;
    const maxWidth = 450;

    // dragging state
    let sidebarRef: HTMLDivElement | undefined;
    let isResizing = false;

    function startResize(e: MouseEvent) {
        isResizing = true;
        e.preventDefault();
        document.body.style.cursor = "col-resize";
    }

    function stopResize() {
        if (isResizing) {
            isResizing = false;
            document.body.style.cursor = "";
        }
    }

    function handleResize(e: MouseEvent) {
        if (!isResizing || !sidebarRef) return;
        const sidebarLeft = sidebarRef.getBoundingClientRect().left;
        const newWidth = Math.min(Math.max(e.clientX - sidebarLeft, minWidth), maxWidth);
        setSidebarWidth(newWidth);
    }

    // global listeners for smooth dragging
    onMount(() => {
        window.addEventListener("mousemove", handleResize);
        window.addEventListener("mouseup", stopResize);

        onCleanup(() => {
            window.removeEventListener("mousemove", handleResize);
            window.removeEventListener("mouseup", stopResize);
        });
    });

    // factory to create a TabEntry
    function makeTab(path: string): TabEntry {
        const [tab, setTab] = createStore(new Tab(path));
        return { tab, setTab };
    }

    // tabs array (store) — array of TabEntry
    const [tabs, setTabs] = createStore<TabEntry[]>([
        makeTab("C:\\Users\\ACER\\Documents"),
    ]);

    // current tab points to a TabEntry (or null)
    const [currentTab, setCurrentTab] = createSignal<TabEntry | null>(tabs[0] ?? null);

    // helpers: add/close/duplicate tabs — App owns state
    function addTab(path = "C:") {
        const entry = makeTab(path);
        setTabs((prev) => [...prev, entry]);
        // Important: after store updates, re-bind currentTab
        queueMicrotask(() => {
            const index = tabs.length - 1;
            setCurrentTab(tabs[index]); // point to reactive version
        });
    }

    function removeTabById(id: number) {
        setTabs((prev) => {
            const updated = prev.filter((e) => e.tab.id !== id);
            // if current was removed, set new current
            const isCurrentRemoved = currentTab()?.tab.id === id;
            if (updated.length === 0) {
                // close window if no tabs (same behavior you had)
                // note: getCurrentWindow used previously; import and use if needed
            } else if (isCurrentRemoved) {
                setCurrentTab(updated[0]);
            }
            return updated;
        });
    }

    function duplicateTabById(id: number) {
        const entry = tabs.find((e) => e.tab.id === id);
        if (!entry) return;
        const newEntry = makeTab(entry.tab.workingDir);
        // copy history if you want:
        // newEntry.setTab("backStack", [...entry.tab.backStack]); // optional
        setTabs((prev) => [...prev, newEntry]);
        setCurrentTab(newEntry);
    }

    // 🔹 Search state
    const [searchMode, setSearchMode] = createSignal(false);
    const [searchBarMode, setSearchBarMode] = createSignal<
        "text" | "image" | "audio" | "document"
    >("text");

    let focusSearchInput: (() => void) | null = null;

    useGlobalShortcuts({
        toggleSearchMode: () => {
            setSearchMode((prev) => !prev);
            queueMicrotask(() => focusSearchInput?.());
        },
        setSearchMode: (mode) => {
            setSearchBarMode(mode);
            setSearchMode(true);
            queueMicrotask(() => focusSearchInput?.());
        },
    });

    return (
        <div class="w-full h-full flex flex-col overflow-hidden">
            <TitleBar />

            <TabMenu
                tabs={tabs}
                setTabs={setTabs}
                currentTab={currentTab}
                setCurrentTab={setCurrentTab}
                addTab={() => addTab()}
                removeTab={(id: number) => removeTabById(id)}
                duplicateTab={(id: number) => duplicateTabById(id)}
            />

            <div class="flex flex-col flex-1 min-h-0 bg-gray-200/40 z-1">
                {/* NavigationBar — fixed height */}
                <NavigationBar
                    currentTabEntry={currentTab}
                    setCurrentTabEntry={setCurrentTab}
                    searchMode={searchMode()}
                    setSearchMode={setSearchMode}
                    searchBarMode={searchBarMode()}
                    setSearchBarMode={setSearchBarMode}
                    registerFocusHandler={(fn) => (focusSearchInput = fn)}
                />

                <div class="flex flex-row flex-1 min-h-0 overflow-hidden select-none">
                    {/* Sidebar + Resizer group */}
                    <div class="flex flex-row flex-1 min-h-0 overflow-hidden select-none">
                        {/* Sidebar container */}
                        <div
                            ref={sidebarRef}
                            class="flex flex-row flex-shrink-0 border-r border-gray-400/30"
                            style={{
                                width: `${sidebarWidth()}px`,
                                "min-width": `${minWidth}px`,
                                "max-width": `${maxWidth}px`,
                                transition: isResizing ? "none" : "width 0.1s ease-out",
                            }}
                        >
                            <Sidebar
                                currentTab={currentTab}
                                setCurrentTab={setCurrentTab}
                                width={sidebarWidth()}
                                setWidth={setSidebarWidth}
                            />
                            {/* Resizer handle — inside same flex group for consistent layout */}
                            {/* <div
                                onMouseDown={startResize}
                                class="w-1 cursor-col-resize bg-transparent hover:bg-white/20 active:bg-white/40 transition-colors duration-150"
                                style={{
                                    "user-select": "none",
                                    "touch-action": "none",
                                }}
                            /> */}
                        </div>

                        {/* Main content area */}
                        <div class="flex-1 min-w-0 overflow-auto flex flex-col">
                            <ActionBar />
                            {/* ContentPanel below */}
                            <div class="flex-1 overflow-auto">
                                {/* Content panel code goes here */}
                            </div>
                        </div>
                    </div>
                </div>
            </div>

        </div>
    );
}
