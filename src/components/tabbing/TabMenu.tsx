/**
 * TabMenu.tsx
 * -------------
 * Tab UI. Expects App to own the tab state (TabEntry[] + helpers).
 */

import type { SetStoreFunction } from "solid-js/store";
import type { Accessor, Setter } from "solid-js";
import { For, onCleanup } from "solid-js";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { FaSolidPlus, FaSolidChevronLeft, FaSolidChevronRight } from "solid-icons/fa";

import Tab from "../../classes/Tab";
import TabHeading from "./TabHeading";
import type { TabEntry } from "../../App";

export interface TabMenuProps {
    tabs: TabEntry[];
    setTabs: SetStoreFunction<TabEntry[]>;
    currentTab: Accessor<TabEntry | null>;
    setCurrentTab: Setter<TabEntry | null>;

    // App-provided helpers (App owns state)
    addTab: (path?: string) => void;
    removeTab: (id: number) => void;
    duplicateTab: (id: number) => void;
}

export default function TabMenu(props: TabMenuProps) {

    let scrollContainer!: HTMLDivElement;
    let scrollInterval: number | null = null;

    function startScroll(direction: "left" | "right") {
        stopScroll();
        scrollInterval = window.setInterval(() => {
            scrollContainer.scrollBy({
                left: direction === "left" ? -15 : 15,
                behavior: "auto",
            });
        }, 16);
    }
    function stopScroll() {
        if (scrollInterval !== null) {
            clearInterval(scrollInterval);
            scrollInterval = null;
        }
    }
    onCleanup(stopScroll);

    return (
        <div class="flex flex-row items-center w-full h-fit px-2 box-border border-gray-300">
            <button
                onMouseDown={() => startScroll("left")}
                onMouseUp={stopScroll}
                onMouseLeave={stopScroll}
                class="p-1 rounded hover:bg-gray-200/70 active:bg-gray-300 transition"
                aria-label="Scroll tabs left"
            >
                <FaSolidChevronLeft class="w-3 h-3" />
            </button>

            <div ref={scrollContainer} class="flex flex-row overflow-hidden grow items-center">
                <For each={props.tabs}>
                    {(entry) => (
                        <TabHeading
                            tab={entry.tab}
                            // give TabHeading the current Tab (not the TabEntry)
                            currentTab={() => props.currentTab()?.tab ?? null}
                            // TabHeading will call setCurrentTab with (tab: Tab | null).
                            // Map that Tab back to the matching TabEntry and set it in App.
                            setCurrentTab={(tab: Tab | null) => {
                                if (!tab) {
                                    props.setCurrentTab(null);
                                    return;
                                }
                                const match = props.tabs.find((t) => t.tab.id === tab.id);
                                if (match) props.setCurrentTab(match);
                            }}
                            // pass App-owned helpers
                            removeTab={props.removeTab}
                            // duplicateTab={() => props.duplicateTab(entry.tab.id)}
                        />
                    )}
                </For>
            </div>

            <button
                onClick={() => props.addTab()}
                class="ml-1 px-2 py-1 rounded hover:bg-gray-200/70 active:bg-gray-300 transition flex items-center justify-center"
                title="New Tab"
            >
                <FaSolidPlus class="w-4 h-4" />
            </button>

            <button
                onMouseDown={() => startScroll("right")}
                onMouseUp={stopScroll}
                onMouseLeave={stopScroll}
                class="p-1 rounded hover:bg-gray-200/70 active:bg-gray-300 transition"
                aria-label="Scroll tabs right"
            >
                <FaSolidChevronRight class="w-3 h-3" />
            </button>
        </div>
    );
}
