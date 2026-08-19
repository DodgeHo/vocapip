import { useRef } from "react";
import type { PointerEvent } from "react";
import { zhCN } from "../i18n/zh-CN";
import type { PetMood } from "../lib/types";

interface PetCharacterProps {
  mood: PetMood;
  onActivate?: () => void;
  onRequestDrag?: () => void | Promise<void>;
  onContextMenu?: (x: number, y: number) => void;
  compact?: boolean;
}

const DRAG_THRESHOLD = 6;

export function PetCharacter({
  mood,
  onActivate,
  onRequestDrag,
  onContextMenu,
  compact = false,
}: PetCharacterProps) {
  const pointerStart = useRef<{ x: number; y: number } | null>(null);
  const dragged = useRef(false);

  const handlePointerDown = (event: PointerEvent<HTMLButtonElement>) => {
    if (event.button !== 0) return;
    pointerStart.current = { x: event.screenX, y: event.screenY };
    dragged.current = false;
  };

  const handlePointerMove = (event: PointerEvent<HTMLButtonElement>) => {
    const start = pointerStart.current;
    if (!start || dragged.current || (event.buttons & 1) === 0) return;
    const distance = Math.hypot(
      event.screenX - start.x,
      event.screenY - start.y,
    );
    if (distance < DRAG_THRESHOLD) return;
    dragged.current = true;
    void onRequestDrag?.();
  };

  const clearPointer = () => {
    pointerStart.current = null;
    window.setTimeout(() => {
      dragged.current = false;
    }, 0);
  };

  return (
    <button
      type="button"
      className={"pet-character pet-" + mood + (compact ? " pet-compact" : "")}
      onPointerDown={handlePointerDown}
      onPointerMove={handlePointerMove}
      onPointerUp={clearPointer}
      onPointerCancel={clearPointer}
      onClick={() => {
        if (!dragged.current) onActivate?.();
      }}
      onContextMenu={(event) => {
        event.preventDefault();
        onContextMenu?.(event.clientX, event.clientY);
      }}
      aria-label={zhCN.pet.ariaLabel}
    >
      <svg
        className="pet-illustration"
        viewBox="0 0 180 180"
        aria-hidden="true"
      >
        <g className="pet-tail-layer">
          <path
            className="pet-shape pet-tail-shape"
            d="M132 112c24-12 35 4 25 20-8 13-27 7-33-2"
          />
          <path
            className="pet-tail-tip"
            d="M153 120c5 5 3 12-3 15-6 3-13 0-16-5 8-1 14-4 19-10Z"
          />
        </g>
        <g className="pet-body-layer">
          <path
            className="pet-shape pet-body-shape"
            d="M51 105c3-24 19-37 39-37s36 13 39 37l5 42c1 12-8 22-20 22H66c-12 0-21-10-20-22l5-42Z"
          />
          <path
            className="pet-belly-shape"
            d="M68 112c7-10 14-14 22-14s15 4 22 14l4 32c1 8-5 14-13 14H77c-8 0-14-6-13-14l4-32Z"
          />
          <path
            className="pet-arm pet-arm-left"
            d="M55 111c-13 8-18 21-11 29 5 6 14 2 21-7"
          />
          <path
            className="pet-arm pet-arm-right"
            d="M125 111c13 8 18 21 11 29-5 6-14 2-21-7"
          />
          <path
            className="pet-foot"
            d="M57 162c7-7 17-8 24-1M99 161c8-6 18-5 24 2"
          />
          <g className="pet-book-badge">
            <path d="M74 119h32v26H74z" />
            <path d="M90 119v26M78 125h8M94 125h8" />
          </g>
        </g>
        <g className="pet-head-layer">
          <path
            className="pet-ear pet-ear-left"
            d="M51 55 45 17c-1-7 7-11 12-6l24 22"
          />
          <path
            className="pet-ear-inner pet-ear-inner-left"
            d="M54 42 51 24l13 13"
          />
          <path
            className="pet-ear pet-ear-right"
            d="m129 55 6-38c1-7-7-11-12-6L99 33"
          />
          <path
            className="pet-ear-inner pet-ear-inner-right"
            d="m126 42 3-18-13 13"
          />
          <path
            className="pet-shape pet-head-shape"
            d="M40 60c2-23 20-38 50-38s48 15 50 38v21c0 25-20 42-50 42S40 106 40 81V60Z"
          />
          <path
            className="pet-face-patch"
            d="M54 70c5-17 17-25 36-25s31 8 36 25v15c0 17-15 28-36 28S54 102 54 85V70Z"
          />
          <g className="pet-eyes">
            <path className="pet-eye pet-eye-left" d="M68 77v8" />
            <path className="pet-eye pet-eye-right" d="M112 77v8" />
          </g>
          <path className="pet-brow pet-brow-left" d="m62 69 12-3" />
          <path className="pet-brow pet-brow-right" d="m106 66 12 3" />
          <path className="pet-mouth" d="M83 92c4 5 10 5 14 0" />
          <path className="pet-nose" d="m86 87 4 3 4-3" />
          <circle className="pet-cheek pet-cheek-left" cx="65" cy="94" r="6" />
          <circle
            className="pet-cheek pet-cheek-right"
            cx="115"
            cy="94"
            r="6"
          />
        </g>
        <g className="pet-status-symbol">
          <circle cx="143" cy="42" r="18" />
          <path className="pet-status-check" d="m134 42 6 6 12-14" />
          <path className="pet-status-alert" d="M143 32v13m0 6v1" />
          <path className="pet-status-up" d="m135 44 8-10 8 10m-8-10v18" />
        </g>
        <g className="pet-upgrade-sparkles">
          <path d="M29 58v-14m-7 7h14M151 88V74m-7 7h14M35 118v-10m-5 5h10" />
        </g>
      </svg>
    </button>
  );
}
