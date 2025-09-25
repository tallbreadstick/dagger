import { Accessor, Component, Setter } from "solid-js";
import Tab from "../../classes/Tab";
import { FaSolidXmark } from "solid-icons/fa";

const TabHeading: Component<{
    currentTab: Accessor<Tab | null>,
    setCurrentTab: Setter<Tab | null>,
    tab: Tab,
    removeTab: (id: number) => void
}> = ({ currentTab, setCurrentTab, tab, removeTab }) => {
    const tokens = tab.workingDir.split("\\");
    const name = tokens[tokens.length - 1];

    function selectTab() {
        setCurrentTab(tab);
    }

    const isActive = () => currentTab() === tab;

    return (
        <div class="group relative">
            <div
                onClick={selectTab}
                class={
                    "relative w-50 px-2 py-1 pr-6 box-border cursor-pointer " +
                    (isActive()
                        ? "bg-gray-200/40 upper-shadow rounded-tl-md rounded-tr-md z-10"
                        : "z-0")
                }
            >
                {!isActive() && (
                    <div class="absolute right-0 top-1/4 bottom-0 w-px bg-gray-700" />
                )}

                <label
                    class={
                        "text-sm font-semibold select-none " +
                        (isActive() ? "text-black" : "text-gray-700")
                    }
                >
                    {name}
                </label>

                {/* Close (X) button */}
                <button
                    class="absolute right-1 top-1/2 -translate-y-1/2 w-4 h-4 flex items-center justify-center 
                           text-gray-500 hover:text-red-600 opacity-0 group-hover:opacity-100 transition"
                    onClick={(e) => {
                        e.stopPropagation(); // prevent tab selection
                        removeTab(tab.id);
                    }}
                    title="Close tab"
                >
                    <FaSolidXmark class="w-3 h-3" />
                </button>
            </div>
        </div>
    );
};

export default TabHeading;
