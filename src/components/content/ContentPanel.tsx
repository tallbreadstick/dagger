import { createSignal, createEffect, onCleanup, For, Show } from "solid-js";
import { Portal } from "solid-js/web";
import type { TabEntry } from "../../App";
import { streamDirectoryContents, FileChunk } from "../../scripts/stream";
import { openPath } from "@tauri-apps/plugin-opener";
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
    FaSolidFolder
} from "solid-icons/fa";
import Tab from "../../classes/Tab";
import { LazyImage } from "../LazyImage";

export default function ContentPanel(props: {
    currentTab: TabEntry | null;
    setCurrentTab: (entry: TabEntry) => void;
    sortKey: 'name' | 'size' | 'filetype' | 'date_modified';
    setSortKey: (key: 'name' | 'size' | 'filetype' | 'date_modified') => void;
    ascending: boolean;
    setAscending: (v: boolean) => void;
    viewMode: 'grid' | 'list';
    showHidden: boolean;
    showExtensions: boolean;
}) {
    const [files, setFiles] = createSignal<FileChunk[]>([]);
    const [loading, setLoading] = createSignal(false);
    const [error, setError] = createSignal<string | null>(null);
    const [progress, setProgress] = createSignal(0);
    const [showProgress, setShowProgress] = createSignal(false);
    let startTime: number | null = null;
    let progressTimer: number | null = null;

    let cancelStream: (() => Promise<void>) | null = null;

    const loadDirectory = async (path: string) => {
        setFiles([]);
        setLoading(true);
        setProgress(0);
        setShowProgress(true);
        startTime = performance.now();

        // Kill previous simulated progress timer
        if (progressTimer) cancelAnimationFrame(progressTimer);

        if (cancelStream) {
            await cancelStream();
            cancelStream = null;
        }

        const pendingChunks: FileChunk[] = [];
        let rafScheduled = false;

        function scheduleUpdate() {
            if (!rafScheduled) {
                rafScheduled = true;
                requestAnimationFrame(() => {
                    setFiles(prev => [...prev, ...pendingChunks.splice(0)]);
                    rafScheduled = false;
                });
            }
        }

        // 🕒 Fake smooth progress that slows over time
        function simulateProgress() {
            if (!startTime) return;
            const elapsed = (performance.now() - startTime) / 1000; // seconds
            // Easing: fast start, slow finish
            const eased = 1 - Math.exp(-0.6 * elapsed);
            setProgress(Math.min(eased * 0.98, 0.98)); // cap at 98% until real done
            progressTimer = requestAnimationFrame(simulateProgress);
        }
        progressTimer = requestAnimationFrame(simulateProgress);

        const unlisten = await streamDirectoryContents(
            path,
            (chunk: FileChunk) => {
                if (!props.showHidden && chunk.name.startsWith('.')) return;
                pendingChunks.push(chunk);
                scheduleUpdate();
            },
            () => {
                // ✅ Real completion
                if (progressTimer) cancelAnimationFrame(progressTimer);
                setProgress(1);
                setLoading(false);
                setTimeout(() => setShowProgress(false), 400);
                cancelStream = null;
            },
            { sortKey: props.sortKey, ascending: props.ascending }
        );

        cancelStream = async () => {
            unlisten();
            if (progressTimer) cancelAnimationFrame(progressTimer);
        };
    };

    createEffect(() => {
        const path = props.currentTab?.tab.workingDir;
        if (path) loadDirectory(path);
        else setFiles([]);
    });

    onCleanup(async () => {
        if (cancelStream) await cancelStream();
    });

    function getFileIcon(file: FileChunk) {
        const name = file.name;

        if (file.is_dir) {
            return <FaSolidFolder class={`${props.viewMode === 'grid' ? 'w-12 h-12' : 'w-5 h-5'} text-blue-300 mb-1`} />;
        }

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

        if ((imageExts.includes(ext) || videoExts.includes(ext)) && file.thumbnail) {
            return (
                <LazyImage
                    src={`data:image;base64,${file.thumbnail}`}
                    alt={name}
                    class={`${props.viewMode === 'grid' ? 'w-12 h-12' : 'w-5 h-5'} object-cover rounded mb-1`}
                />
            );
        }

        if (docExts.includes(ext)) return <FaSolidFileWord class={`${props.viewMode === 'grid' ? 'w-12 h-12' : 'w-5 h-5'} text-blue-400 mb-1`} />;
        if (presExts.includes(ext)) return <FaSolidFilePowerpoint class={`${props.viewMode === 'grid' ? 'w-12 h-12' : 'w-5 h-5'} text-orange-400 mb-1`} />;
        if (sheetExts.includes(ext)) return <FaSolidFileExcel class={`${props.viewMode === 'grid' ? 'w-12 h-12' : 'w-5 h-5'} text-green-400 mb-1`} />;
        if (videoExts.includes(ext)) return <FaSolidFileVideo class={`${props.viewMode === 'grid' ? 'w-12 h-12' : 'w-5 h-5'} text-purple-400 mb-1`} />;
        if (audioExts.includes(ext)) return <FaSolidFileAudio class={`${props.viewMode === 'grid' ? 'w-12 h-12' : 'w-5 h-5'} text-indigo-400 mb-1`} />;
        if (imageExts.includes(ext)) return <FaSolidFileImage class={`${props.viewMode === 'grid' ? 'w-12 h-12' : 'w-5 h-5'} text-pink-400 mb-1`} />;
        if (archiveExts.includes(ext)) return <FaSolidBoxArchive class={`${props.viewMode === 'grid' ? 'w-12 h-12' : 'w-5 h-5'} text-yellow-400 mb-1`} />;
        if (execExts.includes(ext)) return <FaSolidFileCode class={`${props.viewMode === 'grid' ? 'w-12 h-12' : 'w-5 h-5'} text-red-400 mb-1`} />;
        if (codeExts.includes(ext)) return <FaSolidFileCode class={`${props.viewMode === 'grid' ? 'w-12 h-12' : 'w-5 h-5'} text-gray-600 mb-1`} />;

        return <FaSolidFile class={`${props.viewMode === 'grid' ? 'w-12 h-12' : 'w-5 h-5'} text-gray-500 mb-1`} />;
    }

    const formatDate = (dateStr?: string) => {
        if (!dateStr) return "";
        const d = new Date(dateStr);
        return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')} ` +
            `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`;
    }

    function updateTab(entry: TabEntry, updater: (tab: Tab) => Tab) {
        if (!entry) return;
        const newTab = updater(entry.tab);
        entry.setTab(newTab);
    }

    function handleNavigate(path: string) {
        const entry = props.currentTab;
        if (!entry) return;

        updateTab(entry, (tab) => {
            const newTab = tab.clone();
            newTab.navigateTo(path);
            return newTab;
        });
    }

    const handleDoubleClick = (file: FileChunk) => {
        if (!props.currentTab) return;
        if (file.is_dir) {
            handleNavigate(file.path);
        } else {
            openPath(file.path).catch((err) => {
                setError(err);
            });
        }
    };

    return (
        <div class="flex-1 flex flex-col h-full overflow-hidden">
            <Show when={showProgress()} fallback={<div class="w-full h-1.5 mb-2 bg-gray-200" />}>
                <div class="relative w-full h-1.5 bg-gray-200 overflow-hidden mb-2">
                    <div
                        class="absolute top-0 left-0 h-full bg-gradient-to-r from-blue-400 to-blue-300 shadow-[0_0_10px_rgba(96,165,250,0.7)] transition-all duration-300 ease-out"
                        style={{
                            width: `${progress() * 100}%`,
                            opacity: progress() >= 1 ? 0 : 1,
                        }}
                    />
                </div>
            </Show>
            <div class="flex flex-col h-full w-full p-2 overflow-auto scrollbar-thin scrollbar-thumb-gray-400/60 custom-scrollbar">    

                <Show when={!loading()}>
                    <div
                        class={`${props.viewMode === 'grid'
                            ? 'grid gap-3 justify-items-center'
                            : 'flex flex-col gap-1'}`}
                        style={props.viewMode === 'grid'
                            ? 'grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));'
                            : undefined}
                    >
                        <For each={files()}>
                            {(file) => (
                                <div
                                    onDblClick={() => handleDoubleClick(file)}
                                    class={`flex ${props.viewMode === 'grid' ? 'flex-col items-center p-2 bg-white/80' : 'flex-row items-center p-1 bg-white/40'} rounded shadow hover:bg-blue-50 cursor-pointer w-full`}
                                    title={file.name}
                                >
                                    {getFileIcon(file)}

                                    {props.viewMode === 'grid' ? (
                                        <div class="text-center mt-1 w-full">
                                            <div class="truncate text-xs">{file.name}</div>
                                        </div>
                                    ) : (
                                        <div class="flex flex-1 text-xs text-gray-700 min-w-0 ml-2">
                                            {/* Name column */}
                                            <div class="flex-1 truncate">{file.name}</div>

                                            {/* Type column */}
                                            <div class="w-28 text-right ml-4">{file.is_dir ? 'Folder' : file.name.split('.').pop()?.toUpperCase() ?? ''}</div>

                                            {/* Size column */}
                                            <div class="w-24 text-right ml-6">
                                                {!file.is_dir && file.size != null
                                                    ? `${(file.size / 1024).toLocaleString(undefined, { minimumFractionDigits: 1, maximumFractionDigits: 1 })} KB`
                                                    : '-'}
                                            </div>

                                            {/* Date modified column */}
                                            <div class="w-40 text-right ml-6">{formatDate(file.date_modified)}</div>
                                        </div>
                                    )}
                                </div>
                            )}
                        </For>
                    </div>
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
        </div>
    );
}