import { listen } from "@tauri-apps/api/event";
import { EyeOff, Pin, PinOff, Volume2, X } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { Placement } from "../components/Placement";
import { PetCharacter } from "../components/PetCharacter";
import { ProgressStrip } from "../components/ProgressStrip";
import { zhCN } from "../i18n/zh-CN";
import { api } from "../lib/api";
import { formatInterval, previewReview, scheduleReview } from "../lib/fsrs";
import type {
  BootstrapPayload,
  CustomPetAssets,
  LearningCard,
  PetBubbleSide,
  PetMood,
  PetSurfaceMode,
  PlacementSubmission,
  ReviewRating,
} from "../lib/types";

const ratings: ReviewRating[] = ["again", "hard", "good", "easy"];

type PetActionPayload =
  | string
  | {
      action?: string;
      enabled?: boolean;
      paused?: boolean;
    };

function getPetAction(payload: PetActionPayload): string {
  return typeof payload === "string" ? payload : (payload.action ?? "");
}

export function PetView() {
  const [data, setData] = useState<BootstrapPayload | null>(null);
  const [customAssets, setCustomAssets] = useState<CustomPetAssets>({});
  const [card, setCard] = useState<LearningCard | null>(null);
  const [bubbleOpen, setBubbleOpen] = useState(false);
  const [progressOpen, setProgressOpen] = useState(false);
  const [bubbleSide, setBubbleSide] = useState<PetBubbleSide>("above");
  const [revealed, setRevealed] = useState(false);
  const [mood, setMood] = useState<PetMood>("idle");
  const [busy, setBusy] = useState(false);
  const [feedback, setFeedback] = useState("");
  const [xpFeedback, setXpFeedback] = useState(0);
  const [leveledUp, setLeveledUp] = useState(false);
  const [error, setError] = useState("");

  const load = async () => {
    try {
      const [next, assets] = await Promise.all([
        api.bootstrap(),
        api.getPetAssets(),
      ]);
      setData(next);
      setCard(next.currentCard ?? null);
      setCustomAssets(assets);
    } catch (cause) {
      setError(
        cause instanceof Error ? cause.message : zhCN.common.unknownError,
      );
      setMood("error");
    }
  };

  useEffect(() => {
    void load();
  }, []);

  const surfaceMode: PetSurfaceMode = !data?.placementComplete
    ? "placement"
    : bubbleOpen
      ? "learning"
      : progressOpen
        ? "progress"
        : "idle";

  useEffect(() => {
    if (!data) return;
    let cancelled = false;
    void api
      .setPetSurfaceMode(surfaceMode)
      .then((layout) => {
        if (!cancelled) setBubbleSide(layout.bubbleSide);
      })
      .catch((cause) => {
        if (!cancelled) {
          setError(
            cause instanceof Error ? cause.message : zhCN.common.unknownError,
          );
          setMood("error");
        }
      });
    return () => {
      cancelled = true;
    };
  }, [data, surfaceMode]);

  const openLearning = () => {
    setProgressOpen(false);
    setMood("click");
    setBubbleOpen(true);
    window.setTimeout(() => setMood("idle"), 420);
  };

  useEffect(() => {
    const word = card?.word;
    if (!bubbleOpen || !word || !data?.settings.speechEnabled) return;
    let cancelled = false;
    void api.speakText(word).catch(() => {
      if (!cancelled) setError(zhCN.pet.speechError);
    });
    return () => {
      cancelled = true;
      void api.stopSpeaking();
    };
  }, [bubbleOpen, card?.id, card?.word, data?.settings.speechEnabled]);

  useEffect(() => {
    if (!("__TAURI_INTERNALS__" in window)) return;
    const disposers: Array<() => void> = [];
    let cancelled = false;

    void Promise.all([
      listen("vocapip://reminder-due", openLearning),
      listen<PetActionPayload>("vocapip://pet-action", (event) => {
        const action = getPetAction(event.payload);
        if (action === "learn" || action === "study") {
          openLearning();
          return;
        }
        if (action === "always-on-top" || action === "toggle-always-on-top") {
          const enabled =
            typeof event.payload === "object" &&
            typeof event.payload.enabled === "boolean"
              ? event.payload.enabled
              : undefined;
          if (enabled !== undefined) {
            setData((current) =>
              current
                ? {
                    ...current,
                    settings: { ...current.settings, alwaysOnTop: enabled },
                  }
                : current,
            );
          }
          return;
        }
        if (action === "reminders-paused" || action === "toggle-reminders") {
          const paused =
            typeof event.payload === "object" &&
            typeof event.payload.paused === "boolean"
              ? event.payload.paused
              : undefined;
          if (paused !== undefined) {
            setData((current) =>
              current
                ? {
                    ...current,
                    settings: { ...current.settings, remindersPaused: paused },
                  }
                : current,
            );
          }
        }
      }),
    ]).then((nextDisposers) => {
      if (cancelled) nextDisposers.forEach((dispose) => dispose());
      else disposers.push(...nextDisposers);
    });

    return () => {
      cancelled = true;
      disposers.forEach((dispose) => dispose());
    };
  }, []);

  const previews = useMemo(
    () => (card && revealed ? previewReview(card.fsrs) : null),
    [card, revealed],
  );

  const completePlacement = async (submission: PlacementSubmission) => {
    const next = await api.completePlacement(submission);
    setData(next);
    setCard(next.currentCard ?? null);
  };

  const hidePet = async () => {
    try {
      await api.hidePet();
    } catch (cause) {
      setError(
        cause instanceof Error ? cause.message : zhCN.common.unknownError,
      );
      setMood("error");
    }
  };

  const toggleAlwaysOnTop = async () => {
    if (!data) return;
    const enabled = !data.settings.alwaysOnTop;
    try {
      const applied = await api.setAlwaysOnTop(enabled);
      setData({
        ...data,
        settings: { ...data.settings, alwaysOnTop: applied },
      });
    } catch (cause) {
      setError(
        cause instanceof Error ? cause.message : zhCN.common.unknownError,
      );
      setMood("error");
    }
  };

  const closeLearning = () => {
    void api.stopSpeaking();
    setBubbleOpen(false);
    setRevealed(false);
    setFeedback("");
    setXpFeedback(0);
    setLeveledUp(false);
  };

  const submit = async (rating: ReviewRating) => {
    if (!card || !data) return;
    void api.stopSpeaking();
    setBusy(true);
    setError("");
    const now = new Date();
    const nextFsrs = scheduleReview(card.fsrs, rating, now);
    try {
      const outcome = await api.submitReview(
        card.id,
        rating,
        now.toISOString(),
        nextFsrs,
      );
      setData({
        ...data,
        progress: outcome.progress,
        daily: outcome.daily,
        currentCard: outcome.nextCard,
      });
      setFeedback(
        outcome.leveledUp
          ? zhCN.pet.feedback.upgrade
          : zhCN.pet.feedback[rating],
      );
      setXpFeedback(outcome.xpAwarded);
      setLeveledUp(outcome.leveledUp);
      setMood(
        outcome.leveledUp
          ? "upgrade"
          : rating === "again"
            ? "error"
            : "correct",
      );
      setRevealed(false);
      window.setTimeout(() => {
        setCard(outcome.nextCard ?? null);
        setFeedback("");
        setXpFeedback(0);
        setLeveledUp(false);
        setMood("idle");
        if (!outcome.nextCard) setBubbleOpen(false);
      }, 1050);
    } catch (cause) {
      setError(
        cause instanceof Error ? cause.message : zhCN.common.unknownError,
      );
      setMood("error");
    } finally {
      setBusy(false);
    }
  };

  if (!data && !error) {
    return (
      <div className="pet-window loading-state">{zhCN.common.loading}</div>
    );
  }

  return (
    <div className={`pet-window pet-surface-${surfaceMode}`}>
      {data && (
        <div className={`pet-stage bubble-${bubbleSide}`}>
          <section className="pet-companion" aria-label={zhCN.pet.surfaceLabel}>
            <div className="pet-inline-controls">
              <button
                type="button"
                className={data.settings.alwaysOnTop ? "is-active" : ""}
                title={
                  data.settings.alwaysOnTop
                    ? zhCN.pet.unpinPet
                    : zhCN.pet.pinPet
                }
                aria-label={
                  data.settings.alwaysOnTop
                    ? zhCN.pet.unpinPet
                    : zhCN.pet.pinPet
                }
                aria-pressed={data.settings.alwaysOnTop}
                onClick={() => void toggleAlwaysOnTop()}
              >
                {data.settings.alwaysOnTop ? (
                  <PinOff aria-hidden="true" size={17} />
                ) : (
                  <Pin aria-hidden="true" size={17} />
                )}
              </button>
              <button
                type="button"
                title={zhCN.pet.hideToTray}
                aria-label={zhCN.pet.hideToTray}
                onClick={() => void hidePet()}
              >
                <EyeOff aria-hidden="true" size={17} />
              </button>
            </div>
            <PetCharacter
              mood={mood}
              appearanceId={data.settings.petAppearanceId}
              customAssets={customAssets}
              onActivate={openLearning}
              onRequestDrag={() => api.startPetDrag()}
              onContextMenu={() => void api.showPetContextMenu()}
            />
            <span className="pet-prompt">
              {bubbleOpen
                ? feedback || (card?.word ?? zhCN.pet.noCardsTitle)
                : zhCN.pet.clickToLearn}
            </span>
            <ProgressStrip
              daily={data.daily}
              progress={data.progress}
              open={progressOpen}
              side={bubbleSide}
              xpFeedback={xpFeedback}
              leveledUp={leveledUp}
              onToggle={() => {
                closeLearning();
                setProgressOpen((open) => !open);
              }}
              onClose={() => setProgressOpen(false)}
            />
          </section>

          {bubbleOpen && (
            <section className="learning-bubble" aria-live="polite">
              {feedback ? (
                <div className="feedback-message">
                  <strong>{feedback}</strong>
                  <span>
                    +{xpFeedback} XP · Lv. {data.progress.level}
                  </span>
                </div>
              ) : card ? (
                <>
                  <header>
                    <div>
                      <span className="tier-chip">{card.tier}</span>
                      <span className="pos-chip">{card.partOfSpeech}</span>
                    </div>
                    <button
                      type="button"
                      className="close-button"
                      aria-label={zhCN.common.close}
                      onClick={closeLearning}
                    >
                      <X aria-hidden="true" size={18} />
                    </button>
                  </header>
                  <div className="word-line">
                    <h1>{card.word}</h1>
                    <span className="word-pronunciation">
                      {card.phonetic && <span>{card.phonetic}</span>}
                      <button
                        type="button"
                        className="icon-button speech-button"
                        aria-label={zhCN.pet.speakWord}
                        title={zhCN.pet.speakWord}
                        onClick={() => {
                          void api.speakText(card.word).catch(() => {
                            setError(zhCN.pet.speechError);
                          });
                        }}
                      >
                        <Volume2 aria-hidden="true" size={18} />
                      </button>
                    </span>
                  </div>
                  <p className="example-en">
                    {card.exampleEn || zhCN.pet.noExample}
                  </p>
                  {!revealed ? (
                    <button
                      type="button"
                      className="reveal-button"
                      onClick={() => setRevealed(true)}
                    >
                      {zhCN.pet.reveal}
                    </button>
                  ) : (
                    <>
                      <div className="answer-block">
                        <strong>{card.definitionZh}</strong>
                        <p>{card.exampleZh || zhCN.pet.noExample}</p>
                      </div>
                      <div className="rating-grid">
                        {ratings.map((rating) => (
                          <button
                            type="button"
                            disabled={busy}
                            className={"rating-" + rating}
                            onClick={() => void submit(rating)}
                            key={rating}
                          >
                            <strong>{zhCN.pet.ratings[rating]}</strong>
                            <span>
                              {previews
                                ? formatInterval(previews[rating])
                                : zhCN.pet.ratingHints[rating]}
                            </span>
                          </button>
                        ))}
                      </div>
                    </>
                  )}
                </>
              ) : (
                <div className="empty-bubble">
                  <strong>{zhCN.pet.noCardsTitle}</strong>
                  <p>{zhCN.pet.noCardsBody}</p>
                </div>
              )}
              {error && <p className="inline-error">{error}</p>}
            </section>
          )}
        </div>
      )}
      {data && !data.placementComplete && (
        <Placement onComplete={completePlacement} />
      )}
      {error && !data && (
        <div className="fatal-error">
          <p>{error}</p>
          <button type="button" onClick={() => void load()}>
            {zhCN.common.retry}
          </button>
        </div>
      )}
    </div>
  );
}
