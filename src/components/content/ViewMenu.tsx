/**
 * ViewMenu.tsx
 * ------------------
 * Portal dropdown for toggling file view modes and visual options.
 */

import { onCleanup, onMount, Show } from "solid-js";

export default function ViewMenu(props: {
    onClose: () => void;
    x: number;
    y: number;
    viewMode: 'grid' | 'list';
    setViewMode: (v: 'grid' | 'list') => void;
    showHidden: boolean;
    setShowHidden: (v: boolean) => void;
    showExtensions: boolean;
    setShowExtensions: (v: boolean) => void;
}) {

    onMount(() => {
        const handleEsc = (e: KeyboardEvent) => e.key === "Escape" && props.onClose();
        window.addEventListener("keydown", handleEsc);
        onCleanup(() => window.removeEventListener("keydown", handleEsc));
    });

    return (
        <div
            class="actionbar-menu fixed w-52 rounded-md shadow-md bg-white border border-gray-300 flex flex-col z-50"
            style={{
                top: `${props.y + 4}px`,
                left: `${props.x}px`,
            }}
        >
            <div class="p-2 text-xs font-semibold text-gray-600 uppercase tracking-wide border-b border-gray-200">
                View Mode
            </div>

            <button
                onClick={() => props.setViewMode("grid")}
                class={`flex justify-between items-center px-3 py-1.5 text-sm hover:bg-gray-100 ${props.viewMode === "grid" ? "bg-gray-100 font-medium" : ""
                    }`}
            >
                Grid View
                <Show when={props.viewMode === "grid"}>
                    <span class="text-gray-500 text-xs">✓</span>
                </Show>
            </button>

            <button
                onClick={() => props.setViewMode("list")}
                class={`flex justify-between items-center px-3 py-1.5 text-sm hover:bg-gray-100 ${props.viewMode === "list" ? "bg-gray-100 font-medium" : ""
                    }`}
            >
                List View
                <Show when={props.viewMode === "list"}>
                    <span class="text-gray-500 text-xs">✓</span>
                </Show>
            </button>

            <div class="border-t border-gray-200 mt-1 pt-1">
                <ToggleOption
                    label="Show Hidden Files"
                    checked={props.showHidden}
                    onToggle={() => props.setShowHidden(!props.showHidden)}
                />
                <ToggleOption
                    label="Show File Extensions"
                    checked={props.showExtensions}
                    onToggle={() => props.setShowExtensions(!props.showExtensions)}
                />
            </div>
        </div>
    );
}

function ToggleOption(props: { label: string; checked: boolean; onToggle: () => void }) {
    return (
        <button
            onClick={props.onToggle}
            class="flex w-full justify-between items-center px-3 py-1.5 text-sm hover:bg-gray-100"
        >
            <span>{props.label}</span>
            <span class="text-gray-500 text-xs ml-auto">{props.checked ? "✓" : ""}</span>
        </button>
    );
}
