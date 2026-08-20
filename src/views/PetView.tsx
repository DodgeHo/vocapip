import { listen } from "@tauri-apps/api/event";
import { EyeOff, Pin, PinOff, Volume2, X } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
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
  const [pinBusy, setPinBusy] = useState(false);
  const mounted = useRef(false);
  const loadSequence = useRef(0);
  const pinRequest = useRef(0);
  const moodTimer = useRef<number | undefined>(undefined);
  const feedbackTimer = useRef<number | undefined>(undefined);

  const load = async () => {
    if (!mounted.current) return;
    const request = ++loadSequence.current;
    try {
      const next = await api.bootstrap();
      const assets = await api.getPetAssets();
      if (!mounted.current || request !== loadSequence.current) return;
      setData(next);
      setCard(next.currentCard ?? null);
      setCustomAssets(assets);
    } catch (cause) {
      if (mounted.current) {
        setError(
          cause instanceof Error ? cause.message : zhCN.common.unknownError,
        );
        setMood("error");
      }
    }
  };

  useEffect(() => {
    mounted.current = true;
    void load();
    return () => {
      mounted.current = false;
      if (moodTimer.current !== undefined)
        window.clearTimeout(moodTimer.current);
      if (feedbackTimer.current !== undefined)
        window.clearTimeout(feedbackTimer.current);
    };
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
    if (!mounted.current) return;
    setProgressOpen(false);
    setMood("click");
    setBubbleOpen(true);
    if (moodTimer.current !== undefined) window.clearTimeout(moodTimer.current);
    moodTimer.current = window.setTimeout(() => {
      if (mounted.current) setMood("idle");
    }, 420);
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

    const refresh = () => void load();
    void Promise.allSettled([
      listen("vocapip://reminder-due", openLearning),
      listen("vocapip://settings-changed", refresh),
      listen("vocapip://pet-assets-changed", refresh),
      listen("vocapip://pet-appearance-changed", refresh),
      listen("vocapip://backup-imported", refresh),
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
    ]).then((results) => {
      const failed = results.find((result) => result.status === "rejected");
      results.forEach((result) => {
        if (result.status === "fulfilled") {
          if (cancelled) result.value();
          else disposers.push(result.value);
        }
      });
      if (failed && mounted.current)
        setError(
          failed.reason instanceof Error
            ? `${zhCN.pet.eventSyncError}：${failed.reason.message}`
            : zhCN.pet.eventSyncError,
        );
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
    try {
      const next = await api.completePlacement(submission);
      if (!mounted.current) return;
      setData(next);
      setCard(next.currentCard ?? null);
    } catch (cause) {
      if (mounted.current) {
        setError(
          cause instanceof Error ? cause.message : zhCN.common.unknownError,
        );
        setMood("error");
      }
    }
  };

  const hidePet = async () => {
    try {
      await api.hidePet();
    } catch (cause) {
      if (mounted.current) {
        setError(
          cause instanceof Error ? cause.message : zhCN.common.unknownError,
        );
        setMood("error");
      }
    }
  };

  const toggleAlwaysOnTop = async () => {
    if (!data || pinBusy) return;
    const request = ++pinRequest.current;
    const enabled = !data.settings.alwaysOnTop;
    setPinBusy(true);
    try {
      const applied = await api.setAlwaysOnTop(enabled);
      if (mounted.current && request === pinRequest.current)
        setData((current) =>
          current
            ? {
                ...current,
                settings: { ...current.settings, alwaysOnTop: applied },
              }
            : current,
        );
    } catch (cause) {
      if (mounted.current && request === pinRequest.current) {
        setError(
          cause instanceof Error ? cause.message : zhCN.common.unknownError,
        );
        setMood("error");
      }
    } finally {
      if (mounted.current && request === pinRequest.current) setPinBusy(false);
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
      if (!mounted.current) return;
      setData((current) =>
        current
          ? {
              ...current,
              progress: outcome.progress,
              daily: outcome.daily,
              currentCard: outcome.nextCard,
            }
          : current,
      );
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
      if (feedbackTimer.current !== undefined)
        window.clearTimeout(feedbackTimer.current);
      feedbackTimer.current = window.setTimeout(() => {
        if (!mounted.current) return;
        setCard(outcome.nextCard ?? null);
        setFeedback("");
        setXpFeedback(0);
        setLeveledUp(false);
        setMood("idle");
        if (!outcome.nextCard) setBubbleOpen(false);
      }, 1050);
    } catch (cause) {
      if (mounted.current) {
        setError(
          cause instanceof Error ? cause.message : zhCN.common.unknownError,
        );
        setMood("error");
      }
    } finally {
      if (mounted.current) setBusy(false);
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
                disabled={pinBusy}
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
