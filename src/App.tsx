/**
 * App.tsx
 * ----------
 * Main application root for the Tauri + SolidJS file explorer.
 */

import { createSignal } from "solid-js";
import "./index.css";

import TitleBar from "./components/window/TitleBar";
import Sidebar from "./components/sidebar/Sidebar";
import NavigationBar from "./components/navigation/NavigationBar";
import TabMenu from "./components/tabbing/TabMenu";
import { useWindowFocusEvents } from "./scripts/events";
import { useGlobalShortcuts } from "./scripts/shortcuts";
import { createStore, SetStoreFunction } from "solid-js/store";
import Tab from "./classes/Tab";

/** TabEntry: a store-proxied Tab plus its setTab setter */
export type TabEntry = {
    tab: Tab;                      // proxied store object returned by createStore(new Tab(...))
    setTab: SetStoreFunction<Tab>;  // setter returned by createStore
};

export default function App() {
    useWindowFocusEvents();

    // factory to create a TabEntry
    function makeTab(path: string): TabEntry {
        const [tab, setTab] = createStore(new Tab(path));
        return { tab, setTab };
    }

    // tabs array (store) — array of TabEntry
    const [tabs, setTabs] = createStore<TabEntry[]>([
        makeTab("C:\\Program Files"),
        makeTab("C:\\Program Files (x86)"),
        makeTab("C:\\Users\\Owner"),
        makeTab("C:\\CSIT327 - Information Management 2"),
        makeTab("C:\\Temp"),
    ]);

    // current tab points to a TabEntry (or null)
    const [currentTab, setCurrentTab] = createSignal<TabEntry | null>(tabs[0] ?? null);

    // helpers: add/close/duplicate tabs — App owns state
    function addTab(path = "C:") {
        const entry = makeTab(path);
        setTabs((prev) => [...prev, entry]);
        // make new tab the current tab
        setCurrentTab(entry);
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

            <div class="w-full h-full grow flex flex-col bg-gray-200/40 z-1">
                <NavigationBar
                    currentTabEntry={currentTab}
                    setCurrentTabEntry={setCurrentTab}
                    searchMode={searchMode()}
                    setSearchMode={setSearchMode}
                    searchBarMode={searchBarMode()}
                    setSearchBarMode={setSearchBarMode}
                    registerFocusHandler={(fn) => (focusSearchInput = fn)}
                />

                <div class="flex flex-row grow overflow-hidden">
                    <Sidebar />
                </div>
            </div>
        </div>
    );
}
