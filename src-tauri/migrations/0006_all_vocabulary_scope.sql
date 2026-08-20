-- The pilot pack was a bootstrap default, not a user choice. Existing users
-- still pointing at it are moved to the composite seven-book scope. Explicitly
-- selected packs are left untouched.
UPDATE settings
SET language_pack_id = 'zh-CN__en.vocabularybook.all',
    updated_at = datetime('now')
WHERE language_pack_id IN ('', 'zh-CN__en.pilot');

UPDATE learner_profile
SET active_pack_id = 'zh-CN__en.vocabularybook.all',
    updated_at = datetime('now')
WHERE active_pack_id IS NULL OR active_pack_id = 'zh-CN__en.pilot';

PRAGMA user_version = 6;
