import { createSignal, createEffect, Show, For } from "solid-js";
import { getDirectoryTreeFromRoot, FileNode } from "../../scripts/navigation";
import type { TabEntry } from "../../App";
import { TreeNode } from "./TreeNode";
import Tab from "../../classes/Tab";
import { resolveUserHome } from "../../scripts/navigation";
import {
  FaSolidFolder,
  FaSolidDownload,
  FaSolidDesktop,
  FaSolidPhotoFilm,
  FaSolidMusic,
  FaSolidVideo
} from "solid-icons/fa";

export default function Sidebar(props: {
  currentTab?: () => TabEntry | null;
  setCurrentTab?: (entry: TabEntry) => void;
}) {
  const [rootNode, setRootNode] = createSignal<FileNode | null>(null);
  const [quickAccess, setQuickAccess] = createSignal<
    { name: string; path: string; icon: any }[]
  >([]);

  // Resolve user home and setup Quick Access with icons
  createEffect(async () => {
    try {
      const home = await resolveUserHome();
      setQuickAccess([
        { name: "Documents", path: `${home}\\Documents`, icon: FaSolidFolder },
        { name: "Downloads", path: `${home}\\Downloads`, icon: FaSolidDownload },
        { name: "Desktop", path: `${home}\\Desktop`, icon: FaSolidDesktop },
        { name: "Pictures", path: `${home}\\Pictures`, icon: FaSolidPhotoFilm },
        { name: "Music", path: `${home}\\Music`, icon: FaSolidMusic },
        { name: "Videos", path: `${home}\\Videos`, icon: FaSolidVideo },
      ]);
    } catch (err) {
      console.error("Failed to set Quick Access paths:", err);
    }
  });

  createEffect(() => {
    const current = props.currentTab?.();
    if (current) {
      const path = current.tab.workingDir;
      getDirectoryTreeFromRoot(path)
        .then((tree) => setRootNode(tree as FileNode))
        .catch((err) => console.error("Failed to load directory tree:", err));
    }
  });

  function updateTab(entry: TabEntry, updater: (tab: Tab) => Tab) {
    if (!entry) return;
    const newTab = updater(entry.tab);
    entry.setTab(newTab);
  }

  function handleNavigate(path: string) {
    const entry = props.currentTab?.();
    if (!entry) return;

    updateTab(entry, (tab) => {
      const newTab = tab.clone();
      newTab.navigateTo(path);
      return newTab;
    });
  }

  const workingDir = () => props.currentTab?.()?.tab.workingDir ?? "";

  return (
    <div class="h-full flex flex-col border-r font-['Outfit'] font-light border-gray-400/30 overflow-hidden bg-transparent backdrop-blur-sm">
      <div class="px-2 py-2 text-xs uppercase font-semibold text-gray-400 tracking-wider">File Tree</div>

      <div class="flex-1 overflow-y-auto px-1 pb-2 custom-scrollbar" style={{ "min-height": "0" }}>
        <Show when={rootNode()} fallback={<div class="text-xs text-black pl-3">Loading...</div>}>
          <TreeNode
            node={rootNode()!}
            depth={0}
            onNavigate={handleNavigate}
            workingDir={workingDir()}
          />
        </Show>
      </div>

      <div class="px-2 py-2 text-xs uppercase font-semibold text-gray-400 tracking-wider border-t border-gray-400/30">
        Quick Access
      </div>
      <div class="flex flex-col px-2 pb-2">
        <For each={quickAccess()}>
          {(item) => (
            <div
              class="cursor-pointer hover:bg-white/10 active:bg-white/20 px-2 py-1 rounded-md transition-colors flex items-center gap-2 text-sm text-black truncate"
              onClick={() => handleNavigate(item.path)}
            >
              <item.icon class="w-3 h-3 text-gray-400" />
              {item.name}
            </div>
          )}
        </For>
      </div>
    </div>
  );
}
