import { createSignal, onCleanup, onMount, Show } from "solid-js";
import { Portal } from "solid-js/web";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

interface ConflictPayload {
    request_id: number;
    src: string;
    dest: string;
    name: string;
}

export default function ConflictPrompt() {
    const [conflict, setConflict] = createSignal<ConflictPayload | null>(null);
    const [applyToAll, setApplyToAll] = createSignal(false);

    onMount(async () => {
        const unlisten = await listen<ConflictPayload>(
            "clipboard-paste-conflict",
            (event) => {
                setConflict(event.payload);
            }
        );
        onCleanup(unlisten);
    });

    const resolveConflict = async (strategy: "Ignore" | "Replace" | "Index") => {
        const c = conflict();
        if (!c) return;
        try {
            await invoke("resolve_copy_conflict", {
                payload: {
                    request_id: c.request_id,
                    strategy,
                    repeat_for_all: applyToAll(),
                },
            });
        } catch (err) {
            console.error("resolve_copy_conflict failed:", err);
        }
        setConflict(null);
    };

    function existing() {
        const c = conflict();
        if (!c) return "";
        const parts = c.dest.split(/[/\\]/); // handles Windows + Unix
        return parts[parts.length - 1];
    }

    function incoming() {
        const c = conflict();
        if (!c) return "";
        const parts = c.src.split(/[/\\]/);
        return parts[parts.length - 1];
    }


    return (
        <Show when={conflict()}>
            <Portal>
                <div class="fixed inset-0 z-[9999] flex items-center justify-center bg-black/40 backdrop-blur-sm">
                    <div class="bg-white rounded-lg shadow-lg p-6 w-96 font-[Outfit] text-gray-800 border border-gray-200">
                        <h2 class="text-lg font-semibold mb-2">File Conflict</h2>
                        <p class="text-sm text-gray-600 mb-4">
                            A file named <strong>{incoming()}</strong> already exists in this location.
                            <br />
                            Existing file: <span class="text-gray-500">{existing()}</span>
                        </p>

                        <label class="flex items-center gap-2 mb-4 text-sm text-gray-600">
                            <input
                                type="checkbox"
                                class="rounded border-gray-300"
                                checked={applyToAll()}
                                onChange={(e) => setApplyToAll((e.target as HTMLInputElement).checked)}
                            />
                            Apply to all conflicts
                        </label>

                        <div class="flex justify-end gap-2">
                            <button
                                class="px-3 py-1.5 bg-gray-200 hover:bg-gray-300 rounded text-sm"
                                onClick={() => resolveConflict("Ignore")}
                            >
                                Ignore
                            </button>
                            <button
                                class="px-3 py-1.5 bg-blue-500 hover:bg-blue-600 text-white rounded text-sm"
                                onClick={() => resolveConflict("Index")}
                            >
                                Rename
                            </button>
                            <button
                                class="px-3 py-1.5 bg-red-500 hover:bg-red-600 text-white rounded text-sm"
                                onClick={() => resolveConflict("Replace")}
                            >
                                Replace
                            </button>
                        </div>
                    </div>
                </div>
            </Portal>
        </Show>
    );
}
