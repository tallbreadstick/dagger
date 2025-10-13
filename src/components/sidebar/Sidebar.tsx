import { createSignal, createEffect, Show, For } from "solid-js";
import { getDirectoryTreeFromRoot, FileNode, resolveUserHome, resolveQuickAccess } from "../../scripts/navigation";
import type { TabEntry } from "../../App";
import { TreeNode } from "./TreeNode";
import Tab from "../../classes/Tab";
import {
    FaSolidFolder,
    FaSolidDownload,
    FaSolidDesktop,
    FaSolidPhotoFilm,
    FaSolidMusic,
    FaSolidVideo,
    FaSolidHouse
} from "solid-icons/fa";

export default function Sidebar(props: {
    currentTab?: () => TabEntry | null;
    setCurrentTab?: (entry: TabEntry) => void;
    width: number;
    setWidth: (w: number) => void;
}) {
    const [rootNode, setRootNode] = createSignal<FileNode | null>(null);
    const [treeWorkingDir, setTreeWorkingDir] = createSignal<string>("");
    const [quickAccess, setQuickAccess] = createSignal<
        { name: string; path: string; icon: any }[]
    >([]);

    // 🏠 Setup Quick Access
    createEffect(async () => {
        try {
            const map = await resolveQuickAccess();
            setQuickAccess([
                { name: "Home", path: map["Home"], icon: FaSolidHouse },
                { name: "Documents", path: map["Documents"], icon: FaSolidFolder },
                { name: "Downloads", path: map["Downloads"], icon: FaSolidDownload },
                { name: "Desktop", path: map["Desktop"], icon: FaSolidDesktop },
                { name: "Pictures", path: map["Pictures"], icon: FaSolidPhotoFilm },
                { name: "Music", path: map["Music"], icon: FaSolidMusic },
                { name: "Videos", path: map["Videos"], icon: FaSolidVideo },
            ]);
        } catch (err) {
            console.error("Failed to set Quick Access paths:", err);
        }
    });

    // 🧭 Watch current tab for directory changes
    createEffect(() => {
        const current = props.currentTab?.();
        if (current) {
            const path = current.tab.workingDir;
            getDirectoryTreeFromRoot(path)
                .then((tree) => {
                    setRootNode(tree as FileNode);
                    setTreeWorkingDir(path); // Update working dir only after tree is ready
                })
                .catch((err) => console.error("Failed to load directory tree:", err));
        }
    });

    function updateTab(entry: TabEntry, updater: (tab: Tab) => Tab) {
        if (!entry) return;
        const newTab = updater(entry.tab);
        entry.setTab(newTab);
    }

    function handleNavigate(path: string) {
        const entry = props.currentTab?.();
        if (!entry) return;

        updateTab(entry, (tab) => {
            const newTab = tab.clone();
            newTab.navigateTo(path);
            return newTab;
        });
    }

    // 🎚️ Sidebar resize handling
    let sidebarRef: HTMLDivElement | undefined;

    const startDrag = (e: MouseEvent) => {
        e.preventDefault();
        const startX = e.clientX;
        const startWidth = props.width;

        const onMouseMove = (e: MouseEvent) => {
            const delta = e.clientX - startX;
            props.setWidth(Math.max(180, startWidth + delta));
        };

        const onMouseUp = () => {
            window.removeEventListener("mousemove", onMouseMove);
            window.removeEventListener("mouseup", onMouseUp);
        };

        window.addEventListener("mousemove", onMouseMove);
        window.addEventListener("mouseup", onMouseUp);
    };

    return (
        <div
            ref={sidebarRef}
            class="h-full flex flex-col border-r font-['Outfit'] font-light border-gray-400/30 overflow-hidden bg-transparent backdrop-blur-sm relative"
            style={{ width: `${props.width}px` }}
        >
            {/* File Tree */}
            <div class="px-2 py-2 text-xs uppercase font-semibold text-gray-400 tracking-wider">
                File Tree
            </div>

            <div class="flex-1 overflow-y-auto px-1 pb-2 custom-scrollbar" style={{ "min-height": "0" }}>
                <Show when={rootNode()} fallback={<div class="text-xs text-black pl-3">Loading...</div>}>
                    <TreeNode
                        node={rootNode()!}
                        depth={0}
                        onNavigate={handleNavigate}
                        workingDir={treeWorkingDir()}
                    />
                </Show>
            </div>

            {/* Quick Access */}
            <div class="px-2 py-2 text-xs uppercase font-semibold text-gray-400 tracking-wider border-t border-gray-400/30">
                Quick Access
            </div>
            <div class="flex flex-col px-2 pb-2">
                <For each={quickAccess()}>
                    {(item) => (
                        <div
                            class="cursor-pointer hover:bg-white/10 active:bg-white/20 px-2 py-1 rounded-md transition-colors flex items-center gap-2 text-sm text-black truncate"
                            onClick={() => handleNavigate(item.path)}
                        >
                            <item.icon class="w-3 h-3 text-gray-400" />
                            {item.name}
                        </div>
                    )}
                </For>
            </div>

            {/* Draggable handle inside Sidebar */}
            <div
                class="absolute top-0 right-0 w-1 h-full cursor-ew-resize hover:bg-white/20 active:bg-white/40 transition-colors"
                onMouseDown={startDrag}
            />
        </div>
    );
}