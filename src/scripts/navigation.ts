import { invoke } from "@tauri-apps/api/core";

export interface FileItem {
  name: string;
  path: string;
  is_dir: boolean;
  size?: number | null;
}

export interface FileNode {
  name: string;
  path: string;
  is_dir: boolean;
  children?: FileNode[] | null;
}

export async function listDirectoryContents(path: string): Promise<FileItem[]> {
  try {
    return await invoke<FileItem[]>("list_directory_contents", { path });
  } catch (err) {
    console.error("Error reading directory contents:", err);
    throw err;
  }
}

export async function getDirectoryTreeFromRoot(path: string): Promise<FileNode> {
  try {
    return await invoke<FileNode>("get_tree_from_root", { targetPath: path });
  } catch (err) {
    console.error("Error fetching tree from root:", err);
    throw err;
  }
}
