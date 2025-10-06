import { createSignal, onCleanup, onMount } from "solid-js";
import { getCurrentWindow } from "@tauri-apps/api/window";
import DaggerIcon from "../../assets/dagger.png";

const appWindow = getCurrentWindow();

export default function TitleBar() {
    const [maximized, setMaximized] = createSignal(false);

    // Update state whenever window is maximized/unmaximized
    async function updateState() {
        setMaximized(await appWindow.isMaximized());
    }

    onMount(async () => {
        await updateState();
        const unlistenMax = await appWindow.onResized(updateState);
        onCleanup(() => {
            unlistenMax();
        });
    });

    return (
        <div
            data-tauri-drag-region
            class="w-full h-8 flex items-center justify-between bg-gray-950/70 text-white select-none"
        >
            {/* Left: Logo + Title */}
            <div class="flex items-center gap-2 pl-2">
                {/* Replace with your own SVG/logo */}
                <img data-tauri-drag-region src={DaggerIcon} class="w-5 h-5" />
                <span data-tauri-drag-region class="text-sm font-medium">Dagger File Explorer</span>
            </div>

            {/* Right: Window Controls */}
            <div class="flex items-center h-full">
                <button
                    onClick={() => appWindow.minimize()}
                    class="w-10 h-full flex items-center justify-center hover:bg-gray-700"
                    title="Minimize"
                >
                    &#8211;
                </button>
                <button
                    onClick={() => (maximized() ? appWindow.unmaximize() : appWindow.maximize())}
                    class="w-10 h-full flex items-center justify-center hover:bg-gray-700"
                    title={maximized() ? "Restore Down" : "Maximize"}
                >
                    {maximized() ? "🗗" : "🗖"}
                </button>
                <button
                    onClick={() => appWindow.close()}
                    class="w-10 h-full flex items-center justify-center hover:bg-red-600"
                    title="Close"
                >
                    ✕
                </button>
            </div>
        </div>
    );
}
