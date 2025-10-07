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
};

let currentStreamId = 0;

/**
 * Stream a directory's contents from the Tauri backend.
 * @param path Directory path to stream
 * @param onChunk Callback fired per file/folder chunk
 * @param onComplete Callback fired once the directory has finished streaming
 * @param options Optional sorting configuration
 * @returns A function to unsubscribe/cancel the stream
 */

export async function streamDirectoryContents(
    path: string,
    onChunk: (chunk: FileChunk) => void,
    onComplete: () => void,
    options: StreamOptions
) {
    const { sortKey = 'name', ascending = true } = options || {};
    const requestId = ++currentStreamId;

    const unlistenChunk = await listen('file-chunk', (event) => {
        const payload = event.payload as any;
        if (!payload) return;
        if (payload.request_id !== requestId) return; // IGNORE stale chunks
        onChunk(payload);
    });

    const unlistenComplete = await listen('file-chunk-complete', (event) => {
        const payload = event.payload as any;
        if (payload?.request_id !== requestId) return;
        if (payload?.path === path) onComplete?.();
    });

    await invoke('stream_directory_contents', {
        path, sortKey, ascending, requestId
    });

    return async () => {
        unlistenChunk();
        unlistenComplete();
    };
}