import { createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { FaSolidFolder, FaSolidMagnifyingGlass, FaSolidMicrophone, FaSolidPen } from "solid-icons/fa";

export default function SearchBar() {
    const [mode, setMode] = createSignal("text"); // "text" | "image" | "audio" | "document"
    const [selectedFile, setSelectedFile] = createSignal<string | null>(null);

    const handleImageSelect = async () => {
        try {
            const path = await invoke<string>("upload_image_file");
            setSelectedFile(path);
            console.log("Selected image:", path);
        } catch (err) {
            console.warn("Image selection canceled or failed:", err);
        }
    };

    const handleAudioSelect = async () => {
        try {
            const path = await invoke<string>("upload_audio_file");
            setSelectedFile(path);
            console.log("Selected audio:", path);
        } catch (err) {
            console.warn("Audio selection canceled or failed:", err);
        }
    };

    const handleDocumentSelect = async () => {
        try {
            const path = await invoke<string>("upload_document_file");
            setSelectedFile(path);
            console.log("Selected document:", path);
        } catch (err) {
            console.warn("Document selection canceled or failed:", err);
        }
    };

    return (
        <div class="flex flex-row items-center w-full gap-2">
            {/* Mode selector */}
            <select
                value={mode()}
                onInput={(e) => setMode(e.currentTarget.value)}
                class="text-sm border border-gray-300 rounded px-1 py-0.5 outline-none bg-gray-200"
            >
                <option value="text">Text Occurrence</option>
                <option value="image">Vector: Image</option>
                <option value="audio">Vector: Audio</option>
                <option value="document">Vector: Document</option>
            </select>

            {/* Adaptive input */}
            {mode() === "text" && (
                <input class="w-full text-sm outline-none border border-gray-300 rounded px-2 py-1" placeholder="Search text in files..." />
            )}

            {mode() === "image" && (
                <div class="flex flex-row items-center w-full gap-2">
                    <input
                        class="w-full text-sm outline-none border border-gray-300 rounded px-2 py-1"
                        placeholder="Describe an image..."
                    />
                    <button title="Draw a sketch" class="p-1 hover:bg-gray-200 rounded">
                        <FaSolidPen class="w-4 h-4" />
                    </button>
                    <button
                        title="Upload image"
                        onClick={handleImageSelect}
                        class="p-1 hover:bg-gray-200 rounded"
                    >
                        <FaSolidFolder class="w-4 h-4 text-black" />
                    </button>
                </div>
            )}

            {mode() === "audio" && (
                <div class="flex flex-row items-center w-full gap-2">
                    <input
                        class="w-full text-sm outline-none border border-gray-300 rounded px-2 py-1"
                        placeholder="Describe audio..."
                    />
                    <button title="Record with mic" class="p-1 hover:bg-gray-200 rounded">
                        <FaSolidMicrophone class="w-4 h-4" />
                    </button>
                    <button
                        title="Upload audio"
                        onClick={handleAudioSelect}
                        class="p-1 hover:bg-gray-200 rounded"
                    >
                        <FaSolidFolder class="w-4 h-4 text-black" />
                    </button>
                </div>
            )}

            {mode() === "document" && (
                <div class="flex flex-row items-center w-full gap-2">
                    <textarea
                        class="w-full text-sm outline-none border border-gray-300 rounded px-2 py-1 resize-none"
                        rows={1}
                        placeholder="Describe a document..."
                    />
                    <button
                        title="Upload document"
                        onClick={handleDocumentSelect}
                        class="p-1 hover:bg-gray-200 rounded"
                    >
                        <FaSolidFolder class="w-4 h-4 text-black" />
                    </button>
                </div>
            )}

            {/* Search button */}
            <button class="p-2 hover:bg-gray-200 rounded">
                <FaSolidMagnifyingGlass class="w-4 h-4 text-gray-600" />
            </button>
        </div>
    );
}
