import {
    createSignal,
    createEffect,
    For,
    Show,
    onCleanup,
} from "solid-js";
import { Portal } from "solid-js/web";
import { invoke } from "@tauri-apps/api/core";
import { FaSolidChevronRight } from "solid-icons/fa";
import { listDirectoryContents } from "../../scripts/navigation";
import type { FileNode } from "../../scripts/navigation";

export default function PathBar(props: {
    currentPath: string;
    onNavigate: (newPath: string) => void;
}) {
    const [segments, setSegments] = createSignal<string[]>([]);
    const [editMode, setEditMode] = createSignal(false);
    const [inputValue, setInputValue] = createSignal("");
    const [dropdownIndex, setDropdownIndex] = createSignal<number | null>(null);
    const [subfolders, setSubfolders] = createSignal<FileNode[]>([]);
    const [dropdownPosition, setDropdownPosition] = createSignal<{
        x: number;
        y: number;
    } | null>(null);
    let inputRef: HTMLInputElement | undefined;

    // Parse current path into segments
    createEffect(() => {
        const path = props.currentPath;
        if (!editMode()) {
            const parts = path
                .replace(/\\/g, "/")
                .split("/")
                .filter((p) => p.length > 0);
            setSegments(parts);
        }
    });

    // Handle outside clicks for edit mode & dropdown
    const handleClickOutside = (e: MouseEvent) => {
        const target = e.target as HTMLElement;
        const insidePathBar = target.closest(".path-bar-container");
        const insideDropdown = target.closest(".pathbar-dropdown-portal");
        if (!insidePathBar && !insideDropdown) {
            setDropdownIndex(null);
            if (editMode()) {
                setEditMode(false);
                setInputValue("");
            }
        }
    };
    document.addEventListener("click", handleClickOutside);
    onCleanup(() => document.removeEventListener("click", handleClickOutside));

    const handleSegmentClick = (index: number) => {
        const parts = segments().slice(0, index + 1);
        const newPath = parts.join("/");
        props.onNavigate(newPath);
    };

    const toggleDropdown = async (index: number, event: MouseEvent) => {
        event.stopPropagation();

        if (dropdownIndex() === index) {
            setDropdownIndex(null);
            return;
        }

        const parts = segments().slice(0, index + 1);
        let basePath = parts.join("/");

        if (/^[A-Za-z]:$/.test(basePath)) {
            basePath = basePath + "/";
        }

        try {
            const contents = await listDirectoryContents(basePath);
            const folders = contents.filter((item) => item.is_dir);
            setSubfolders(folders);
            setDropdownIndex(index);

            const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
            setDropdownPosition({
                x: rect.left,
                y: rect.bottom + window.scrollY,
            });
        } catch (err) {
            console.error("Failed to list subfolders:", err);
        }
    };

    const handleInputKey = async (e: KeyboardEvent) => {
        if (e.key === "Enter") {
            const cmd = inputValue().trim();
            if (cmd.length === 0) {
                setEditMode(false);
                return;
            }

            try {
                const result = await invoke<{ kind: string; value: string }>(
                    "resolve_path_command",
                    { command: cmd },
                );

                if (result.kind === "path") {
                    props.onNavigate(result.value);
                } else if (result.kind === "action") {
                    console.log(result.value);
                }
            } catch (err) {
                console.error("Path command failed:", err);
            } finally {
                setEditMode(false);
            }
        } else if (e.key === "Escape") {
            setEditMode(false);
        }
    };

    // Auto-select entire path when entering edit mode
    createEffect(() => {
        if (editMode() && inputRef) {
            queueMicrotask(() => {
                inputRef?.focus();
                inputRef.value = props.currentPath;
                inputRef?.select();
            });
        }
    });

    return (
        <div
            class="path-bar-container relative flex flex-row items-center bg-gray-200 border border-gray-300 rounded px-2 py-1 w-full font-outfit"
        >
            {/* Segmented Path View */}
            <Show
                when={!editMode()}
                fallback={
                    <input
                        ref={inputRef}
                        type="text"
                        value={inputValue()}
                        onInput={(e) => setInputValue(e.currentTarget.value)}
                        onKeyDown={handleInputKey}
                        class="w-full bg-transparent outline-none text-sm text-gray-900 font-outfit px-1 py-0.5"
                    />
                }
            >
                <div
                    class="flex flex-row items-center overflow-x-auto no-scrollbar w-full cursor-text"
                    onDblClick={(e) => {
                        // Only trigger edit if the click is NOT on a segment or chevron
                        const target = e.target as HTMLElement;
                        if (
                            !target.closest(".path-segment") &&
                            !target.closest(".path-chevron")
                        ) {
                            e.stopPropagation();
                            setEditMode(true); // your function to enable editing
                        }
                    }}
                >
                    <For each={segments()}>
                        {(segment, i) => (
                            <div class="flex flex-row items-center">
                                <button
                                    onClick={(e) => {
                                        e.stopPropagation();
                                        handleSegmentClick(i());
                                    }}
                                    class="path-segment text-sm text-gray-800 font-medium px-1.5 py-0.5 rounded hover:bg-gray-300 hover:text-black transition-colors font-outfit"
                                >
                                    {segment}
                                </button>

                                <Show when={i() < segments().length - 1}>
                                    <button
                                        class="path-chevron px-1 rounded hover:bg-gray-300 transition-colors"
                                        onClick={(e) => toggleDropdown(i(), e)}
                                    >
                                        <FaSolidChevronRight class="w-3.5 h-3.5 text-gray-600 hover:text-gray-800 transition-colors" />
                                    </button>
                                </Show>
                            </div>
                        )}
                    </For>
                </div>

            </Show>

            {/* Dropdown Portal */}
            <Show when={dropdownIndex() !== null && dropdownPosition()}>
                <Portal>
                    <div
                        class="pathbar-dropdown-portal absolute bg-white border border-gray-300 rounded shadow-lg z-[9999] overflow-y-auto font-outfit"
                        style={{
                            left: `${dropdownPosition()!.x}px`,
                            top: `${dropdownPosition()!.y}px`,
                            width: "180px",
                            "max-height": "200px",
                        }}
                    >
                        <style>
                            {`
							.pathbar-dropdown-portal::-webkit-scrollbar {
								width: 6px;
							}
							.pathbar-dropdown-portal::-webkit-scrollbar-thumb {
								background-color: #ccc;
								border-radius: 3px;
							}
							.pathbar-dropdown-portal::-webkit-scrollbar-track {
								background: transparent;
							}
							`}
                        </style>

                        <For each={subfolders()}>
                            {(folder) => (
                                <div
                                    class="px-3 py-1 hover:bg-gray-100 cursor-pointer text-sm text-gray-800 select-none text-ellipsis overflow-hidden whitespace-nowrap"
                                    onClick={(e) => {
                                        e.stopPropagation();
                                        setDropdownIndex(null);
                                        props.onNavigate(folder.path);
                                    }}
                                    title={folder.name}
                                >
                                    {folder.name}
                                </div>
                            )}
                        </For>
                    </div>
                </Portal>
            </Show>
        </div>
    );
}
