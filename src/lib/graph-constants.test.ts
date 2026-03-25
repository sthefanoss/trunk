import { describe, expect, it } from "vitest";
import {
  DOT_RADIUS,
  EDGE_STROKE,
  LANE_WIDTH,
  MERGE_STROKE,
  ROW_HEIGHT,
} from "./graph-constants.js";

describe("graph-constants", () => {
  describe("unified constants", () => {
    it("LANE_WIDTH is 16", () => expect(LANE_WIDTH).toBe(16));
    it("ROW_HEIGHT is 26", () => expect(ROW_HEIGHT).toBe(26));
    it("DOT_RADIUS is 6", () => expect(DOT_RADIUS).toBe(6));
    it("EDGE_STROKE is 1.5", () => expect(EDGE_STROKE).toBe(1.5));
    it("MERGE_STROKE is 2", () => expect(MERGE_STROKE).toBe(2));
  });
});
