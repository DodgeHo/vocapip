UPDATE settings
SET reminder_interval_minutes = 30,
    updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
WHERE reminder_interval_minutes = 20;

PRAGMA user_version = 3;
