import { Accessor, Component, createMemo } from "solid-js";
import Tab from "../../classes/Tab";
import { FaSolidXmark } from "solid-icons/fa";

const TabHeading: Component<{
    currentTab: Accessor<Tab | null>;
    setCurrentTab: (tab: Tab | null) => void;
    tab: Tab;
    removeTab: (id: number) => void;
}> = ({ currentTab, setCurrentTab, tab, removeTab }) => {

    const name = createMemo(() => {
        // Split by either '\' or '/' to handle Windows + POSIX paths
        const tokens = tab.workingDir.split(/[\\/]/).filter(Boolean);
        return tokens.length > 0
            ? tokens[tokens.length - 1]
            : (tab.workingDir.trim() === "" ? "/" : tab.workingDir);
    });

    function selectTab() {
        setCurrentTab(tab);
    }

    const isActive = () => currentTab()?.id === tab.id;

    return (
        <div class="group relative">
            <div
                onClick={selectTab}
                class={
                    "relative min-w-40 max-w-50 px-3 py-1 pr-6 box-border cursor-pointer " +
                    (isActive()
                        ? "bg-gray-200/40 upper-shadow rounded-tl-md rounded-tr-md z-10"
                        : "z-0 hover:bg-gray-300/40 rounded-tl-md rounded-tr-md")
                }
            >
                <label
                    class={
                        "text-sm select-none whitespace-nowrap overflow-hidden text-ellipsis pr-5 block " +
                        (isActive() ? "text-black" : "text-gray-700")
                    }
                    title={name()} // 🔁 dynamic
                >
                    {name()} {/* 🔁 dynamic */}
                </label>

                {/* Close (X) button */}
                <button
                    class="absolute right-1 top-1/2 -translate-y-1/2 w-4 h-4 flex items-center justify-center 
                 text-gray-500 hover:text-red-600 opacity-0 group-hover:opacity-100 transition"
                    onClick={(e) => {
                        e.stopPropagation(); // prevent tab selection
                        removeTab(tab.id);
                    }}
                    title="Close Tab"
                >
                    <FaSolidXmark class="w-3 h-3" />
                </button>
            </div>
        </div>
    );
};

export default TabHeading;
