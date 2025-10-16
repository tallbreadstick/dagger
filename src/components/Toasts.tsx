import { toast } from "solid-toast";
import { createSignal, onCleanup, onMount } from "solid-js";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

export function showCopyToast(count: number) {
    toast.custom((t) => (
        <div
            class={`w-80 px-3 py-2 border backdrop-blur-md shadow-md text-gray-800
                     font-[Outfit] text-sm select-none transition-all duration-500
                     bg-white/80 border-gray-300`}
        >
            {/* Header */}
            <div class="flex justify-between items-center mb-1">
                <span class="font-medium">Copy Complete</span>
                <button
                    class="opacity-50 hover:opacity-100 transition"
                    onClick={() => toast.dismiss(t.id)}
                >
                    ✕
                </button>
            </div>

            {/* Footer label (same alignment + style as paste complete message) */}
            <div class="text-[11px] text-gray-600 text-right">
                Copied {count} item{count !== 1 ? "s" : ""} to clipboard.
            </div>
        </div>
    ), {
        position: "bottom-right",
        duration: 2500,
    });
}

export function showPasteProgressToast(workingDir: string) {
    toast.custom((t) => {
        const [phase, setPhase] = createSignal<"scan" | "paste" | "complete" | "error">("scan");
        const [progress, setProgress] = createSignal(0);
        const [label, setLabel] = createSignal("Preparing...");
        const [copiedCount, setCopiedCount] = createSignal(0);

        let totalSize = 0;
        let transferred = 0;
        let unlistenFns: (() => void)[] = [];

        const formatBytes = (b: number) => {
            const sizes = ["B", "KB", "MB", "GB"];
            if (b === 0) return "0 B";
            const i = Math.floor(Math.log(b) / Math.log(1024));
            return `${(b / Math.pow(1024, i)).toFixed(1)} ${sizes[i]}`;
        };

        const updateProgress = () => {
            setProgress(totalSize > 0 ? (transferred / totalSize) * 100 : 100);
            setLabel(`${formatBytes(transferred)} / ${formatBytes(totalSize)}`);
        };

        onMount(async () => {
            const requestId = Date.now();
            const add = async (name: string, fn: (p: any) => void) => {
                const u = await listen(name, (e) => fn(e.payload));
                unlistenFns.push(u);
            };

            await add("clipboard-paste-scan", (p) => {
                if (p.request_id !== requestId) return;
                totalSize = p.total_size;
                transferred = 0;
                setPhase("paste");
                setLabel(`Scanning ${p.file_count ?? "files"}...`);
                updateProgress();
            });

            await add("clipboard-paste-file", (p) => {
                if (p.request_id !== requestId) return;
                transferred += p.size;
                updateProgress();
            });

            await add("clipboard-paste-complete", (p) => {
                if (p.request_id !== requestId) return;
                setCopiedCount(p.files_copied);
                setProgress(100);
                setLabel(`Copied ${p.files_copied} files`);
                setPhase("complete");

                // ⏳ wait before auto-dismiss
                setTimeout(() => toast.dismiss(t.id), 2000);
            });

            await add("clipboard-paste-file-error", (p) => {
                if (p.request_id !== requestId) return;
                console.warn("Error:", p);
                setPhase("error");
                setLabel("Failed to copy some files");
                setTimeout(() => toast.dismiss(t.id), 2000);
            });

            try {
                await invoke("paste_items_from_clipboard", { workingDir, requestId });
            } catch (err) {
                console.error(err);
                setPhase("error");
                setLabel("Paste failed");
                setTimeout(() => toast.dismiss(t.id), 2000);
            }
        });

        onCleanup(() => unlistenFns.forEach((u) => u()));

        return (
            <div
                class={`w-80 px-3 py-2 border backdrop-blur-md shadow-md text-gray-800 
                       font-[Outfit] text-sm select-none transition-all duration-500
                       ${phase() === "error"
                        ? "bg-red-50/90 border-red-200"
                        : "bg-white/80 border-gray-300"
                    }`}
            >
                <div class="flex justify-between items-center mb-1">
                    <span class="font-medium">
                        {{
                            scan: "Preparing files...",
                            paste: "Pasting files...",
                            complete: "Paste Complete",
                            error: "Paste Failed",
                        }[phase()]}
                    </span>
                    <button
                        class="opacity-50 hover:opacity-100 transition"
                        onClick={() => toast.dismiss(t.id)}
                    >
                        ✕
                    </button>
                </div>

                {/* Progress bar – hide when complete */}
                {phase() !== "complete" && (
                    <div class="h-1.5 bg-gray-200 rounded overflow-hidden mb-1">
                        <div
                            class={`h-full transition-all duration-300 ease-out
                                   ${phase() === "error"
                                    ? "bg-red-400"
                                    : "bg-blue-500"
                                }`}
                            style={{ width: `${progress()}%` }}
                        />
                    </div>
                )}

                {/* Label */}
                <div class="text-[11px] text-gray-600 text-right">
                    {phase() === "complete"
                        ? `Pasted ${copiedCount()} files from clipboard.`
                        : label()}
                </div>
            </div>
        );
    }, {
        position: "bottom-right",
        duration: 10000, // arbitrary; gets auto-dismissed manually anyway
    });
}
