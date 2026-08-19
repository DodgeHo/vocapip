import { Flame, Target, TimerReset, TrendingUp } from "lucide-react";
import { zhCN } from "../i18n/zh-CN";
import type {
  DailyProgress,
  PetBubbleSide,
  PlayerProgress,
} from "../lib/types";

interface ProgressStripProps {
  daily: DailyProgress;
  progress: PlayerProgress;
  open: boolean;
  side: PetBubbleSide;
  xpFeedback?: number;
  leveledUp?: boolean;
  onToggle: () => void;
  onClose: () => void;
}

export function ProgressStrip({
  daily,
  progress,
  open,
  side,
  xpFeedback = 0,
  leveledUp = false,
  onToggle,
  onClose,
}: ProgressStripProps) {
  const dailyPercent = Math.min(
    100,
    Math.round((daily.completed / Math.max(1, daily.goal)) * 100),
  );
  const xpPercent = Math.min(
    100,
    Math.round(
      (progress.xpIntoLevel / Math.max(1, progress.xpForNextLevel)) * 100,
    ),
  );
  const xpRemaining = Math.max(
    0,
    progress.xpForNextLevel - progress.xpIntoLevel,
  );

  return (
    <div
      className={"progress-dock progress-dock-" + side}
      onBlur={(event) => {
        if (!event.currentTarget.contains(event.relatedTarget)) onClose();
      }}
    >
      {open && (
        <div
          className="progress-popover"
          role="dialog"
          aria-label={zhCN.pet.progressDetails}
        >
          <div className="progress-detail-row">
            <Target aria-hidden="true" size={16} />
            <span>{zhCN.pet.todayGoal}</span>
            <strong>
              {daily.completed} / {daily.goal}
            </strong>
          </div>
          <div className="progress-detail-row">
            <TimerReset aria-hidden="true" size={16} />
            <span>{zhCN.pet.reviewDue}</span>
            <strong>{daily.reviewsDue}</strong>
          </div>
          <div className="progress-detail-row">
            <Flame aria-hidden="true" size={16} />
            <span>{zhCN.pet.streak}</span>
            <strong>
              {progress.streakDays}
              {zhCN.pet.days}
            </strong>
          </div>
          <div className="progress-detail-row">
            <TrendingUp aria-hidden="true" size={16} />
            <span>{zhCN.pet.untilNextLevel}</span>
            <strong>{xpRemaining} XP</strong>
          </div>
        </div>
      )}
      <button
        type="button"
        className={
          "progress-base" + (leveledUp ? " progress-base-upgrade" : "")
        }
        onClick={onToggle}
        aria-expanded={open}
        aria-label={zhCN.pet.progressDetails}
      >
        <span className="level-medallion">Lv.{progress.level}</span>
        <span className="progress-base-copy">
          <span className="progress-base-label">
            <strong>{daily.completed}</strong> / {daily.goal} {zhCN.pet.today}
          </span>
          <span className="progress-meter" aria-hidden="true">
            <span style={{ transform: `scaleX(${dailyPercent / 100})` }} />
          </span>
          <span className="xp-meter" aria-hidden="true">
            <span style={{ transform: `scaleX(${xpPercent / 100})` }} />
          </span>
        </span>
        <span className="xp-value">{progress.xpIntoLevel} XP</span>
        {xpFeedback > 0 && <span className="xp-burst">+{xpFeedback} XP</span>}
      </button>
    </div>
  );
}
