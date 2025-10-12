/**
 * ActionBar.tsx
 * ------------------
 * File actions toolbar sitting above the content panel.
 */

import { createSignal, Show, For } from "solid-js";
import { Portal } from "solid-js/web";
import {
    FaSolidPlus,
    FaSolidScissors,
    FaSolidCopy,
    FaSolidPaste,
    FaSolidTrash,
    FaSolidSort,
    FaSolidEye,
    FaSolidPen,
    FaSolidTableCellsLarge,
    FaSolidTableCells,
    FaSolidTableList,
} from "solid-icons/fa";
import NewFileMenu from "./NewFileMenu";
import SortMenu from "./SortMenu";
import ViewMenu from "./ViewMenu";

export default function ActionBar(props: {
    sortKey: 'name' | 'size' | 'filetype' | 'date_modified';
    setSortKey: (key: 'name' | 'size' | 'filetype' | 'date_modified') => void;
    ascending: boolean;
    setAscending: (v: boolean) => void;
    viewMode: 'grid' | 'list';
    setViewMode: (v: 'grid' | 'list') => void;
    showHidden: boolean;
    setShowHidden: (v: boolean) => void;
    showExtensions: boolean;
    setShowExtensions: (v: boolean) => void;
    iconSize: 'small' | 'medium';
    setIconSize: (v: 'small' | 'medium') => void;
}) {
    const [openMenu, setOpenMenu] = createSignal<null | "new" | "sort" | "view">(null);
    const [menuPos, setMenuPos] = createSignal<{ x: number; y: number }>({ x: 0, y: 0 });

    function toggleMenu(type: "new" | "sort" | "view", e: MouseEvent) {
        const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
        setMenuPos({ x: rect.left, y: rect.bottom });
        setOpenMenu(openMenu() === type ? null : type);
    }

    function handleClickAway(e: MouseEvent) {
        const target = e.target as HTMLElement;
        if (!target.closest(".actionbar-menu") && !target.closest(".action-button")) {
            setOpenMenu(null);
        }
    }
    document.addEventListener("click", handleClickAway);

    const iconPresets = [
        { key: 'small-grid', icon: <FaSolidTableCells />, set: () => {
            props.setIconSize('small');
            props.setViewMode('grid');
        } },
        { key: 'medium-grid', icon: <FaSolidTableCellsLarge />, set: () => {
            props.setIconSize('medium');
            props.setViewMode('grid');
        } },
        { key: 'small-list', icon: <FaSolidTableList />, set: () => {
            props.setIconSize('small');
            props.setViewMode('list');
        } },
    ] as const;

    return (
        <div class="flex items-center gap-2 px-3 py-1.5 border-b border-gray-300/50 bg-gray-100/60 backdrop-blur-md select-none z-10 justify-between">
            {/* LEFT section */}
            <div class="flex items-center gap-2">
                {/* New File */}
                <button
                    class="action-button flex items-center gap-1 px-2 py-1 rounded hover:bg-white/50 transition"
                    onClick={(e) => toggleMenu("new", e)}
                >
                    <FaSolidPlus />
                    <span class="text-sm">New</span>
                </button>

                {/* Edit actions */}
                <div class="flex items-center gap-1 border-l border-gray-400/40 pl-2">
                    <ActionIcon icon={<FaSolidScissors />} label="Cut" />
                    <ActionIcon icon={<FaSolidCopy />} label="Copy" />
                    <ActionIcon icon={<FaSolidPaste />} label="Paste" />
                    <ActionIcon icon={<FaSolidPen />} label="Rename" />
                    <ActionIcon icon={<FaSolidTrash />} label="Delete" />
                </div>

                {/* Divider */}
                <div class="w-px h-8 bg-gray-400/50" />

                {/* Sort */}
                <button
                    class="action-button flex items-center gap-1 px-2 py-1 rounded hover:bg-white/50 transition"
                    onClick={(e) => toggleMenu("sort", e)}
                >
                    <FaSolidSort />
                    <span class="text-sm">Sort</span>
                </button>

                {/* View */}
                <button
                    class="action-button flex items-center gap-1 px-2 py-1 rounded hover:bg-white/50 transition"
                    onClick={(e) => toggleMenu("view", e)}
                >
                    <FaSolidEye />
                    <span class="text-sm">View</span>
                </button>
            </div>

            {/* RIGHT section — Icon size selector */}
            <div class="flex items-center gap-1">
                <For each={iconPresets}>
                    {(opt) => (
                        <button
                            class="p-2 rounded transition hover:bg-white/50"
                            title={`Icon size: ${opt.key}`}
                            onClick={opt.set}
                        >
                            {opt.icon}
                        </button>
                    )}
                </For>
            </div>

            {/* PORTALS for menus */}
            <Portal>
                <Show when={openMenu() === "new"}>
                    <NewFileMenu
                        onClose={() => setOpenMenu(null)}
                        x={menuPos().x}
                        y={menuPos().y}
                    />
                </Show>

                <Show when={openMenu() === "sort"}>
                    <SortMenu
                        onClose={() => setOpenMenu(null)}
                        x={menuPos().x}
                        y={menuPos().y}
                        sortKey={props.sortKey as 'name' | 'size' | 'type' | 'modified'}
                        setSortKey={props.setSortKey as (k: 'name' | 'size' | 'type' | 'modified') => void}
                        ascending={props.ascending}
                        setAscending={props.setAscending}
                    />
                </Show>

                <Show when={openMenu() === "view"}>
                    <ViewMenu
                        onClose={() => setOpenMenu(null)}
                        x={menuPos().x}
                        y={menuPos().y}
                        viewMode={props.viewMode}
                        setViewMode={props.setViewMode}
                        showHidden={props.showHidden}
                        setShowHidden={props.setShowHidden}
                        showExtensions={props.showExtensions}
                        setShowExtensions={props.setShowExtensions}
                    />
                </Show>
            </Portal>
        </div>
    );
}

function ActionIcon(props: { icon: any; label: string }) {
    return (
        <button
            title={props.label}
            class="action-button p-2 rounded hover:bg-white/50 transition"
        >
            {props.icon}
        </button>
    );
}
