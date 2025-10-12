import { Accessor, Component, For, JSX, Show } from "solid-js";
import { FileChunk } from "../../scripts/stream";

type StandardLayoutProps = {
    files: Accessor<FileChunk[]>;
    loading: Accessor<boolean>;
    viewMode: Accessor<'grid' | 'list'>;
    iconSize: Accessor<'small' | 'medium'>;
    handleDoubleClick: (file: FileChunk) => void;
    getFileIcon: (file: FileChunk) => JSX.Element;
    formatDate: (date: number | undefined) => string;
    selectedItems: Accessor<Set<string>>;
    startDragOrSelect: (file: FileChunk, index: number, e: MouseEvent) => void;
}

const StandardLayout: Component<StandardLayoutProps> = ({
    files,
    loading,
    viewMode,
    iconSize,
    handleDoubleClick,
    getFileIcon,
    formatDate,
    selectedItems,
    startDragOrSelect,
}) => {

    function itemWidth() {
        switch (iconSize()) {
            case 'small':
                return '90px';
            case 'medium':
                return '120px';
        }
    }
    
    return (
        <div class="flex flex-col h-full w-full p-2 overflow-auto scrollbar-thin scrollbar-thumb-gray-400/60 custom-scrollbar">
            <Show when={!loading()}>
                <div
                    class={`${viewMode() === 'grid' ? 'grid gap-3 justify-items-center' : 'flex flex-col gap-1'}`}
                    style={viewMode() === 'grid' ? `grid-template-columns: repeat(auto-fill, minmax(${itemWidth()}, 1fr));` : undefined}
                >
                    <For each={files()}>
                        {(file, index) => (
                            <div
                                onDblClick={() => handleDoubleClick(file)}
                                onClick={(e) => startDragOrSelect(file, index(), e)}
                                class="flex rounded shadow cursor-pointer w-full selectable-item"
                                classList={{
                                    'flex-col items-center p-2': viewMode() === 'grid',
                                    'flex-row items-center p-1': viewMode() === 'list',
                                    'bg-white/80': !selectedItems().has(file.path),
                                    'bg-white/40': viewMode() === 'list' && !selectedItems().has(file.path),
                                    'bg-blue-200': selectedItems().has(file.path),
                                }}
                                data-path={file.path}
                                title={file.name}
                            >
                                {getFileIcon(file)}
                                {viewMode() === 'grid' ? (
                                    <div class="text-center mt-1 w-full">
                                        <div class="truncate text-xs">{file.name}</div>
                                    </div>
                                ) : (
                                    <div class="flex flex-1 text-xs text-gray-700 min-w-0 ml-2">
                                        <div class="flex-1 truncate">{file.name}</div>
                                        <div class="w-28 text-right ml-4">{file.is_dir ? 'Folder' : file.name.split('.').pop()?.toUpperCase() ?? ''}</div>
                                        <div class="w-24 text-right ml-6">
                                            {!file.is_dir && file.size != null
                                                ? `${(file.size / 1024).toLocaleString(undefined, { minimumFractionDigits: 1, maximumFractionDigits: 1 })} KB`
                                                : '-'}
                                        </div>
                                        <div class="w-40 text-right ml-6">{formatDate(file.date_modified)}</div>
                                    </div>
                                )}
                            </div>

                        )}
                    </For>
                </div>
            </Show>
        </div>
    );
}

export default StandardLayout;