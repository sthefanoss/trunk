import { describe, expect, it } from "vitest";
import { createUndoRedoState } from "./undo-redo.svelte.js";

describe("createUndoRedoState", () => {
	it("starts with empty redoStack", () => {
		const mgr = createUndoRedoState();
		expect(mgr.state.redoStack).toHaveLength(0);
	});

	it("push adds entry", () => {
		const mgr = createUndoRedoState();
		mgr.push({ subject: "test", body: null });
		expect(mgr.state.redoStack).toHaveLength(1);
	});

	it("pop returns last pushed entry (LIFO)", () => {
		const mgr = createUndoRedoState();
		mgr.push({ subject: "first", body: null });
		mgr.push({ subject: "second", body: "desc" });
		const popped = mgr.pop();
		expect(popped).toEqual({ subject: "second", body: "desc" });
	});

	it("pop returns undefined on empty stack", () => {
		const mgr = createUndoRedoState();
		expect(mgr.pop()).toBeUndefined();
	});

	it("clear empties the stack", () => {
		const mgr = createUndoRedoState();
		mgr.push({ subject: "a", body: null });
		mgr.push({ subject: "b", body: null });
		mgr.push({ subject: "c", body: null });
		mgr.clear();
		expect(mgr.state.redoStack).toHaveLength(0);
	});

	it("instances are independent", () => {
		const a = createUndoRedoState();
		const b = createUndoRedoState();
		a.push({ subject: "only-on-a", body: null });
		expect(b.state.redoStack).toHaveLength(0);
		expect(b.pop()).toBeUndefined();
	});
});
