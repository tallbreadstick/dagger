/**
 * NewFileMenu.tsx
 * ------------------
 * Dropdown for choosing file type when creating new files.
 */

import { onCleanup, onMount } from "solid-js";

export default function NewFileMenu(props: { onClose: () => void, x: number, y: number }) {
    let menuRef: HTMLDivElement | undefined;

    onMount(() => {
        const handleEsc = (e: KeyboardEvent) => e.key === "Escape" && props.onClose();
        window.addEventListener("keydown", handleEsc);
        onCleanup(() => window.removeEventListener("keydown", handleEsc));
    });

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
            <MenuItem label="Text File (.txt)" />
            <MenuItem label="Markdown File (.md)" />
            <MenuItem label="JSON File (.json)" />
            <MenuItem label="JavaScript File (.js)" />
        </div>
    );
}

function MenuItem(props: { label: string }) {
    return (
        <button class="text-left px-3 py-1.5 text-sm hover:bg-gray-100">
            {props.label}
        </button>
    );
}
