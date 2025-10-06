/**
 * TabMenu.tsx
 * -------------
 * Tab UI. Expects App to own the tab state (TabEntry[] + helpers).
 */

import type { SetStoreFunction } from "solid-js/store";
import type { Accessor, Setter } from "solid-js";
import { createEffect, For, onCleanup } from "solid-js";
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
    let containerRef!: HTMLDivElement;
    let addButtonRef!: HTMLButtonElement;

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

    function onWheel(e: WheelEvent) {
        if (e.ctrlKey && scrollContainer) {
            e.preventDefault(); // prevent zooming
            scrollContainer.scrollBy({
                left: e.deltaY > 0 ? 80 : -80, // scroll direction
                behavior: "smooth",
            });
        }
    }

    onCleanup(() => scrollContainer?.removeEventListener("wheel", onWheel));

    createEffect(() => {
        scrollContainer?.addEventListener("wheel", onWheel, { passive: false });
    });

    // optional: dynamically check overflow and adjust "+" position
    const updateAddButtonPosition = () => {
        if (!containerRef || !scrollContainer || !addButtonRef) return;

        const tabsWidth = scrollContainer.scrollWidth;
        const containerWidth = containerRef.clientWidth;
        if (tabsWidth > containerWidth) {
            // overflow: pin "+" next to scroll-right
            addButtonRef.style.position = "absolute";
            addButtonRef.style.right = "30px"; // space for scroll-right button
        } else {
            // fit: normal inline
            addButtonRef.style.position = "static";
            addButtonRef.style.right = "";
        }
    };

    const onResize = () => updateAddButtonPosition();

    onCleanup(() => window.removeEventListener("resize", onResize));
    window.addEventListener("resize", onResize);

    return (
        <div class="relative flex items-center w-full h-fit px-2 box-border border-gray-300 scrollbar-none">
            {/* Scroll left */}
            <button
                onMouseDown={() => startScroll("left")}
                onMouseUp={stopScroll}
                onMouseLeave={stopScroll}
                class="p-1 rounded hover:bg-gray-200/70 active:bg-gray-300 transition"
                aria-label="Scroll tabs left"
            >
                <FaSolidChevronLeft class="w-3 h-3" />
            </button>

            {/* Scrollable tabs container */}
            <div
                ref={scrollContainer}
                class="flex flex-row overflow-x-auto grow items-center scrollbar-none relative"
                style={{ "scroll-behavior": "smooth", "scrollbar-width": "none" }}
            >
                <For each={props.tabs}>
                    {(entry) => (
                        <TabHeading
                            tab={entry.tab}
                            currentTab={() => props.currentTab()?.tab ?? null}
                            setCurrentTab={(tab: Tab | null) => {
                                if (!tab) return props.setCurrentTab(null);
                                const match = props.tabs.find((t) => t.tab.id === tab.id);
                                if (match) props.setCurrentTab(match);
                            }}
                            removeTab={props.removeTab}
                        />
                    )}
                </For>

                {/* Inline "+" button inside scroll container */}
                <button
                    onClick={() => props.addTab()}
                    class="ml-1 px-2 py-1 rounded hover:bg-gray-200/70 active:bg-gray-300 transition flex items-center justify-center shrink-0"
                    title="New Tab"
                >
                    <FaSolidPlus class="w-4 h-4" />
                </button>
            </div>

            {/* Scroll right */}
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
