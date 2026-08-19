PRAGMA foreign_keys = ON;
PRAGMA application_id = 1280333140;

CREATE TABLE IF NOT EXISTS language_packs (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  source_language TEXT NOT NULL,
  target_language TEXT NOT NULL,
  explanation_language TEXT NOT NULL,
  pack_version TEXT NOT NULL,
  compatibility_version TEXT NOT NULL,
  record_count INTEGER NOT NULL,
  pack_license TEXT NOT NULL,
  imported_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS lexemes (
  id TEXT PRIMARY KEY,
  pack_id TEXT NOT NULL REFERENCES language_packs(id) ON DELETE RESTRICT,
  word TEXT NOT NULL,
  phonetic TEXT,
  part_of_speech TEXT NOT NULL,
  definition_zh TEXT NOT NULL,
  example_en TEXT NOT NULL,
  example_zh TEXT NOT NULL,
  tier TEXT NOT NULL CHECK (tier IN ('T1', 'T2', 'T3', 'T4', 'T5')),
  frequency_rank INTEGER NOT NULL CHECK (frequency_rank >= 0),
  record_license TEXT NOT NULL,
  source_id TEXT NOT NULL,
  source_url TEXT NOT NULL,
  provenance_json TEXT NOT NULL,
  learning_ready INTEGER NOT NULL DEFAULT 1 CHECK (learning_ready IN (0, 1)),
  UNIQUE(pack_id, word)
);

CREATE TABLE IF NOT EXISTS study_cards (
  id TEXT PRIMARY KEY,
  lexeme_id TEXT NOT NULL UNIQUE REFERENCES lexemes(id) ON DELETE CASCADE,
  status TEXT NOT NULL DEFAULT 'new' CHECK (status IN ('new', 'learning', 'review', 'relearning')),
  due_at TEXT,
  stability REAL,
  difficulty REAL,
  elapsed_days INTEGER,
  scheduled_days INTEGER,
  learning_steps INTEGER,
  reps INTEGER NOT NULL DEFAULT 0,
  lapses INTEGER NOT NULL DEFAULT 0,
  fsrs_state INTEGER,
  last_review_at TEXT,
  scheduler_snapshot TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS review_logs (
  id TEXT PRIMARY KEY,
  card_id TEXT NOT NULL REFERENCES study_cards(id) ON DELETE CASCADE,
  rating TEXT NOT NULL CHECK (rating IN ('again', 'hard', 'good', 'easy')),
  reviewed_at TEXT NOT NULL,
  review_day TEXT NOT NULL,
  duration_ms INTEGER,
  mode TEXT NOT NULL DEFAULT 'self-rating',
  is_objective INTEGER NOT NULL DEFAULT 0 CHECK (is_objective IN (0, 1)),
  is_correct INTEGER CHECK (is_correct IN (0, 1)),
  xp_awarded INTEGER NOT NULL,
  scheduler_snapshot TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS learner_profile (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  self_reported_tier TEXT NOT NULL DEFAULT 'T1',
  placed_tier TEXT,
  placement_confidence REAL,
  current_comfort_tier TEXT NOT NULL DEFAULT 'T1',
  tier_mode TEXT NOT NULL DEFAULT 'adaptive' CHECK (tier_mode IN ('adaptive', 'locked')),
  active_pack_id TEXT,
  placement_complete INTEGER NOT NULL DEFAULT 0 CHECK (placement_complete IN (0, 1)),
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS placement_attempts (
  id TEXT PRIMARY KEY,
  started_at TEXT NOT NULL,
  completed_at TEXT NOT NULL,
  question_count INTEGER NOT NULL,
  highest_stable_tier TEXT,
  recommended_tier TEXT NOT NULL,
  confidence REAL NOT NULL,
  answers_json TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS user_progress (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  total_xp INTEGER NOT NULL DEFAULT 0,
  level INTEGER NOT NULL DEFAULT 1,
  streak_days INTEGER NOT NULL DEFAULT 0,
  last_study_day TEXT,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS settings (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  reminder_interval_minutes INTEGER NOT NULL DEFAULT 30,
  daily_quota INTEGER NOT NULL DEFAULT 10,
  quiet_hours_enabled INTEGER NOT NULL DEFAULT 0 CHECK (quiet_hours_enabled IN (0, 1)),
  quiet_start TEXT NOT NULL DEFAULT '22:00',
  quiet_end TEXT NOT NULL DEFAULT '08:00',
  autostart INTEGER NOT NULL DEFAULT 0 CHECK (autostart IN (0, 1)),
  always_on_top INTEGER NOT NULL DEFAULT 0 CHECK (always_on_top IN (0, 1)),
  language_pack_id TEXT NOT NULL DEFAULT 'zh-CN__en.pilot',
  reminders_paused INTEGER NOT NULL DEFAULT 0 CHECK (reminders_paused IN (0, 1)),
  quiet_mode_until TEXT,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS favorites (
  card_id TEXT PRIMARY KEY REFERENCES study_cards(id) ON DELETE CASCADE,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS relearn_queue (
  card_id TEXT PRIMARY KEY REFERENCES study_cards(id) ON DELETE CASCADE,
  queued_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS daily_stats (
  study_day TEXT PRIMARY KEY,
  completed INTEGER NOT NULL DEFAULT 0,
  xp_earned INTEGER NOT NULL DEFAULT 0,
  new_cards INTEGER NOT NULL DEFAULT 0,
  review_cards INTEGER NOT NULL DEFAULT 0,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS learning_records (
  id TEXT PRIMARY KEY,
  card_id TEXT NOT NULL REFERENCES study_cards(id) ON DELETE CASCADE,
  event_type TEXT NOT NULL,
  occurred_at TEXT NOT NULL,
  payload_json TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS app_runtime (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

INSERT OR IGNORE INTO learner_profile (id, updated_at) VALUES (1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));
INSERT OR IGNORE INTO user_progress (id, updated_at) VALUES (1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));
INSERT OR IGNORE INTO settings (id, updated_at) VALUES (1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'));

PRAGMA user_version = 1;
