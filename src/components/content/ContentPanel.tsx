import { createSignal, createEffect, onCleanup, Show, Accessor, Setter } from "solid-js";
import { Portal } from "solid-js/web";
import type { TabEntry } from "../../App";
import { streamDirectoryContents, FileChunk } from "../../scripts/stream";
import {
    FaSolidFile,
    FaSolidFileWord,
    FaSolidFileExcel,
    FaSolidFileVideo,
    FaSolidFileAudio,
    FaSolidFileImage,
    FaSolidBoxArchive,
    FaSolidFileCode,
    FaSolidFilePowerpoint,
    FaSolidFolder,
    FaSolidPlay,
    FaSolidArrowTurnUp
} from "solid-icons/fa";
import { LazyImage } from "../LazyImage";
import { openFromPath } from "../../scripts/navigation";
import HomeLayout from "./HomeLayout";
import StandardLayout from "./StandardLayout";

export default function ContentPanel(props: {
    currentTab: TabEntry | null;
    setCurrentTab: (entry: TabEntry) => void;
    sortKey: Accessor<'name' | 'size' | 'filetype' | 'date_modified'>;
    ascending: Accessor<boolean>;
    viewMode: Accessor<'grid' | 'list'>;
    showHidden: Accessor<boolean>;
    showExtensions: Accessor<boolean>;
    iconSize: Accessor<'small' | 'medium' | 'large'>;
    refresh?: Accessor<number>;
    setRefresh?: Setter<number>;
    selectedItems: Accessor<Set<string>>;
    setSelectedItems: Setter<Set<string>>;
    lastClickedIndex: Accessor<number | null>;
    setLastClickedIndex: Setter<number | null>;
    isDragging: Accessor<boolean>;
    setIsDragging: Setter<boolean>;
    justDragged: Accessor<boolean>;
    setJustDragged: Setter<boolean>;
}) {

    let _panelEl: HTMLDivElement;

    const [_fileMap, setFileMap] = createSignal<Map<string, FileChunk>>(new Map());
    const [files, setFiles] = createSignal<FileChunk[]>([]);
    const [loading, setLoading] = createSignal(false);
    const [error, setError] = createSignal<string | null>(null);
    const [progress, setProgress] = createSignal(0);
    const [showProgress, setShowProgress] = createSignal(false);
    let startTime: number | null = null;
    let progressTimer: number | null = null;

    let cancelStream: (() => Promise<void>) | null = null;

    // 🧩 Detect special home path
    const isHomePath = () => props.currentTab?.tab.workingDir === "Home";

    // --- STREAM LOGIC ---
    const loadDirectory = async (path: string) => {
        setFileMap(new Map());
        setFiles([]);
        setLoading(true);
        setProgress(0);
        setShowProgress(true);
        startTime = performance.now();

        const normalized = path.replace(/\\/g, "/").trim();
        const isDriveRoot = normalized === "/" || /^[A-Za-z]:\/?$/.test(normalized);

        if (progressTimer) cancelAnimationFrame(progressTimer);
        if (cancelStream) {
            await cancelStream();
            cancelStream = null;
        }

        function simulateProgress() {
            if (!startTime) return;
            const elapsed = (performance.now() - startTime) / 1000;
            const eased = 1 - Math.exp(-0.6 * elapsed);
            setProgress(Math.min(eased * 0.98, 0.98));
            progressTimer = requestAnimationFrame(simulateProgress);
        }
        progressTimer = requestAnimationFrame(simulateProgress);

        const unlisten = await streamDirectoryContents(
            path,
            (chunk: FileChunk) => {
                if (!props.showHidden && chunk.name.startsWith('.')) return;
                setFileMap(prev => {
                    const newMap = new Map(prev);
                    newMap.set(chunk.path, chunk);
                    setFiles(Array.from(newMap.values()));
                    return newMap;
                });
                if (isDriveRoot) {
                    setProgress(1);
                    setShowProgress(false);
                    setLoading(false);
                    cancelStream = null;
                }
            },
            () => setLoading(false),
            (filePath: string, thumbnail: string | null) => {
                setFileMap(prev => {
                    const newMap = new Map(prev);
                    const existing = newMap.get(filePath);
                    if (existing) {
                        newMap.set(filePath, { ...existing, thumbnail });
                        setFiles(Array.from(newMap.values()));
                    }
                    return newMap;
                });
            },
            () => {
                if (progressTimer) cancelAnimationFrame(progressTimer);
                setProgress(1);
                setTimeout(() => setShowProgress(false), 400);
                cancelStream = null;
            },
            { sortKey: props.sortKey(), ascending: props.ascending(), showHidden: props.showHidden() }
        );

        cancelStream = async () => {
            unlisten();
            if (progressTimer) cancelAnimationFrame(progressTimer);
        };
    };

    createEffect(async () => {
        const path = props.currentTab?.tab.workingDir;
        props.sortKey();
        props.ascending();
        props.showHidden();
        props.refresh?.();
        if (!path) return setFiles([]);
        await loadDirectory(path);
    });

    onCleanup(async () => {
        if (cancelStream) await cancelStream();
    });

    function getIconSize() {
        if (isHomePath()) return 'w-12 h-12';
        if (props.viewMode() === 'list') return 'w-5 h-5';
        switch (props.iconSize()) {
            case 'small':
                return 'w-8 h-8';
            case 'medium':
                return 'w-12 h-12';
            case 'large':
                return 'w-16 h-16';
        }
    }

    function getFileIcon(file: FileChunk) {
        const name = file.name;
        const iconSize = getIconSize();

        if (file.is_dir)
            return <FaSolidFolder class={`${iconSize} text-blue-300 mb-1`} />;

        const ext = name.split(".").pop()?.toLowerCase() ?? "";
        const docExts = ["pdf", "doc", "docx", "odt", "txt", "rtf", "md", "pages", "tex", "log"];
        const presExts = ["ppt", "pptx", "odp", "key", "gslides"];
        const sheetExts = ["xls", "xlsx", "csv", "ods", "numbers"];
        const videoExts = ["mp4", "mov", "m4v", "mkv", "avi", "webm", "flv", "wmv", "mpg", "mpeg", "ogv"];
        const audioExts = ["mp3", "wav", "ogg", "m4a", "flac", "aac", "wma", "aiff", "alac"];
        const imageExts = ["png", "jpg", "jpeg", "gif", "bmp", "webp", "tiff", "svg", "heic", "ico", "psd", "ai", "eps"];
        const archiveExts = ["zip", "7z", "rar", "tar", "gz", "bz2", "xz", "iso", "dmg", "cab", "lzh", "arj"];
        const execExts = ["exe", "msi", "jar", "bat", "sh", "app", "bin", "command", "run", "py", "pl", "rb"];
        const codeExts = ["js", "ts", "html", "htm", "css", "scss", "sass", "json", "xml", "yml", "yaml", "toml"];

        const isVideo = videoExts.includes(ext);
        const isImage = imageExts.includes(ext);
        const isShortcut = ext === "lnk";

        // --- 1️⃣ Thumbnail handling ---
        if ((isImage || isVideo || isShortcut) && file.thumbnail) {
            return (
                <div class="relative inline-block mb-1">
                    <LazyImage
                        src={`data:image;base64,${file.thumbnail}`}
                        alt={name}
                        class={`${iconSize} object-cover rounded`}
                    />
                    {isVideo && (
                        <div class="absolute inset-0 flex items-center justify-center">
                            <FaSolidPlay class={`text-white opacity-80 ${props.iconSize() === 'medium' || isHomePath() ? 'w-6 h-6' : 'w-3 h-3'}`} />
                        </div>
                    )}
                    {isShortcut && (
                        <div class="absolute bottom-0 right-0 p-0.5 bg-black/60 rounded-full">
                            <FaSolidArrowTurnUp class="w-3 h-3 text-white opacity-90 rotate-45" />
                        </div>
                    )}
                </div>
            );
        }

        // --- 2️⃣ Fallback icon logic ---
        if (isShortcut)
            return (
                <div class="relative inline-block mb-1">
                    <FaSolidFile class={`text-gray-400 ${iconSize}`} />
                    <div class="absolute bottom-0 right-0 p-0.5 bg-black/60 rounded-full">
                        <FaSolidArrowTurnUp class="w-3 h-3 text-white opacity-90 rotate-45" />
                    </div>
                </div>
            );

        if (docExts.includes(ext)) return <FaSolidFileWord class={`text-blue-400 ${iconSize}`} />;
        if (presExts.includes(ext)) return <FaSolidFilePowerpoint class={`text-orange-400 ${iconSize}`} />;
        if (sheetExts.includes(ext)) return <FaSolidFileExcel class={`text-green-400 ${iconSize}`} />;
        if (videoExts.includes(ext)) return <FaSolidFileVideo class={`text-purple-400 ${iconSize}`} />;
        if (audioExts.includes(ext)) return <FaSolidFileAudio class={`text-indigo-400 ${iconSize}`} />;
        if (imageExts.includes(ext)) return <FaSolidFileImage class={`text-pink-400 ${iconSize}`} />;
        if (archiveExts.includes(ext)) return <FaSolidBoxArchive class={`text-yellow-400 ${iconSize}`} />;
        if (execExts.includes(ext)) return <FaSolidFileCode class={`text-red-400 ${iconSize}`} />;
        if (codeExts.includes(ext)) return <FaSolidFileCode class={`text-gray-600 ${iconSize}`} />;

        return <FaSolidFile class={`text-gray-500 ${iconSize}`} />;
    }


    const formatDate = (t?: number) =>
        t ? new Date(t * 1000).toLocaleString() : "";

    function handleNavigate(path: string) {
        const entry = props.currentTab;
        if (!entry) return;
        const newTab = entry.tab.clone();
        newTab.navigateTo(path);
        entry.setTab(newTab);
    }

    const handleDoubleClick = (file: FileChunk) => {
        if (!props.currentTab) return;
        if (file.is_dir) handleNavigate(file.path);
        else openFromPath(file.path).catch(err => setError(err));
    };

    // --------------------------------------
    // SELECTION + DRAG HANDLERS (FIXED)
    // --------------------------------------

    const dragSelection = new Set<string>();
    let dragStartedOnItem = false;
    let dragActive = false;
    let dragClearApplied = false;
    const DRAG_THRESHOLD = 4;

    const [dragStart, setDragStart] = createSignal({ x: 0, y: 0 });
    const [dragEnd, setDragEnd] = createSignal({ x: 0, y: 0 });

    function updateSelection(newSet: Set<string>) {
        // Force Solid reactivity
        props.setSelectedItems(new Set(newSet));
    }

    function handleItemClick(file: FileChunk, index: number, e: MouseEvent) {
        if (props.justDragged()) {
            props.setJustDragged(false);
            return;
        }

        const ctrl = e.ctrlKey || e.metaKey;
        const shift = e.shiftKey;
        const current = props.selectedItems();
        const next = new Set(current);

        if (shift && props.lastClickedIndex() !== null) {
            const start = Math.min(props.lastClickedIndex()!, index);
            const end = Math.max(props.lastClickedIndex()!, index);
            const range = files().slice(start, end + 1);
            range.forEach(f => next.add(f.path));
        } else if (ctrl) {
            if (e.type === 'mouseup') return;
            if (next.has(file.path)) next.delete(file.path);
            else next.add(file.path);
        } else {
            next.clear();
            next.add(file.path);
        }

        props.setLastClickedIndex(index);
        updateSelection(next);
    }

    function handleMouseDown(e: MouseEvent) {
        if (e.button !== 0) return;
        setDragStart({ x: e.clientX, y: e.clientY });
        setDragEnd({ x: e.clientX, y: e.clientY });

        props.setIsDragging(true);
        props.setJustDragged(false);
        dragSelection.clear();
        dragClearApplied = false;
        dragStartedOnItem = !!(e.target as HTMLElement)?.closest(".selectable-item");
        dragActive = false;

        const ctrl = e.ctrlKey || e.metaKey;
        if (!ctrl && !dragStartedOnItem) {
            updateSelection(new Set());
            props.setLastClickedIndex(null);
        }
    }

    function handleMouseMove(e: MouseEvent) {
        if (!props.isDragging()) return;

        const start = dragStart();
        const dx = e.clientX - start.x;
        const dy = e.clientY - start.y;

        if (!dragActive && Math.hypot(dx, dy) > DRAG_THRESHOLD) {
            dragActive = true;
            props.setJustDragged(true);
        }
        if (!dragActive) return;

        setDragEnd({ x: e.clientX, y: e.clientY });

        const rect = {
            left: Math.min(start.x, e.clientX),
            right: Math.max(start.x, e.clientX),
            top: Math.min(start.y, e.clientY),
            bottom: Math.max(start.y, e.clientY),
        };

        dragSelection.clear();
        document.querySelectorAll(".selectable-item").forEach(el => {
            const r = el.getBoundingClientRect();
            const intersects =
                r.left < rect.right &&
                r.right > rect.left &&
                r.top < rect.bottom &&
                r.bottom > rect.top;
            if (intersects) {
                const path = el.getAttribute("data-path");
                if (path) dragSelection.add(path);
            }
        });

        const ctrl = e.ctrlKey || e.metaKey;
        if (!dragClearApplied && !ctrl) {
            updateSelection(new Set());
            dragClearApplied = true;
        }

        const base = ctrl ? new Set(props.selectedItems()) : new Set<string>();
        dragSelection.forEach(p => base.add(p));
        updateSelection(base);
    }

    function handleMouseUp(e: MouseEvent) {
        if (!props.isDragging()) return;
        props.setIsDragging(false);

        if (dragActive) {
            props.setJustDragged(true);
            setTimeout(() => props.setJustDragged(false), 40);
        } else {
            handleItemClickUnderMouse(e);
        }

        dragSelection.clear();
        dragClearApplied = false;
        dragStartedOnItem = false;
        dragActive = false;
    }

    function handleItemClickUnderMouse(e: MouseEvent) {
        const el = (e.target as HTMLElement)?.closest(".selectable-item");
        if (!el) return;
        const path = el.getAttribute("data-path");
        if (!path) return;
        const idx = files().findIndex(f => f.path === path);
        if (idx === -1) return;
        handleItemClick(files()[idx], idx, e);
    }

    function startDragOrSelect(file: FileChunk, index: number, e: MouseEvent) {
        if (e.button !== 0) return;
        handleItemClick(file, index, e);
    }

    return (
        <div
            ref={(el) => (_panelEl = el)}
            class="relative flex-1 flex flex-col h-full overflow-hidden">
            <Show when={showProgress()} fallback={<div class="w-full h-1.5 mb-2 bg-gray-200" />}>
                <div class="relative w-full h-1.5 bg-gray-200 overflow-hidden mb-2">
                    <div
                        class="absolute top-0 left-0 h-full bg-gradient-to-r from-blue-400 to-blue-300 transition-all duration-300 ease-out"
                        style={{ width: `${progress() * 100}%`, opacity: progress() >= 1 ? 0 : 1 }}
                    />
                </div>
            </Show>

            <div
                class="flex flex-grow flex-col overflow-hidden"
                onMouseDown={handleMouseDown}
                onMouseMove={handleMouseMove}
                onMouseUp={handleMouseUp}>
                <Show
                    when={isHomePath()}
                    fallback={
                        <StandardLayout
                            files={files}
                            loading={loading}
                            viewMode={props.viewMode}
                            showExtensions={props.showExtensions}
                            iconSize={props.iconSize}
                            handleDoubleClick={handleDoubleClick}
                            getFileIcon={getFileIcon}
                            formatDate={formatDate}
                            selectedItems={props.selectedItems}
                            startDragOrSelect={startDragOrSelect}
                        />
                    }>
                    <HomeLayout
                        files={files}
                        handleDoubleClick={handleDoubleClick}
                        getFileIcon={getFileIcon}
                        formatDate={formatDate}
                        selectedItems={props.selectedItems}
                        startDragOrSelect={startDragOrSelect}
                    />
                </Show>
            </div>

            <div class="p-2 text-xs text-gray-700 flex justify-between">
                <span>
                    {props.selectedItems().size} item
                    {props.selectedItems().size === 1 ? "" : "s"} selected
                    {" / "}
                    {files().length} total
                </span>
                <span>
                    {(() => {
                        let total = 0;
                        for (const f of files()) {
                            if (props.selectedItems().has(f.path)) total += f.size ?? 0;
                        }

                        // format bytes nicely
                        const units = ["B", "KB", "MB", "GB", "TB"];
                        let i = 0;
                        while (total >= 1024 && i < units.length - 1) {
                            total /= 1024;
                            i++;
                        }
                        return `${total.toFixed(2)} ${units[i]}`;
                    })()}
                </span>
            </div>

            <Show when={props.isDragging()}>
                <Portal>
                    <div
                        class="fixed pointer-events-none z-50 border border-blue-400/70 bg-blue-200/30 rounded-sm"
                        style={{
                            left: `${Math.min(dragStart().x, dragEnd().x)}px`,
                            top: `${Math.min(dragStart().y, dragEnd().y)}px`,
                            width: `${Math.abs(dragEnd().x - dragStart().x)}px`,
                            height: `${Math.abs(dragEnd().y - dragStart().y)}px`,
                        }}
                    />
                </Portal>
            </Show>


            <Show when={error()}>
                <Portal>
                    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
                        <div class="bg-white rounded-md p-4 shadow-lg w-80 max-w-full">
                            <h2 class="font-semibold text-lg mb-2 text-red-500">Error</h2>
                            <p class="text-sm text-gray-700 break-words mb-4">{error()}</p>
                            <div class="flex justify-end gap-2">
                                <button class="px-3 py-1.5 bg-gray-200 rounded hover:bg-gray-300 text-sm" onClick={() => setError(null)}>OK</button>
                                <button class="px-3 py-1.5 bg-red-500 text-white rounded hover:bg-red-600 text-sm" onClick={() => setError(null)}>Close</button>
                            </div>
                        </div>
                    </div>
                </Portal>
            </Show>
        </div>
    );
}
