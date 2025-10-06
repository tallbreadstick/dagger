import { createEffect, createSignal, For, Show } from "solid-js";
import { getDirectoryTree, listDirectoryContents, FileNode } from "../../scripts/navigation";
import { FaSolidFolder, FaSolidFolderOpen, FaSolidFile } from "solid-icons/fa";
import type { TabEntry } from "../../App";

/**
 * Recursive node for displaying folders and files in the sidebar tree view
 */
function TreeNode(props: {
    node: FileNode;
    depth: number;
    onNavigate: (path: string) => void;
}) {
    const [expanded, setExpanded] = createSignal(false);
    const [children, setChildren] = createSignal<FileNode[] | null>(props.node.children ?? null);
    const [loading, setLoading] = createSignal(false);

    async function toggleExpand() {
        if (!props.node.is_dir) {
            props.onNavigate(props.node.path);
            return;
        }

        if (!expanded()) {
            setExpanded(true);
            if (children() === null) {
                setLoading(true);
                try {
                    const items = await listDirectoryContents(props.node.path);
                    const nodes: FileNode[] = items.map((item) => ({
                        name: item.name,
                        path: item.path,
                        is_dir: item.is_dir,
                        children: item.is_dir ? null : undefined,
                    }));
                    setChildren(nodes);
                } finally {
                    setLoading(false);
                }
            }
        } else {
            setExpanded(false);
        }
    }

    return (
        <div class="select-none">
            <div
                class="flex items-center gap-1 cursor-pointer hover:bg-white/10 active:bg-white/20 px-2 py-1 rounded-md transition-colors"
                style={{ "padding-left": `${props.depth * 14}px` }}
                onClick={toggleExpand}
            >
                <Show when={props.node.is_dir} fallback={<FaSolidFile class="text-gray-400 w-3 h-3" />}>
                    <Show when={expanded()} fallback={<FaSolidFolder class="text-blue-400 w-3 h-3" />}>
                        <FaSolidFolderOpen class="text-blue-400 w-3 h-3" />
                    </Show>
                </Show>
                <span class="truncate text-sm text-black">{props.node.name || props.node.path}</span>
            </div>

            <Show when={expanded()}>
                <div class="ml-1">
                    <Show when={loading()}>
                        <div class="text-xs text-gray-400 pl-5">Loading...</div>
                    </Show>
                    <For each={children()}>
                        {(child) => <TreeNode node={child} depth={props.depth + 1} onNavigate={props.onNavigate} />}
                    </For>
                </div>
            </Show>
        </div>
    );
}

/**
 * Sidebar — Progressive File Tree
 *
 * NOTE: do NOT set a fixed width here (no w-72). The parent (App) controls width.
 */
export default function Sidebar(props: {
    currentTab?: () => TabEntry | null;
    setCurrentTab?: (entry: TabEntry) => void;
}) {
    const [rootNode, setRootNode] = createSignal<FileNode | null>(null);

    createEffect(() => {
        const current = props.currentTab?.();
        if (current) {
            const path = current.tab.workingDir;
            getDirectoryTree(path, 1)
                .then(setRootNode)
                .catch((err) => console.error("Failed to load directory tree:", err));
        }
    });

    function handleNavigate(path: string) {
        const current = props.currentTab?.();
        if (!current) return;
        current.setTab("workingDir", path);
    }

    return (
        <div class="h-full flex flex-col border-r font-['Outfit'] font-light border-gray-400/30 overflow-hidden bg-transparent backdrop-blur-sm">
            <div class="px-2 py-2 text-xs uppercase font-semibold text-gray-400 tracking-wider">File Tree</div>

            <div class="flex-1 overflow-y-auto px-1 pb-2 custom-scrollbar" style={{ "min-height": "0" }}>
                <Show when={rootNode()} fallback={<div class="text-xs text-black pl-3">Loading...</div>}>
                    <TreeNode node={rootNode()!} depth={0} onNavigate={handleNavigate} />
                </Show>
            </div>
        </div>
    );
}
