/**
 * SortMenu.tsx
 * ------------------
 * Portal dropdown for sorting files by various criteria.
 */

import { createSignal, onCleanup, onMount, For, Show } from "solid-js";

export default function SortMenu(props: { onClose: () => void; x: number; y: number }) {
    const [sortBy, setSortBy] = createSignal("name");
    const [ascending, setAscending] = createSignal(true);

    const options = [
        { key: "name", label: "Name" },
        { key: "size", label: "Size" },
        { key: "type", label: "Type" },
        { key: "modified", label: "Date Modified" },
    ];

    onMount(() => {
        const handleEsc = (e: KeyboardEvent) => e.key === "Escape" && props.onClose();
        window.addEventListener("keydown", handleEsc);
        onCleanup(() => window.removeEventListener("keydown", handleEsc));
    });

    return (
        <div
            class="actionbar-menu fixed w-56 rounded-md shadow-md bg-white border border-gray-300 flex flex-col z-50"
            style={{
                top: `${props.y + 4}px`,
                left: `${props.x}px`,
            }}
        >
            <div class="p-2 text-xs font-semibold text-gray-600 uppercase tracking-wide border-b border-gray-200">
                Sort By
            </div>

            <For each={options}>
                {(opt) => (
                    <button
                        onClick={() => setSortBy(opt.key)}
                        class={`flex justify-between items-center px-3 py-1.5 text-sm hover:bg-gray-100 ${sortBy() === opt.key ? "bg-gray-100 font-medium" : ""
                            }`}
                    >
                        {opt.label}
                        <Show when={sortBy() === opt.key}>
                            <span class="text-gray-500 text-xs">✓</span>
                        </Show>
                    </button>
                )}
            </For>

            <div class="border-t border-gray-200 mt-1 pt-1">
                <button
                    onClick={() => setAscending((prev) => !prev)}
                    class="flex w-full justify-between items-center px-3 py-1.5 text-sm hover:bg-gray-100"
                >
                    <span>Order</span>
                    <span class="text-gray-500 text-xs text-nowrap ml-auto">
                        {ascending() ? "Ascending ↑" : "Descending ↓"}
                    </span>
                </button>
            </div>
        </div>
    );
}
