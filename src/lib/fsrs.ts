import {
  Rating,
  State,
  createEmptyCard,
  fsrs,
  type Card,
  type CardInput,
  type Grade,
} from "ts-fsrs";
import { zhCN } from "../i18n/zh-CN";
import type { FsrsCardSnapshot, ReviewRating } from "./types";
const scheduler = fsrs({
  request_retention: 0.9,
  maximum_interval: 36500,
  enable_fuzz: false,
  enable_short_term: true,
  learning_steps: ["1m", "10m"],
  relearning_steps: ["10m"],
});
const ratingMap: Record<ReviewRating, Grade> = {
  again: Rating.Again,
  hard: Rating.Hard,
  good: Rating.Good,
  easy: Rating.Easy,
};
function toCardInput(snapshot?: FsrsCardSnapshot, now = new Date()): CardInput {
  if (!snapshot) return createEmptyCard(now);
  return {
    due: snapshot.due,
    stability: snapshot.stability,
    difficulty: snapshot.difficulty,
    elapsed_days: snapshot.elapsedDays,
    scheduled_days: snapshot.scheduledDays,
    learning_steps: snapshot.learningSteps,
    reps: snapshot.reps,
    lapses: snapshot.lapses,
    state: snapshot.state as State,
    last_review: snapshot.lastReview,
  };
}
function serializeCard(card: Card): FsrsCardSnapshot {
  return {
    due: card.due.toISOString(),
    stability: card.stability,
    difficulty: card.difficulty,
    elapsedDays: card.elapsed_days,
    scheduledDays: card.scheduled_days,
    learningSteps: card.learning_steps,
    reps: card.reps,
    lapses: card.lapses,
    state: card.state,
    lastReview: card.last_review?.toISOString(),
  };
}
export function createFsrsCard(now = new Date()): FsrsCardSnapshot {
  return serializeCard(createEmptyCard(now));
}
export function scheduleReview(
  snapshot: FsrsCardSnapshot | undefined,
  rating: ReviewRating,
  now = new Date(),
): FsrsCardSnapshot {
  return serializeCard(
    scheduler.next(toCardInput(snapshot, now), now, ratingMap[rating]).card,
  );
}
export function previewReview(
  snapshot: FsrsCardSnapshot | undefined,
  now = new Date(),
): Record<ReviewRating, FsrsCardSnapshot> {
  const preview = scheduler.repeat(toCardInput(snapshot, now), now);
  return {
    again: serializeCard(preview[Rating.Again].card),
    hard: serializeCard(preview[Rating.Hard].card),
    good: serializeCard(preview[Rating.Good].card),
    easy: serializeCard(preview[Rating.Easy].card),
  };
}
export function formatInterval(
  snapshot: FsrsCardSnapshot,
  now = new Date(),
): string {
  const minutes = Math.max(
    1,
    Math.round((new Date(snapshot.due).getTime() - now.getTime()) / 60000),
  );
  if (minutes < 60) return minutes + " " + zhCN.common.intervalUnits.minute;
  const hours = Math.round(minutes / 60);
  if (hours < 24) return hours + " " + zhCN.common.intervalUnits.hour;
  const days = Math.round(hours / 24);
  if (days < 30) return days + " " + zhCN.common.intervalUnits.day;
  return Math.round(days / 30) + " " + zhCN.common.intervalUnits.month;
}
