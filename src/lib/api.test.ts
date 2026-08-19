// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { api, parseLearningCard } from "./api";

afterEach(() => {
  localStorage.clear();
  vi.restoreAllMocks();
});

beforeEach(() => localStorage.clear());

describe("frontend data contracts", () => {
  it("normalizes nullable vocabularybook fields without inventing content", () => {
    const card = parseLearningCard({
      id: "vocabularybook:middle-school:ab12",
      word: "abandon",
      phonetic: null,
      partOfSpeech: "v.",
      definitionZh: "放弃",
      exampleEn: null,
      exampleZh: null,
      tier: "T1",
      frequencyRank: null,
      status: "new",
      favorite: false,
    });

    expect(card.phonetic).toBeUndefined();
    expect(card.exampleEn).toBe("");
    expect(card.exampleZh).toBe("");
    expect(card.frequencyRank).toBe(0);
  });

  it("exposes all bundled appearances through the offline adapter", async () => {
    await expect(api.listPetAppearances()).resolves.toEqual([
      { id: "mint", bundled: true },
      { id: "coral", bundled: true },
      { id: "sun", bundled: true },
      { id: "night", bundled: true },
      { id: "custom", bundled: false },
    ]);
  });

  it("stores each custom pet state through the browser adapter", async () => {
    const dataUrl = "data:image/png;base64,AA==";
    await api.importPetImage("default", dataUrl);
    await api.importPetImage("upgrade", dataUrl);

    await expect(api.getPetAsset("default")).resolves.toMatchObject({
      state: "default",
      dataUrl,
    });
    await expect(api.getPetAsset("upgrade")).resolves.toMatchObject({
      state: "upgrade",
      dataUrl,
    });

    await api.removePetImage("default");
    await expect(api.getPetAsset("default")).resolves.toBeNull();
  });
});
