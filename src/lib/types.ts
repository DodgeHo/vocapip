export type LevelBand = "T1" | "T2" | "T3" | "T4" | "T5";
export type ReviewRating = "again" | "hard" | "good" | "easy";
export type PetMood = "idle" | "click" | "error" | "correct" | "upgrade";
export type PetAppearanceId = "mint" | "coral" | "sun" | "night" | "custom";
export type PetAssetState = "default" | PetMood;
export type CustomPetAssets = Partial<Record<PetAssetState, string>>;
export interface PetAppearanceSummary {
  id: PetAppearanceId;
  bundled: boolean;
  selected?: boolean;
  availableStates?: PetAssetState[];
}
export interface PetAssetRecord {
  state: PetAssetState;
  dataUrl: string;
  id?: string;
  appearanceId?: PetAppearanceId;
  mimeType?: string;
  width?: number;
  height?: number;
  updatedAt?: string;
}
export type PetSurfaceMode = "idle" | "learning" | "progress" | "placement";
export type PetBubbleSide = "above" | "below";
export interface PetSurfaceLayout {
  mode: PetSurfaceMode;
  bubbleSide: PetBubbleSide;
}
export type CardStatus = "new" | "learning" | "review" | "relearning";
export interface FsrsCardSnapshot {
  due: string;
  stability: number;
  difficulty: number;
  elapsedDays: number;
  scheduledDays: number;
  learningSteps: number;
  reps: number;
  lapses: number;
  state: number;
  lastReview?: string;
}
export interface LearningCard {
  id: string;
  word: string;
  phonetic?: string;
  partOfSpeech: string;
  definitionZh: string;
  exampleEn: string;
  exampleZh: string;
  tier: LevelBand;
  frequencyRank: number;
  status: CardStatus;
  favorite: boolean;
  sourcePackId?: string;
  sourcePackName?: string;
  lastStudiedAt?: string;
  nextReviewAt?: string;
  fsrs?: FsrsCardSnapshot;
}
export interface DailyProgress {
  completed: number;
  goal: number;
  reviewsDue: number;
}
export interface PlayerProgress {
  xp: number;
  level: number;
  xpIntoLevel: number;
  xpForNextLevel: number;
  streakDays: number;
}
export interface LanguagePackSummary {
  id: string;
  name: string;
  sourceLanguage: string;
  targetLanguage: string;
  explanationLanguage: string;
  packVersion: string;
  compatibilityVersion: string;
  recordCount: number;
  uniqueWordCount: number;
  composite: boolean;
}
export interface AppSettings {
  reminderIntervalMinutes: number;
  dailyQuota: number;
  quietHoursEnabled: boolean;
  quietStart: string;
  quietEnd: string;
  autostart: boolean;
  alwaysOnTop: boolean;
  languagePackId: string;
  remindersPaused: boolean;
  speechEnabled: boolean;
  petAppearanceId: PetAppearanceId;
  quietModeUntil?: string;
}
export interface BootstrapPayload {
  progress: PlayerProgress;
  daily: DailyProgress;
  settings: AppSettings;
  languagePacks: LanguagePackSummary[];
  placementComplete: boolean;
  placementTier?: LevelBand;
  currentCard?: LearningCard;
}
export interface ReviewOutcome {
  card: LearningCard;
  progress: PlayerProgress;
  daily: DailyProgress;
  xpAwarded: number;
  leveledUp: boolean;
  feedback: string;
  nextCard?: LearningCard;
}
export interface LibraryQuery {
  search: string;
  status: "all" | CardStatus | "favorite" | "recent";
  limit: number;
  offset: number;
}
export interface LibraryPayload {
  items: LearningCard[];
  counts: Record<LibraryQuery["status"], number>;
  hasMore: boolean;
}
export interface PlacementAnswer {
  questionId: string;
  correct: boolean;
}
export interface PlacementSubmission {
  selfReportedTier: LevelBand;
  answers?: PlacementAnswer[];
}
export interface LicenseDocument {
  id: string;
  title: string;
  body: string;
}
