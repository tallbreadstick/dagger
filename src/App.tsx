import { createStore } from "solid-js/store";
import { createSignal, For } from "solid-js";
import { onCleanup } from "solid-js";
import "./index.css";
import Tab from "./classes/Tab";
import TabHeading from "./components/tabbing/TabHeading";

import { FaSolidPlus, FaSolidChevronLeft, FaSolidChevronRight } from "solid-icons/fa";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Sidebar from "./components/sidebar/Sidebar";
import NavigationBar from "./components/navigation/NavigationBar";
import TitleBar from "./components/window/TitleBar";

export default function App() {

    const appWindow = getCurrentWindow();

    const [tabs, setTabs] = createStore([
        new Tab("C:\\Program Files"),
        new Tab("C:\\Program Files (x86)"),
        new Tab("C:\\Users\\Owner"),
        new Tab("C:\\CSIT327 - Information Management 2"),
        new Tab("C:\\Temp")
    ]);

    const [currentTab, setCurrentTab] = createSignal<Tab | null>(tabs[0] || null);

    let scrollContainer!: HTMLDivElement;
    let scrollInterval: number | null = null;

    function addTab() {
        setTabs(tabs => [...tabs, new Tab("C:")]);
        // auto-scroll to right when new tab is added
        queueMicrotask(() => {
            scrollContainer.scrollTo({ left: scrollContainer.scrollWidth, behavior: "smooth" });
        });
    }

    async function removeTab(id: number) {
        setTabs(prev => {
            const updated = prev.filter(tab => tab.id !== id);

            // if no tabs left, close the app
            if (updated.length === 0) {
                appWindow.close();
            } else {
                // if current tab was closed, switch to first tab
                if (currentTab()?.id === id) {
                    setCurrentTab(updated[0]);
                }
            }

            return updated;
        });
    }

    function duplicateTab(id: number) {
        const tab = tabs.find(tab => tab.id === id);
        if (tab) {
            setTabs(tabs => [...tabs, new Tab(tab.workingDir)]);
        }
    }

    function startScroll(direction: "left" | "right") {
        stopScroll();
        scrollInterval = window.setInterval(() => {
            scrollContainer.scrollBy({
                left: direction === "left" ? -15 : 15,
                behavior: "auto"
            });
        }, 16); // ~60fps
    }

    function stopScroll() {
        if (scrollInterval !== null) {
            clearInterval(scrollInterval);
            scrollInterval = null;
        }
    }

    // Ensure interval clears if component unmounts
    onCleanup(stopScroll);

    return (
        <div class="w-full h-full flex flex-col overflow-hidden">
            {/* Title bar */}
            <TitleBar />
            {/* Tab bar */}
            <div class="flex flex-row items-center w-full h-fit px-2 box-border border-gray-300">
                {/* Left arrow */}
                <button
                    onMouseDown={() => startScroll("left")}
                    onMouseUp={stopScroll}
                    onMouseLeave={stopScroll}
                    class="p-1 rounded hover:bg-gray-200/70 active:bg-gray-300 transition"
                >
                    <FaSolidChevronLeft class="w-3 h-3" />
                </button>

                {/* Scrollable area */}
                <div
                    ref={scrollContainer}
                    class="flex flex-row overflow-hidden grow items-center"
                >
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

                {/* Add new tab button */}
                <button
                    onClick={addTab}
                    class="ml-1 px-2 py-1 rounded hover:bg-gray-200/70 active:bg-gray-300 transition flex items-center justify-center"
                    title="New Tab"
                >
                    <FaSolidPlus class="w-4 h-4" />
                </button>

                {/* Right arrow */}
                <button
                    onMouseDown={() => startScroll("right")}
                    onMouseUp={stopScroll}
                    onMouseLeave={stopScroll}
                    class="p-1 rounded hover:bg-gray-200/70 active:bg-gray-300 transition"
                >
                    <FaSolidChevronRight class="w-3 h-3" />
                </button>
            </div>

            {/* Content area */}
            <div class="w-full h-full grow flex flex-col bg-gray-200/40 z-1">
                {/* Sidebar */}
                <NavigationBar />
            </div>
        </div>
    );
}
