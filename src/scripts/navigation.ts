import { invoke } from "@tauri-apps/api/core";

/**
 * Represents a single file or folder item in a flat directory listing.
 */
export interface FileItem {
  name: string;
  path: string;
  is_dir: boolean;
  size?: number | null;
}

/**
 * Represents a recursive node in the file tree.
 */
export interface FileNode {
  name: string;
  path: string;
  is_dir: boolean;
  children?: FileNode[] | null;
}

/**
 * Fetches the immediate contents of a directory (non-recursive).
 * This is used for lazy-loading folder contents or listing files.
 * 
 * @param path - Absolute or relative path to the directory
 * @returns A promise resolving to a sorted list of FileItem objects
 */
export async function listDirectoryContents(path: string): Promise<FileItem[]> {
  try {
    const result = await invoke<FileItem[]>("list_directory_contents", { path });
    return result;
  } catch (err) {
    console.error(`Error reading directory contents:`, err);
    throw err;
  }
}

/**
 * Recursively fetches a directory tree up to a specified depth.
 * Useful for showing a sidebar tree view.
 * 
 * @param path - Absolute or relative path to the root directory
 * @param depth - How deep to traverse (e.g., 3 means 3 levels down)
 * @returns A promise resolving to a FileNode tree
 */
export async function getDirectoryTree(path: string, depth = 2): Promise<FileNode> {
  try {
    const result = await invoke<FileNode>("get_directory_tree", { path, depth });
    return result;
  } catch (err) {
    console.error(`Error fetching directory tree:`, err);
    throw err;
  }
}
