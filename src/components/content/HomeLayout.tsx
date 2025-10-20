import { Accessor, Component, createMemo, For, JSX, Show } from "solid-js";
import { FileChunk } from "../../scripts/stream";

type HomeLayoutProps = {
    files: Accessor<FileChunk[]>;
    handleDoubleClick: (file: FileChunk) => void;
    getFileIcon: (file: FileChunk) => JSX.Element;
    formatDate: (date: number | undefined) => string;
    selectedItems: Accessor<Set<string>>;
    startDragOrSelect: (file: FileChunk, index: number, e: MouseEvent) => void;
};

const HomeLayout: Component<HomeLayoutProps> = ({
    files,
    handleDoubleClick,
    getFileIcon,
    formatDate,
    selectedItems,
    startDragOrSelect,
}) => {
    const dirs = createMemo(() => files().filter(f => !f.pinned && f.is_dir));
    const pinned = createMemo(() => files().filter(f => f.pinned));
    const recents = createMemo(() => files().filter(f => !f.pinned && !f.is_dir));

    return (
        <div class="flex flex-col h-full w-full p-3 overflow-auto gap-4 custom-scrollbar">
            {/* TOP: Recent Dirs Grid */}
            <Show when={dirs().length}>
                <div>
                    <h2 class="font-semibold text-sm text-gray-700 mb-2">Recent Directories</h2>
                    <div
                        class="grid gap-3 justify-items-center"
                        style={`grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));`} // mimic StandardLayout
                    >
                        <For each={dirs()}>
                            {(f, index) => (
                                <div
                                    onClick={(e) => startDragOrSelect(f, index(), e)}
                                    onDblClick={() => handleDoubleClick(f)}
                                    class="flex flex-col items-center p-2 rounded shadow cursor-pointer selectable-item transition w-full"
                                    classList={{
                                        "bg-white/80": !selectedItems().has(f.path),
                                        "bg-blue-200": selectedItems().has(f.path),
                                    }}
                                    data-path={f.path}
                                    title={f.name}
                                >
                                    {getFileIcon(f)}
                                    <div class="truncate text-xs mt-1 w-full text-center">{f.name}</div>
                                </div>
                            )}
                        </For>
                    </div>
                </div>
            </Show>

            {/* MIDDLE: Pinned Items */}
            <Show when={pinned().length}>
                <div>
                    <h2 class="font-semibold text-sm text-gray-700 mb-2">Pinned</h2>
                    <div class="flex flex-col gap-1">
                        <For each={pinned()}>
                            {(f, index) => (
                                <div
                                    onClick={(e) => startDragOrSelect(f, index(), e)}
                                    onDblClick={() => handleDoubleClick(f)}
                                    class="flex flex-row items-center p-2 rounded shadow cursor-pointer selectable-item transition"
                                    classList={{
                                        "bg-white/70": !selectedItems().has(f.path),
                                        "bg-blue-200": selectedItems().has(f.path),
                                    }}
                                    data-path={f.path}
                                    title={f.name}
                                >
                                    {getFileIcon(f)}
                                    <div class="flex-1 ml-2 text-xs truncate">{f.name}</div>
                                </div>
                            )}
                        </For>
                    </div>
                </div>
            </Show>

            {/* BOTTOM: Recent Files */}
            <Show when={recents().length}>
                <div>
                    <h2 class="font-semibold text-sm text-gray-700 mb-2">Recent Files</h2>
                    <div class="flex flex-col gap-1">
                        <For each={recents()}>
                            {(f, index) => (
                                <div
                                    onClick={(e) => startDragOrSelect(f, index(), e)}
                                    onDblClick={() => handleDoubleClick(f)}
                                    class="flex flex-row items-center p-2 rounded shadow cursor-pointer selectable-item transition"
                                    classList={{
                                        "bg-white/60": !selectedItems().has(f.path),
                                        "bg-blue-200": selectedItems().has(f.path),
                                    }}
                                    data-path={f.path}
                                    title={f.name}
                                >
                                    {getFileIcon(f)}
                                    <div class="flex-1 ml-2 text-xs truncate">{f.name}</div>
                                    <div class="text-gray-500 text-xs">{formatDate(f.date_modified)}</div>
                                </div>
                            )}
                        </For>
                    </div>
                </div>
            </Show>
        </div>
    );
};

export default HomeLayout;
