// src/tauri/stream.ts
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export type FileChunk = {
    name: string;
    path: string;
    is_dir: boolean;
    size?: number;
    filetype?: string;
    thumbnail?: string | null;
    date_modified?: string | undefined;
};

export type StreamOptions = {
    sortKey?: 'name' | 'size' | 'filetype' | 'date_modified';
    ascending?: boolean;
    showHidden?: boolean;
};

let currentStreamId = 0;

/**
 * Stream a directory's contents from the Tauri backend.
 * @param path Directory path to stream
 * @param onMetadata Callback fired per file/folder metadata
 * @param onMetadataComplete Callback fired once all metadata has been emitted
 * @param onThumbnail Callback fired when a file's thumbnail is available
 * @param onComplete Callback fired once the directory has finished streaming
 * @param options Optional sorting configuration
 * @returns A function to unsubscribe/cancel the stream
 */
export async function streamDirectoryContents(
    path: string,
    onMetadata: (chunk: FileChunk) => void,
    onMetadataComplete: () => void,
    onThumbnail: (path: string, thumbnail: string | null) => void,
    onComplete: () => void,
    options: StreamOptions = {}
) {
    const { sortKey = 'name', ascending = true, showHidden = false } = options;
    const requestId = ++currentStreamId;

    // Phase 1: Metadata
    const unlistenMetadata = await listen('file-metadata', (event) => {
        const payload = event.payload as any;
        if (!payload || payload.request_id !== requestId) return;
        onMetadata(payload);
    });

    // Metadata complete
    const unlistenMetadataComplete = await listen('file-metadata-complete', (event) => {
        const payload = event.payload as any;
        if (!payload || payload.request_id !== requestId) return;
        onMetadataComplete?.();
    });

    // Phase 2: Thumbnails
    const unlistenThumbnail = await listen('file-thumbnail', (event) => {
        const payload = event.payload as any;
        if (!payload || payload.request_id !== requestId) return;
        onThumbnail(payload.path, payload.thumbnail);
    });

    // Phase 3: Complete
    const unlistenComplete = await listen('file-stream-complete', (event) => {
        const payload = event.payload as any;
        if (!payload || payload.request_id !== requestId) return;
        if (payload.path === path) onComplete?.();
    });

    await invoke('stream_directory_contents', {
        path,
        sortKey,
        ascending,
        showHidden,
        requestId,
    });

    return async () => {
        unlistenMetadata();
        unlistenMetadataComplete();
        unlistenThumbnail();
        unlistenComplete();
    };
}
