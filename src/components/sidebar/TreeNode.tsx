import { createEffect, createSignal, For, Show } from "solid-js";
import { listDirectoryContents, FileNode } from "../../scripts/navigation";
import { FaSolidFolder, FaSolidFolderOpen, FaSolidFile } from "solid-icons/fa";

export function TreeNode(props: {
    node: FileNode;
    depth: number;
    onNavigate: (path: string) => void;
    workingDir: string;
    parentChildren?: () => FileNode[];
    parentSetChildren?: (children: FileNode[]) => void;
}) {
    const [expanded, setExpanded] = createSignal(false);
    const [children, setChildren] = createSignal<FileNode[]>(props.node.children ?? []);
    const [loading, setLoading] = createSignal(false);

    createEffect(() => {
        if (props.workingDir.startsWith(props.node.path)) {
            setExpanded(true);
        } else {
            setExpanded(false);
        }
    });

    async function loadChildren() {
        if (!props.node.is_dir) return;
        setLoading(true);
        try {
            const items = await listDirectoryContents(props.node.path);
            const nodes: FileNode[] = items.map((item) => ({
                name: item.name,
                path: item.path,
                is_dir: item.is_dir,
                children: item.is_dir ? [] : undefined,
            }));
            setChildren(nodes);
        } catch (err) {
            console.error("Error loading children:", err);
            setChildren([]);
        } finally {
            setLoading(false);
        }
    }

    function toggleExpand(e: MouseEvent) {
        e.stopPropagation();
        if (!props.node.is_dir) return;

        if (!expanded()) {
            // Collapse siblings if parent provided
            if (props.parentChildren && props.parentSetChildren) {
                const siblings = props.parentChildren();
                const updatedSiblings = siblings.map((sib) => {
                    if (sib !== props.node && sib.children) {
                        return { ...sib }; // placeholder, actual collapse handled in child signals
                    }
                    return sib;
                });
                props.parentSetChildren(updatedSiblings);
            }

            setExpanded(true);
            if (children().length === 0) {
                loadChildren();
            }
        } else {
            setExpanded(false);
        }
    }

    function handleDoubleClick(e: MouseEvent) {
        e.stopPropagation();
        if (props.node.is_dir) props.onNavigate(props.node.path);
    }

    return (
        <div class="select-none">
            <div
                class="flex items-center gap-1 cursor-pointer hover:bg-white/10 active:bg-white/20 px-2 py-1 rounded-md transition-colors"
                style={{ "padding-left": `${props.depth * 14}px` }}
                onClick={toggleExpand}
                onDblClick={handleDoubleClick}
            >
                <Show
                    when={props.node.is_dir}
                    fallback={<FaSolidFile class="text-gray-500 w-3 h-3" />}
                >
                    <Show when={expanded()} fallback={<FaSolidFolder class="text-gray-50 w-3 h-3" />}>
                        <FaSolidFolderOpen class="text-gray-50 w-3 h-3" />
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
                        {(child) => (
                            <TreeNode
                                node={child}
                                depth={props.depth + 1}
                                onNavigate={props.onNavigate}
                                workingDir={props.workingDir}
                                parentChildren={children}
                                parentSetChildren={setChildren}
                            />
                        )}
                    </For>
                </div>
            </Show>
        </div>
    );
}
