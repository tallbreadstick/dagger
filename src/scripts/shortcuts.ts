// src/scripts/shortcuts.ts
import { onCleanup } from "solid-js";

export type ShortcutActions = {
    toggleSearchMode: () => void;
    setSearchMode: (mode: "text" | "image" | "audio" | "document") => void;
    openSelectedItem: () => void;
};

// Global keyboard listener
export function useGlobalShortcuts(actions: ShortcutActions) {
    const handleKeyDown = (e: KeyboardEvent) => {
        if (e.ctrlKey) {
            switch (e.key.toLowerCase()) {
                case "f":
                    e.preventDefault();
                    actions.toggleSearchMode();
                    break;
                case "1":
                    e.preventDefault();
                    actions.setSearchMode("text");
                    break;
                case "2":
                    e.preventDefault();
                    actions.setSearchMode("image");
                    break;
                case "3":
                    e.preventDefault();
                    actions.setSearchMode("audio");
                    break;
                case "4":
                    e.preventDefault();
                    actions.setSearchMode("document");
                    break;
            }
        } else {
            switch (e.key.toLocaleLowerCase()) {
                case "enter":
                    e.preventDefault();
                    actions.openSelectedItem();
                    break;
            }
        }
    };

    window.addEventListener("keydown", handleKeyDown);
    onCleanup(() => window.removeEventListener("keydown", handleKeyDown));
}
