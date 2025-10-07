/**
 * NewFileMenu.tsx
 * ------------------
 * Dropdown for choosing file type when creating new files.
 * Preserves old behavior, adds new types with colored icons.
 */

import { onCleanup, onMount } from "solid-js";
import {
    FaSolidFolder,
    FaSolidArrowUpRightFromSquare,
    FaSolidFileWord,
    FaSolidFilePowerpoint,
    FaSolidFileExcel,
    FaSolidFileLines,
    FaSolidTerminal,
} from "solid-icons/fa";

export default function NewFileMenu(props: { onClose: () => void; x: number; y: number }) {
    let menuRef: HTMLDivElement | undefined;

    onMount(() => {
        const handleEsc = (e: KeyboardEvent) => e.key === "Escape" && props.onClose();
        window.addEventListener("keydown", handleEsc);
        onCleanup(() => window.removeEventListener("keydown", handleEsc));
    });

    const items = [
        { label: "Folder", icon: FaSolidFolder, color: "#facc15" },
        { label: "Shortcut", icon: FaSolidArrowUpRightFromSquare, color: "#3b82f6" },
        { label: "Document", icon: FaSolidFileWord, color: "#2563eb" },
        { label: "Presentation", icon: FaSolidFilePowerpoint, color: "#dc2626" },
        { label: "Spreadsheet", icon: FaSolidFileExcel, color: "#16a34a" },
        { label: "Plain Text", icon: FaSolidFileLines, color: "#6b7280" },
        { label: "Console Script", icon: FaSolidTerminal, color: "#f97316" }
    ];

    return (
        <div
            ref={menuRef}
            class="actionbar-menu w-48 rounded-md shadow-md bg-white border border-gray-300 flex flex-col z-50"
            style={{
                position: "absolute",
                top: `${props.y + 4}px`,
                left: `${props.x}px`,
            }}
        >
            {items.map((item) => (
                <MenuItem label={item.label} icon={item.icon} color={item.color} />
            ))}
        </div>
    );
}

function MenuItem(props: { label: string; icon: any; color: string }) {
    return (
        <button class="text-left px-3 py-1.5 text-sm hover:bg-gray-100 flex items-center gap-2">
            {props.icon && <props.icon style={{ color: props.color }} class="w-4 h-4 shrink-0" />}
            <span>{props.label}</span>
        </button>
    );
}
