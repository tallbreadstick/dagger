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

/**
 * Resolves the current user's home directory using the Tauri backend.
 * @returns A promise that resolves to the home directory path as a string.
 */
export async function resolveUserHome(): Promise<string> {
  try {
    const homeDir = await invoke<string>("resolve_user");
    return homeDir;
  } catch (err) {
    console.error("Failed to resolve user home directory:", err);
    throw err;
  }
}

/**
 * Opens a file or directory using the system default handler.
 * Also registers the access in the recent list on the backend.
 * @param path - The absolute path to open.
 */
export async function openFromPath(path: string): Promise<void> {
  try {
    await invoke("open_from_path", { path });
  } catch (err) {
    console.error(`Failed to open path "${path}":`, err);
    throw err;
  }
}
