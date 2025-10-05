import { listen } from "@tauri-apps/api/event";
import { onMount } from "solid-js";

export function useWindowFocusEvents() {
    onMount(async () => {
        await listen("window-blur", () => {
            document.body.classList.add("unfocused");
        });
        await listen("window-focus", () => {
            document.body.classList.remove("unfocused");
        });
    });
}
