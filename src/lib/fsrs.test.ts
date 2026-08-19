import { describe, expect, it } from "vitest";
import {
  createFsrsCard,
  formatInterval,
  previewReview,
  scheduleReview,
} from "./fsrs";
describe("FSRS adapter", () => {
  it("creates serializable snapshots for all four ratings", () => {
    const now = new Date("2026-08-18T08:00:00.000Z");
    const card = createFsrsCard(now);
    const previews = previewReview(card, now);
    expect(Object.keys(previews)).toEqual(["again", "hard", "good", "easy"]);
    expect(previews.again.reps).toBe(1);
    expect(new Date(previews.easy.due).getTime()).toBeGreaterThan(
      new Date(previews.again.due).getTime(),
    );
    expect(formatInterval(previews.good, now)).toMatch(/分钟|小时|天|个月/);
  });
  it("applies the selected rating without mutating the stored snapshot", () => {
    const now = new Date("2026-08-18T08:00:00.000Z");
    const card = createFsrsCard(now);
    const before = JSON.stringify(card);
    const next = scheduleReview(card, "good", now);
    expect(JSON.stringify(card)).toBe(before);
    expect(next.reps).toBe(1);
    expect(next.lastReview).toBe(now.toISOString());
  });
});
