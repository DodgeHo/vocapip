-- The 100-word pilot alias was also a bootstrap default in early builds.
-- Move it to the composite scope without changing an explicitly selected pack.
UPDATE settings
SET language_pack_id = 'zh-CN__en.vocabularybook.all',
    updated_at = datetime('now')
WHERE language_pack_id = 'zh-CN__en.pilot.100';

UPDATE learner_profile
SET active_pack_id = 'zh-CN__en.vocabularybook.all',
    updated_at = datetime('now')
WHERE active_pack_id = 'zh-CN__en.pilot.100';

PRAGMA user_version = 7;
