ALTER TABLE settings
  ADD COLUMN speech_enabled INTEGER NOT NULL DEFAULT 0
  CHECK (speech_enabled IN (0, 1));

ALTER TABLE settings
  ADD COLUMN pet_appearance_id TEXT NOT NULL DEFAULT 'mint'
  CHECK (pet_appearance_id IN ('mint', 'coral', 'sun', 'night', 'custom'));

CREATE TABLE pet_assets (
  id TEXT PRIMARY KEY,
  appearance_id TEXT NOT NULL DEFAULT 'custom'
    CHECK (appearance_id = 'custom'),
  state TEXT NOT NULL
    CHECK (state IN ('default', 'idle', 'click', 'error', 'correct', 'upgrade')),
  mime_type TEXT NOT NULL
    CHECK (mime_type IN ('image/png', 'image/jpeg', 'image/webp')),
  width INTEGER NOT NULL CHECK (width BETWEEN 1 AND 4096),
  height INTEGER NOT NULL CHECK (height BETWEEN 1 AND 4096),
  bytes BLOB NOT NULL CHECK (length(bytes) BETWEEN 1 AND 5242880),
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE(appearance_id, state)
);

PRAGMA user_version = 5;
