import { describe, expect, it } from "vitest";
import type { PersistedTab, TabInfo } from "./tab-types.js";
import { createTabId } from "./tab-types.js";

describe("tab types and helpers", () => {
  it("createTabId returns UUID v4 format", () => {
    const id = createTabId();
    expect(id).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i);
  });

  it("createTabId returns unique values", () => {
    const ids = new Set(Array.from({ length: 100 }, () => createTabId()));
    expect(ids.size).toBe(100);
  });

  it("TabInfo accepts valid object", () => {
    const tab: TabInfo = { id: "abc", repoPath: null, repoName: "New Tab", dirty: false };
    expect(tab.id).toBe("abc");
    expect(tab.repoPath).toBeNull();
    expect(tab.repoName).toBe("New Tab");
    expect(tab.dirty).toBe(false);
  });

  it("PersistedTab is a subset of TabInfo", () => {
    const tab: TabInfo = { id: "123", repoPath: "/path/to/repo", repoName: "repo", dirty: true };
    const persisted: PersistedTab = { id: tab.id, repoPath: tab.repoPath, repoName: tab.repoName };
    expect(persisted.id).toBe(tab.id);
    expect(persisted.repoPath).toBe(tab.repoPath);
    expect(persisted.repoName).toBe(tab.repoName);
    // PersistedTab should not have 'dirty' — compile-time check, but runtime verify absence
    expect("dirty" in persisted).toBe(false);
  });
});
