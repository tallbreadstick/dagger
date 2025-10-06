import { createEffect, createSignal, For, Show } from "solid-js";
import { listDirectoryContents, FileNode } from "../../scripts/navigation";
import {
    FaSolidFolder,
    FaSolidFolderOpen,
    FaSolidFile,
    FaSolidFilePdf,
    FaSolidFileWord,
    FaSolidFileExcel,
    FaSolidBoxArchive,
    FaSolidFileAudio,
    FaSolidFileVideo,
    FaSolidFileImage,
    FaSolidFileCode,
} from "solid-icons/fa";
import { openPath } from "@tauri-apps/plugin-opener";

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
            if (props.parentChildren && props.parentSetChildren) {
                const siblings = props.parentChildren();
                const updatedSiblings = siblings.map((sib) => {
                    if (sib !== props.node && sib.children) {
                        return { ...sib }; // actual collapse handled by child signals
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
        if (props.node.is_dir) {
            props.onNavigate(props.node.path);
        } else {
            // Open non-directory files
            openPath(props.node.path).catch((err) => {
                console.error("Failed to open file:", err);
            });
        }
    }

    // Return the proper icon component for a file based on its extension
    function getFileIcon(name: string) {
        const ext = name.split(".").pop()?.toLowerCase() ?? "";

        const docExts = ["pdf", "doc", "docx", "odt", "txt", "rtf", "md"];
        const sheetExts = ["xls", "xlsx", "csv", "ods"];
        const videoExts = ["mp4", "mov", "m4v", "mkv", "avi", "webm"];
        const audioExts = ["mp3", "wav", "ogg", "m4a", "flac", "aac"];
        const imageExts = ["png", "jpg", "jpeg", "gif", "bmp", "webp", "tiff", "svg"];
        const archiveExts = ["zip", "7z", "rar", "tar", "gz", "bz2", "xz"];
        const execExts = ["exe", "msi", "jar", "bat", "sh", "app", "bin"];

        if (docExts.includes(ext)) return <FaSolidFileWord class="text-blue-400 w-3 h-3" />;
        if (sheetExts.includes(ext)) return <FaSolidFileExcel class="text-green-400 w-3 h-3" />;
        if (videoExts.includes(ext)) return <FaSolidFileVideo class="text-purple-400 w-3 h-3" />;
        if (audioExts.includes(ext)) return <FaSolidFileAudio class="text-indigo-400 w-3 h-3" />;
        if (imageExts.includes(ext)) return <FaSolidFileImage class="text-pink-400 w-3 h-3" />;
        if (archiveExts.includes(ext)) return <FaSolidBoxArchive class="text-yellow-400 w-3 h-3" />;
        if (execExts.includes(ext)) return <FaSolidFileCode class="text-red-400 w-3 h-3" />;

        return <FaSolidFile class="text-gray-500 w-3 h-3" />; // fallback
    }

    return (
        <div class="select-none">
            <div
                class="flex items-center gap-1 cursor-pointer hover:bg-white/10 active:bg-white/20 px-2 py-1 rounded-md transition-colors"
                style={{ "padding-left": `${props.depth * 14}px` }}
                onClick={toggleExpand}
                onDblClick={handleDoubleClick}
            >
                <Show when={props.node.is_dir} fallback={getFileIcon(props.node.name)}>
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
