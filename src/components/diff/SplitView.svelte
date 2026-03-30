<script lang="ts">
import type { ContentMode, DiffLine, DiffOrigin, FileDiff } from "../../lib/types.js";

interface Props {
  contentMode: ContentMode;
  fileDiffs: FileDiff[];
  selectedPath: string | null;
  diffKind: "unstaged" | "staged" | "commit";
  hunkOperationInFlight: boolean;
  ignoreWhitespace: boolean;
  showInvisibles: boolean;
  wordWrap: boolean;
  selectedHunkKey: string | null;
  selectedLineIndices: Set<number>;
  selectedCount: number;
  collapsedFiles: Set<string>;
  hunkElements: Record<string, HTMLDivElement>;
  onfilecollapsetoggle: (path: string) => void;
  onlineclick: (filePath: string, hunkIdx: number, lineIndex: number, origin: DiffOrigin, hunkLines: DiffLine[], e: MouseEvent) => void;
  onstagehunk: (filePath: string, hunkIndex: number) => void;
  onunstagehunk: (filePath: string, hunkIndex: number) => void;
  ondiscardhunk: (filePath: string, hunkIndex: number) => void;
  onstagelines: (filePath: string, hunkIndex: number) => void;
  onunstagelines: (filePath: string, hunkIndex: number) => void;
  ondiscardlines: (filePath: string, hunkIndex: number) => void;
}

let { contentMode, fileDiffs, ...rest }: Props = $props();
</script>

<div class="stub-view">
  Split view — coming soon
</div>

<style>
  .stub-view {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--color-text-muted);
    font-size: 13px;
  }
</style>
