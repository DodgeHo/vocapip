CREATE INDEX IF NOT EXISTS idx_lexemes_pack_tier_rank
  ON lexemes(pack_id, learning_ready, tier, frequency_rank);
CREATE INDEX IF NOT EXISTS idx_lexemes_search
  ON lexemes(word, definition_zh);
CREATE INDEX IF NOT EXISTS idx_cards_due
  ON study_cards(status, due_at);
CREATE INDEX IF NOT EXISTS idx_cards_last_review
  ON study_cards(last_review_at DESC);
CREATE INDEX IF NOT EXISTS idx_review_logs_card_day
  ON review_logs(card_id, review_day, reviewed_at);
CREATE INDEX IF NOT EXISTS idx_review_logs_day
  ON review_logs(review_day);
CREATE INDEX IF NOT EXISTS idx_learning_records_card_time
  ON learning_records(card_id, occurred_at DESC);

PRAGMA user_version = 2;
