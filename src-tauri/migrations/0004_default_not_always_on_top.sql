UPDATE settings
SET always_on_top = 0,
    updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
WHERE id = 1;

PRAGMA user_version = 4;
