import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { disable, enable, isEnabled } from "@tauri-apps/plugin-autostart";
import { z } from "zod";
import { createFsrsCard } from "./fsrs";
import type {
  AppSettings,
  BootstrapPayload,
  FsrsCardSnapshot,
  LearningCard,
  LibraryPayload,
  LibraryQuery,
  LicenseDocument,
  LevelBand,
  PetSurfaceLayout,
  PetSurfaceMode,
  PlacementSubmission,
  ReviewOutcome,
  ReviewRating,
} from "./types";

export interface CommandContract {
  bootstrap: { args: undefined; result: BootstrapPayload };
  get_next_card: {
    args: { excludeCardId?: string };
    result: LearningCard | null;
  };
  submit_review: {
    args: {
      cardId: string;
      rating: ReviewRating;
      reviewedAt: string;
      fsrsCard: FsrsCardSnapshot;
    };
    result: ReviewOutcome;
  };
  list_library: { args: { query: LibraryQuery }; result: LibraryPayload };
  toggle_favorite: { args: { cardId: string }; result: LearningCard };
  queue_relearn: { args: { cardId: string }; result: LearningCard };
  update_settings: { args: { settings: AppSettings }; result: AppSettings };
  complete_placement: {
    args: { submission: PlacementSubmission };
    result: BootstrapPayload;
  };
  export_data: { args: undefined; result: { path: string } | null };
  import_data: { args: undefined; result: BootstrapPayload | null };
  open_window: { args: { view: "pet" | "library" | "settings" }; result: void };
  hide_main: { args: undefined; result: void };
  set_pet_surface_mode: {
    args: { mode: PetSurfaceMode };
    result: PetSurfaceLayout;
  };
  show_pet_context_menu: { args: undefined; result: void };
  set_always_on_top: { args: { enabled: boolean }; result: boolean };
  set_reminders_paused: { args: { paused: boolean }; result: boolean };
  set_quiet_mode_until: {
    args: { until: string | null };
    result: string | null;
  };
  get_license_documents: { args: undefined; result: LicenseDocument[] };
}

const levelSchema = z.enum(["T1", "T2", "T3", "T4", "T5"]);
const cardSchema = z.object({
  id: z.string(),
  word: z.string(),
  phonetic: z.string().optional(),
  partOfSpeech: z.string(),
  definitionZh: z.string(),
  exampleEn: z.string(),
  exampleZh: z.string(),
  tier: levelSchema,
  frequencyRank: z.number(),
  status: z.enum(["new", "learning", "review", "relearning"]),
  favorite: z.boolean(),
  lastStudiedAt: z.string().optional(),
  nextReviewAt: z.string().optional(),
  fsrs: z
    .object({
      due: z.string(),
      stability: z.number(),
      difficulty: z.number(),
      elapsedDays: z.number(),
      scheduledDays: z.number(),
      learningSteps: z.number(),
      reps: z.number(),
      lapses: z.number(),
      state: z.number(),
      lastReview: z.string().optional(),
    })
    .optional(),
});
const settingsSchema = z.object({
  reminderIntervalMinutes: z.number(),
  dailyQuota: z.number(),
  quietHoursEnabled: z.boolean(),
  quietStart: z.string(),
  quietEnd: z.string(),
  autostart: z.boolean(),
  alwaysOnTop: z.boolean(),
  languagePackId: z.string(),
  remindersPaused: z.boolean(),
  quietModeUntil: z.string().optional(),
});
const bootstrapSchema = z.object({
  progress: z.object({
    xp: z.number(),
    level: z.number(),
    xpIntoLevel: z.number(),
    xpForNextLevel: z.number(),
    streakDays: z.number(),
  }),
  daily: z.object({
    completed: z.number(),
    goal: z.number(),
    reviewsDue: z.number(),
  }),
  settings: settingsSchema,
  languagePacks: z.array(
    z.object({
      id: z.string(),
      name: z.string(),
      sourceLanguage: z.string(),
      targetLanguage: z.string(),
      explanationLanguage: z.string(),
      packVersion: z.string(),
      compatibilityVersion: z.string(),
      recordCount: z.number(),
    }),
  ),
  placementComplete: z.boolean(),
  placementTier: levelSchema.optional(),
  currentCard: cardSchema.optional(),
});

const demoCards: LearningCard[] = [
  [
    "lp-001",
    "steady",
    "/ˈstedi/",
    "adj.",
    "稳定的；持续的",
    "She made steady progress by reviewing a few words every day.",
    "她每天复习几个单词，取得了稳定的进步。",
    "T2",
    812,
  ],
  [
    "lp-002",
    "notice",
    "/ˈnoʊtɪs/",
    "v.",
    "注意到；察觉",
    "I noticed a small change in his voice.",
    "我注意到他的声音有一点变化。",
    "T2",
    604,
  ],
  [
    "lp-003",
    "reliable",
    "/rɪˈlaɪəbl/",
    "adj.",
    "可靠的；可信赖的",
    "A reliable routine makes difficult work easier.",
    "可靠的习惯会让困难的工作变得容易。",
    "T3",
    1736,
  ],
  [
    "lp-004",
    "approach",
    "/əˈproʊtʃ/",
    "n.",
    "方法；处理方式",
    "We need a practical approach to the problem.",
    "我们需要一种务实的方法来处理这个问题。",
    "T3",
    538,
  ],
  [
    "lp-005",
    "brief",
    "/briːf/",
    "adj.",
    "简短的；短暂的",
    "The team held a brief meeting before lunch.",
    "团队在午餐前开了一个简短的会议。",
    "T2",
    1269,
  ],
  [
    "lp-006",
    "maintain",
    "/meɪnˈteɪn/",
    "v.",
    "维持；保养",
    "It is easier to maintain a habit than to restart it.",
    "保持一个习惯比重新开始更容易。",
    "T3",
    897,
  ],
  [
    "lp-007",
    "subtle",
    "/ˈsʌtl/",
    "adj.",
    "细微的；不明显的",
    "There is a subtle difference between the two plans.",
    "这两个计划之间有细微的差别。",
    "T4",
    2910,
  ],
  [
    "lp-008",
    "allocate",
    "/ˈæləkeɪt/",
    "v.",
    "分配；划拨",
    "We allocated more time to testing.",
    "我们为测试分配了更多时间。",
    "T4",
    3461,
  ],
  [
    "lp-009",
    "calm",
    "/kɑːm/",
    "adj.",
    "平静的；镇定的",
    "Take a calm look at the facts first.",
    "先冷静地看看事实。",
    "T1",
    1427,
  ],
  [
    "lp-010",
    "gather",
    "/ˈɡæðər/",
    "v.",
    "收集；聚集",
    "She gathered the notes into one folder.",
    "她把笔记收集到一个文件夹里。",
    "T2",
    978,
  ],
  [
    "lp-011",
    "coherent",
    "/koʊˈhɪrənt/",
    "adj.",
    "连贯的；条理清楚的",
    "The report presents a coherent argument.",
    "这份报告提出了连贯的论点。",
    "T5",
    5208,
  ],
  [
    "lp-012",
    "adapt",
    "/əˈdæpt/",
    "v.",
    "适应；调整",
    "Good tools adapt to the way people work.",
    "好的工具会适应人们的工作方式。",
    "T3",
    1560,
  ],
].map((item, index) => ({
  id: item[0] as string,
  word: item[1] as string,
  phonetic: item[2] as string,
  partOfSpeech: item[3] as string,
  definitionZh: item[4] as string,
  exampleEn: item[5] as string,
  exampleZh: item[6] as string,
  tier: item[7] as LevelBand,
  frequencyRank: item[8] as number,
  status: index === 0 ? "learning" : index < 4 ? "review" : "new",
  favorite: index === 2 || index === 6,
  lastStudiedAt:
    index < 4
      ? new Date(Date.now() - index * 86400000).toISOString()
      : undefined,
  fsrs: createFsrsCard(),
}));

interface DemoState {
  bootstrap: BootstrapPayload;
  cards: LearningCard[];
  reviewKeys: string[];
}
const storageKey = "vocapip-demo-v2";
const legacyStorageKey = "languagepet-demo-v2";
function freshDemo(): DemoState {
  const settings: AppSettings = {
    reminderIntervalMinutes: 30,
    dailyQuota: 12,
    quietHoursEnabled: true,
    quietStart: "22:30",
    quietEnd: "08:00",
    autostart: false,
    alwaysOnTop: false,
    languagePackId: "zh-en-core",
    remindersPaused: false,
  };
  return {
    cards: structuredClone(demoCards),
    reviewKeys: [],
    bootstrap: {
      progress: {
        xp: 72,
        level: 3,
        xpIntoLevel: 22,
        xpForNextLevel: 60,
        streakDays: 4,
      },
      daily: { completed: 3, goal: 12, reviewsDue: 4 },
      settings,
      languagePacks: [
        {
          id: "zh-en-core",
          name: "简体中文 → 英语核心词包",
          sourceLanguage: "zh-CN",
          targetLanguage: "en",
          explanationLanguage: "zh-CN",
          packVersion: "0.1.0",
          compatibilityVersion: "1",
          recordCount: 100,
        },
      ],
      placementComplete: false,
      currentCard: structuredClone(demoCards[0]),
    },
  };
}
function loadDemo(): DemoState {
  try {
    const value =
      localStorage.getItem(storageKey) ??
      localStorage.getItem(legacyStorageKey);
    if (!value) return freshDemo();
    const state = JSON.parse(value) as DemoState;
    localStorage.setItem(storageKey, value);
    return state;
  } catch {
    return freshDemo();
  }
}
function storeDemo(state: DemoState): void {
  try {
    localStorage.setItem(storageKey, JSON.stringify(state));
  } catch {
    /* private browser sessions may reject persistence */
  }
}
function isTauri(): boolean {
  return "__TAURI_INTERNALS__" in window;
}
async function call<K extends keyof CommandContract>(
  command: K,
  args: CommandContract[K]["args"],
): Promise<CommandContract[K]["result"]> {
  return invoke<CommandContract[K]["result"]>(command, args ?? {});
}
function nextDemoCard(
  state: DemoState,
  excludeCardId?: string,
): LearningCard | null {
  return (
    state.cards.find(
      (card) =>
        card.id !== excludeCardId &&
        (card.status === "relearning" ||
          card.status === "review" ||
          card.status === "new"),
    ) ?? null
  );
}

export const api = {
  async bootstrap(): Promise<BootstrapPayload> {
    if (isTauri())
      return bootstrapSchema.parse(await call("bootstrap", undefined));
    const state = loadDemo();
    state.bootstrap.currentCard = nextDemoCard(state) ?? undefined;
    return structuredClone(state.bootstrap);
  },
  async getNextCard(excludeCardId?: string): Promise<LearningCard | null> {
    if (isTauri()) {
      const result = await call("get_next_card", { excludeCardId });
      return result ? cardSchema.parse(result) : null;
    }
    return structuredClone(nextDemoCard(loadDemo(), excludeCardId));
  },
  async submitReview(
    cardId: string,
    rating: ReviewRating,
    reviewedAt: string,
    fsrsCard: FsrsCardSnapshot,
  ): Promise<ReviewOutcome> {
    if (isTauri())
      return call("submit_review", { cardId, rating, reviewedAt, fsrsCard });
    const state = loadDemo();
    const index = state.cards.findIndex((card) => card.id === cardId);
    if (index < 0) throw new Error("Card not found");
    const card = state.cards[index];
    const reviewKey = cardId + ":" + reviewedAt.slice(0, 16);
    const xpAwarded = state.reviewKeys.includes(reviewKey)
      ? 0
      : ({ again: 2, hard: 5, good: 8, easy: 10 } as const)[rating];
    state.reviewKeys.push(reviewKey);
    const oldLevel = state.bootstrap.progress.level;
    const total = state.bootstrap.progress.xp + xpAwarded;
    const level = Math.floor(total / 50) + 2;
    state.bootstrap.progress = {
      xp: total,
      level,
      xpIntoLevel: total % 50,
      xpForNextLevel: 50,
      streakDays: state.bootstrap.progress.streakDays,
    };
    state.bootstrap.daily.completed = Math.min(
      state.bootstrap.daily.goal,
      state.bootstrap.daily.completed + 1,
    );
    state.bootstrap.daily.reviewsDue = Math.max(
      0,
      state.bootstrap.daily.reviewsDue - 1,
    );
    const updated: LearningCard = {
      ...card,
      status: rating === "again" ? "relearning" : "review",
      lastStudiedAt: reviewedAt,
      nextReviewAt: fsrsCard.due,
      fsrs: fsrsCard,
    };
    state.cards[index] = updated;
    const nextCard = nextDemoCard(state, cardId) ?? undefined;
    state.bootstrap.currentCard = nextCard;
    storeDemo(state);
    return {
      card: updated,
      progress: state.bootstrap.progress,
      daily: state.bootstrap.daily,
      xpAwarded,
      leveledUp: level > oldLevel,
      feedback: rating,
      nextCard,
    };
  },
  async listLibrary(query: LibraryQuery): Promise<LibraryPayload> {
    if (isTauri()) return call("list_library", { query });
    const cards = loadDemo().cards;
    const lower = query.search.trim().toLocaleLowerCase();
    const recentCutoff = Date.now() - 7 * 86400000;
    const filtered = cards.filter(
      (card) =>
        (!lower ||
          card.word.toLocaleLowerCase().includes(lower) ||
          card.definitionZh.includes(lower)) &&
        (query.status === "all" ||
          (query.status === "favorite"
            ? card.favorite
            : query.status === "recent"
              ? Boolean(
                  card.lastStudiedAt &&
                    new Date(card.lastStudiedAt).getTime() >= recentCutoff,
                )
              : card.status === query.status)),
    );
    const count = (status: LibraryQuery["status"]) =>
      cards.filter(
        (card) =>
          status === "all" ||
          (status === "favorite"
            ? card.favorite
            : status === "recent"
              ? Boolean(
                  card.lastStudiedAt &&
                    new Date(card.lastStudiedAt).getTime() >= recentCutoff,
                )
              : card.status === status),
      ).length;
    return {
      items: structuredClone(filtered),
      counts: {
        all: count("all"),
        new: count("new"),
        learning: count("learning"),
        review: count("review"),
        relearning: count("relearning"),
        favorite: count("favorite"),
        recent: count("recent"),
      },
    };
  },
  async toggleFavorite(cardId: string): Promise<LearningCard> {
    if (isTauri())
      return cardSchema.parse(await call("toggle_favorite", { cardId }));
    const state = loadDemo();
    const card = state.cards.find((item) => item.id === cardId);
    if (!card) throw new Error("Card not found");
    card.favorite = !card.favorite;
    storeDemo(state);
    return structuredClone(card);
  },
  async queueRelearn(cardId: string): Promise<LearningCard> {
    if (isTauri())
      return cardSchema.parse(await call("queue_relearn", { cardId }));
    const state = loadDemo();
    const card = state.cards.find((item) => item.id === cardId);
    if (!card) throw new Error("Card not found");
    card.status = "relearning";
    card.nextReviewAt = new Date().toISOString();
    storeDemo(state);
    return structuredClone(card);
  },
  async updateSettings(settings: AppSettings): Promise<AppSettings> {
    if (isTauri())
      return settingsSchema.parse(await call("update_settings", { settings }));
    const state = loadDemo();
    state.bootstrap.settings = settings;
    storeDemo(state);
    return structuredClone(settings);
  },
  async completePlacement(
    submission: PlacementSubmission,
  ): Promise<BootstrapPayload> {
    if (isTauri())
      return bootstrapSchema.parse(
        await call("complete_placement", { submission }),
      );
    const state = loadDemo();
    state.bootstrap.placementComplete = true;
    state.bootstrap.placementTier = submission.selfReportedTier;
    state.bootstrap.currentCard = nextDemoCard(state) ?? undefined;
    storeDemo(state);
    return structuredClone(state.bootstrap);
  },
  async openWindow(view: "pet" | "library" | "settings"): Promise<void> {
    if (isTauri()) return call("open_window", { view });
    window.location.href = "?view=" + view;
  },
  async hidePet(): Promise<void> {
    if (isTauri()) return call("hide_main", undefined);
  },
  async setPetSurfaceMode(mode: PetSurfaceMode): Promise<PetSurfaceLayout> {
    if (isTauri()) return call("set_pet_surface_mode", { mode });
    return { mode, bubbleSide: "above" };
  },
  async showPetContextMenu(): Promise<void> {
    if (isTauri()) return call("show_pet_context_menu", undefined);
  },
  async startPetDrag(): Promise<void> {
    if (!isTauri()) return;
    await getCurrentWindow().startDragging();
  },
  async setAlwaysOnTop(enabled: boolean): Promise<boolean> {
    if (isTauri()) return call("set_always_on_top", { enabled });
    return enabled;
  },
  async setRemindersPaused(paused: boolean): Promise<boolean> {
    if (isTauri()) return call("set_reminders_paused", { paused });
    return paused;
  },
  async setQuietModeUntil(until: string | null): Promise<string | null> {
    if (isTauri()) return call("set_quiet_mode_until", { until });
    return until;
  },
  async setAutostart(enabled: boolean): Promise<boolean> {
    if (!isTauri()) return enabled;
    if (enabled) await enable();
    else await disable();
    return isEnabled();
  },
  async exportData(): Promise<string | null> {
    if (isTauri()) {
      const result = await call("export_data", undefined);
      return result?.path ?? null;
    }
    const blob = new Blob([JSON.stringify(loadDemo(), null, 2)], {
      type: "application/json",
    });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = "VocaPip-demo-backup.json";
    anchor.click();
    URL.revokeObjectURL(url);
    return anchor.download;
  },
  async importData(): Promise<BootstrapPayload | null> {
    if (isTauri()) {
      const result = await call("import_data", undefined);
      return result ? bootstrapSchema.parse(result) : null;
    }
    return new Promise((resolve, reject) => {
      const input = document.createElement("input");
      input.type = "file";
      input.accept = ".json,application/json";
      input.onchange = async () => {
        try {
          const file = input.files?.[0];
          if (!file) return resolve(null);
          const parsed = JSON.parse(await file.text()) as DemoState;
          const checked = bootstrapSchema.parse(parsed.bootstrap);
          storeDemo(parsed);
          resolve(checked);
        } catch (error) {
          reject(error);
        }
      };
      input.click();
    });
  },
  async getLicenseDocuments(): Promise<LicenseDocument[]> {
    if (isTauri()) return call("get_license_documents", undefined);
    return [
      {
        id: "app",
        title: "VocaPip 应用代码",
        body: "GNU General Public License v3.0 or later (GPL-3.0-or-later)",
      },
      {
        id: "fsrs",
        title: "ts-fsrs",
        body: "MIT License. Copyright ts-fsrs contributors.",
      },
      {
        id: "pack",
        title: "演示语言包",
        body: "浏览器演示内容仅用于界面验证；正式语言包许可证和字段级来源由桌面端许可证文档提供。",
      },
    ];
  },
};
