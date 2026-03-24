import type { FileStatus } from './types.js';

export interface DirectoryNode {
  type: 'directory';
  name: string;       // Display name, may be compressed ("src/lib")
  path: string;       // Full relative path prefix ("src/lib")
  children: TreeNode[];
}

export interface FileNode {
  type: 'file';
  name: string;       // Filename only ("App.svelte")
  path: string;       // Full relative path ("src/App.svelte")
  file: FileStatus;   // Original FileStatus for downstream rendering
}

export type TreeNode = DirectoryNode | FileNode;

/** Stub — returns empty array. Implementation in Task 2. */
export function buildTree(_files: FileStatus[]): TreeNode[] {
  return [];
}
