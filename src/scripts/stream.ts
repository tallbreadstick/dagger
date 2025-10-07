// src/tauri/stream.ts
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

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
    onComplete?: () => void,
    options?: StreamOptions
): Promise<UnlistenFn> {
    const { sortKey = 'name', ascending = true } = options || {};

    // Listen to individual file chunks
    const unlistenChunk = await listen<FileChunk>('file-chunk', (event) => {
        if (event.payload) {
            onChunk(event.payload);
        }
    });

    // Listen to stream completion
    const unlistenComplete = await listen<{ path: string }>('file-chunk-complete', (event) => {
        if (event.payload?.path === path) {
            onComplete?.();
        }
    });

    // Invoke the backend stream command
    await invoke('stream_directory_contents', {
        path,
        sortKey,
        ascending,
    });

    // Return a single function to cancel both listeners
    return async () => {
        await unlistenChunk();
        await unlistenComplete();
    };
}
