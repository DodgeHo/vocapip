import { useRef } from "react";
import type { PointerEvent, ReactNode } from "react";
import { zhCN } from "../i18n/zh-CN";
import type { CustomPetAssets, PetAppearanceId, PetMood } from "../lib/types";

interface PetCharacterProps {
  mood: PetMood;
  appearanceId?: PetAppearanceId;
  customAssets?: CustomPetAssets;
  onActivate?: () => void;
  onRequestDrag?: () => void | Promise<void>;
  onContextMenu?: (x: number, y: number) => void;
  compact?: boolean;
}

interface PetAppearancePreviewProps {
  appearanceId: PetAppearanceId;
  customAssets?: CustomPetAssets;
  mood?: PetMood;
}

const DRAG_THRESHOLD = 6;

function StatusArtwork() {
  return (
    <>
      <g className="pet-status-symbol">
        <circle cx="143" cy="42" r="18" />
        <path className="pet-status-check" d="m134 42 6 6 12-14" />
        <path className="pet-status-alert" d="M143 32v13m0 6v1" />
        <path className="pet-status-up" d="m135 44 8-10 8 10m-8-10v18" />
      </g>
      <g className="pet-upgrade-sparkles">
        <path d="M29 58v-14m-7 7h14M151 88V74m-7 7h14M35 118v-10m-5 5h10" />
      </g>
    </>
  );
}

function MintArtwork() {
  return (
    <svg className="pet-illustration" viewBox="0 0 180 180" aria-hidden="true">
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
        <path className="pet-ear-inner" d="M54 42 51 24l13 13" />
        <path
          className="pet-ear pet-ear-right"
          d="m129 55 6-38c1-7-7-11-12-6L99 33"
        />
        <path className="pet-ear-inner" d="m126 42 3-18-13 13" />
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
        <circle className="pet-cheek pet-cheek-right" cx="115" cy="94" r="6" />
      </g>
      <StatusArtwork />
    </svg>
  );
}

function CoralArtwork() {
  return (
    <svg className="pet-illustration" viewBox="0 0 180 180" aria-hidden="true">
      <path
        className="pet-shape pet-tail-shape"
        d="M128 121c30-16 38 7 23 21-9 8-21 5-24-4 15 5 22-12 7-15"
      />
      <path
        className="pet-shape pet-body-shape"
        d="M55 104c5-20 18-31 35-31s30 11 35 31l6 42c2 13-7 23-20 23H69c-13 0-22-10-20-23l6-42Z"
      />
      <path
        className="pet-belly-shape"
        d="M69 117c6-9 13-13 21-13s15 4 21 13l2 32c1 7-5 12-12 12H79c-7 0-13-5-12-12l2-32Z"
      />
      <path
        className="pet-arm pet-arm-left"
        d="M58 116c-13 8-15 21-7 27 6 4 13-1 18-9"
      />
      <path
        className="pet-arm pet-arm-right"
        d="M122 116c13 8 15 21 7 27-6 4-13-1-18-9"
      />
      <path className="pet-foot" d="M58 162h25m14 0h25" />
      <path className="pet-ear" d="M51 55 48 18l28 22M129 55l3-37-28 22" />
      <path className="pet-ear-inner" d="m55 42-2-16 14 12m58 4 2-16-14 12" />
      <path
        className="pet-shape pet-head-shape"
        d="M43 58c3-24 22-38 47-38s44 14 47 38v27c0 24-19 40-47 40S43 109 43 85V58Z"
      />
      <path
        className="pet-face-patch"
        d="M57 75c4-15 15-23 33-23s29 8 33 23v14c0 16-14 26-33 26S57 105 57 89V75Z"
      />
      <path className="pet-stripe" d="M78 24v19m12-22v23m12-20v19" />
      <path className="pet-eye pet-eye-left" d="M68 79v8" />
      <path className="pet-eye pet-eye-right" d="M112 79v8" />
      <path className="pet-brow pet-brow-left" d="m63 71 11-3" />
      <path className="pet-brow pet-brow-right" d="m106 68 11 3" />
      <path className="pet-mouth" d="M78 96c7 1 9-3 12-6 3 3 5 7 12 6" />
      <path className="pet-nose" d="m86 88 4 3 4-3Z" />
      <circle className="pet-cheek" cx="63" cy="98" r="6" />
      <circle className="pet-cheek" cx="117" cy="98" r="6" />
      <StatusArtwork />
    </svg>
  );
}

function SunArtwork() {
  return (
    <svg className="pet-illustration" viewBox="0 0 180 180" aria-hidden="true">
      <path
        className="pet-tuft"
        d="M77 28c5-14 14-18 21-9-1-10 9-14 15-7 4 5 0 14-9 20"
      />
      <path
        className="pet-shape pet-body-shape pet-bird-body"
        d="M43 86c0-38 19-61 47-61s47 23 47 61v25c0 34-18 56-47 56s-47-22-47-56V86Z"
      />
      <path
        className="pet-belly-shape"
        d="M62 102c0-24 11-38 28-38s28 14 28 38v27c0 20-11 31-28 31s-28-11-28-31v-27Z"
      />
      <path
        className="pet-wing pet-arm-left"
        d="M52 91c-19 5-25 18-17 31 6 9 17 5 26-7"
      />
      <path
        className="pet-wing pet-arm-right"
        d="M128 91c19 5 25 18 17 31-6 9-17 5-26-7"
      />
      <path className="pet-foot" d="M67 165v8m-9 0h18m37-8v8m-9 0h18" />
      <path className="pet-eye pet-eye-left" d="M68 75v9" />
      <path className="pet-eye pet-eye-right" d="M112 75v9" />
      <path className="pet-brow pet-brow-left" d="m62 67 12-2" />
      <path className="pet-brow pet-brow-right" d="m106 65 12 2" />
      <path className="pet-beak" d="m78 91 12-8 12 8-12 10-12-10Z" />
      <circle className="pet-cheek" cx="61" cy="96" r="6" />
      <circle className="pet-cheek" cx="119" cy="96" r="6" />
      <StatusArtwork />
    </svg>
  );
}

function NightArtwork() {
  return (
    <svg className="pet-illustration" viewBox="0 0 180 180" aria-hidden="true">
      <path className="pet-orbit" d="M29 105c18 17 105 22 127-7" />
      <path
        className="pet-orbit-dot"
        d="M28 105a7 7 0 1 0 14 0 7 7 0 0 0-14 0Zm121-7a6 6 0 1 0 12 0 6 6 0 0 0-12 0Z"
      />
      <path
        className="pet-shape pet-body-shape pet-star-body"
        d="M90 17c11 0 17 23 25 30 8 8 32 5 36 15 4 11-17 23-21 34-4 10 8 31-1 38-9 6-27-9-39-9s-30 15-39 9c-9-7 3-28-1-38-4-11-25-23-21-34 4-10 28-7 36-15 8-7 14-30 25-30Z"
      />
      <path
        className="pet-face-patch"
        d="M56 71c8-15 19-22 34-22s26 7 34 22v25c0 18-14 29-34 29S56 114 56 96V71Z"
      />
      <path
        className="pet-arm pet-arm-left"
        d="M50 91c-15 4-22 14-18 22 4 8 14 7 24-2"
      />
      <path
        className="pet-arm pet-arm-right"
        d="M130 91c15 4 22 14 18 22-4 8-14 7-24-2"
      />
      <path
        className="pet-foot"
        d="M64 139c3 17 13 25 24 14m28-14c-3 17-13 25-24 14"
      />
      <path className="pet-eye pet-eye-left" d="M70 79v9" />
      <path className="pet-eye pet-eye-right" d="M110 79v9" />
      <path className="pet-brow pet-brow-left" d="m64 70 11-3" />
      <path className="pet-brow pet-brow-right" d="m105 67 11 3" />
      <path className="pet-mouth" d="M82 99c5 5 11 5 16 0" />
      <circle className="pet-cheek" cx="65" cy="101" r="6" />
      <circle className="pet-cheek" cx="115" cy="101" r="6" />
      <StatusArtwork />
    </svg>
  );
}

export function resolveCustomPetImage(
  assets: CustomPetAssets | undefined,
  mood: PetMood,
): string | null {
  if (!assets) return null;
  return (
    assets[mood] ??
    assets.default ??
    assets.idle ??
    Object.values(assets).find(Boolean) ??
    null
  );
}

function PetArtwork({
  appearanceId,
  customAssets,
  mood = "idle",
}: PetAppearancePreviewProps): ReactNode {
  const customImageDataUrl = resolveCustomPetImage(customAssets, mood);
  if (appearanceId === "custom" && customImageDataUrl) {
    return (
      <span className="pet-custom-artwork">
        <img className="pet-custom-image" src={customImageDataUrl} alt="" />
        <svg
          className="pet-illustration pet-custom-overlay"
          viewBox="0 0 180 180"
          aria-hidden="true"
        >
          <StatusArtwork />
        </svg>
      </span>
    );
  }
  if (appearanceId === "coral") return <CoralArtwork />;
  if (appearanceId === "sun") return <SunArtwork />;
  if (appearanceId === "night") return <NightArtwork />;
  return <MintArtwork />;
}

export function PetAppearancePreview({
  appearanceId,
  customAssets,
  mood = "idle",
}: PetAppearancePreviewProps) {
  return (
    <span
      className={`pet-preview-artwork pet-appearance-${appearanceId}`}
      aria-hidden="true"
    >
      <PetArtwork
        appearanceId={appearanceId}
        customAssets={customAssets}
        mood={mood}
      />
    </span>
  );
}

export function PetCharacter({
  mood,
  appearanceId = "mint",
  customAssets,
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
      className={`pet-character pet-${mood} pet-appearance-${appearanceId}${compact ? " pet-compact" : ""}`}
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
      <PetArtwork
        appearanceId={appearanceId}
        customAssets={customAssets}
        mood={mood}
      />
    </button>
  );
}
