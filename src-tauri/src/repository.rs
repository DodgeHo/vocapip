use crate::db::DbState;
use crate::error::{AppError, AppResult};
use crate::models::*;
use crate::xp::{award_for_review, level_from_xp};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use rusqlite::{params, Connection, OptionalExtension, Row, Transaction};
use std::collections::BTreeMap;
use uuid::Uuid;

pub const ALL_LANGUAGE_PACK_ID: &str = "zh-CN__en.vocabularybook.all";

const CARD_SELECT: &str = "
SELECT c.id, l.word, l.phonetic, l.part_of_speech, l.definition_zh, l.example_en,
       l.example_zh, l.tier, l.frequency_rank, c.status,
       EXISTS(SELECT 1 FROM favorites f WHERE f.card_id = c.id),
       c.last_review_at, c.due_at, c.stability, c.difficulty, c.elapsed_days,
       c.scheduled_days, c.learning_steps, c.reps, c.lapses, c.fsrs_state,
       l.pack_id,
       (SELECT p.name FROM language_packs p WHERE p.id = l.pack_id)
FROM study_cards c
JOIN lexemes l ON l.id = c.lexeme_id";

pub fn bootstrap(state: &DbState) -> AppResult<BootstrapPayload> {
    let connection = state.connect()?;
    bootstrap_with_connection(&connection)
}

pub fn bootstrap_with_connection(connection: &Connection) -> AppResult<BootstrapPayload> {
    let settings = read_settings(connection)?;
    let progress = read_progress(connection)?;
    let daily = read_daily(connection, settings.daily_quota)?;
    let (placement_complete, placement_tier) = connection.query_row(
        "SELECT placement_complete, placed_tier FROM learner_profile WHERE id = 1",
        [],
        |row| Ok((row.get::<_, i64>(0)? != 0, row.get::<_, Option<String>>(1)?)),
    )?;
    let placement_tier = placement_tier
        .as_deref()
        .map(LevelBand::try_from)
        .transpose()
        .map_err(AppError::InvalidData)?;
    let current_card = if placement_complete {
        get_next_card_with_connection(connection, None)?
    } else {
        None
    };

    let mut statement = connection.prepare(
        "SELECT p.id, p.name, p.source_language, p.target_language, p.explanation_language,
                p.pack_version, p.compatibility_version, p.record_count,
                COUNT(DISTINCT LOWER(TRIM(l.word)))
         FROM language_packs p
         LEFT JOIN lexemes l ON l.pack_id = p.id
         GROUP BY p.id ORDER BY p.name",
    )?;
    let language_packs = statement
        .query_map([], |row| {
            Ok(LanguagePackSummary {
                id: row.get(0)?,
                name: row.get(1)?,
                source_language: row.get(2)?,
                target_language: row.get(3)?,
                explanation_language: row.get(4)?,
                pack_version: row.get(5)?,
                compatibility_version: row.get(6)?,
                record_count: row.get(7)?,
                unique_word_count: row.get(8)?,
                composite: false,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let (record_count, unique_word_count): (i64, i64) = connection.query_row(
        "SELECT COUNT(*), COUNT(DISTINCT LOWER(TRIM(word)))
         FROM lexemes
         WHERE pack_id LIKE 'zh-CN__en.vocabularybook.%'",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let mut language_packs = language_packs;
    language_packs.insert(
        0,
        LanguagePackSummary {
            id: ALL_LANGUAGE_PACK_ID.into(),
            name: "全部词库".into(),
            source_language: "zh-CN".into(),
            target_language: "en".into(),
            explanation_language: "zh-CN".into(),
            pack_version: "2026.08.19".into(),
            compatibility_version: "2".into(),
            record_count,
            unique_word_count,
            composite: true,
        },
    );

    Ok(BootstrapPayload {
        progress,
        daily,
        settings,
        language_packs,
        placement_complete,
        placement_tier,
        current_card,
    })
}

pub fn get_next_card(
    state: &DbState,
    exclude_card_id: Option<&str>,
) -> AppResult<Option<LearningCard>> {
    let connection = state.connect()?;
    get_next_card_with_connection(&connection, exclude_card_id)
}

pub fn get_next_card_with_connection(
    connection: &Connection,
    exclude_card_id: Option<&str>,
) -> AppResult<Option<LearningCard>> {
    let now = Utc::now().to_rfc3339();
    let excluded = exclude_card_id.unwrap_or("");
    let due_sql = format!(
        "{CARD_SELECT}
         WHERE c.id != ?1 AND (
           c.id IN (SELECT card_id FROM relearn_queue)
           OR (c.status != 'new' AND c.due_at IS NOT NULL AND c.due_at <= ?2)
         )
         ORDER BY CASE WHEN c.id IN (SELECT card_id FROM relearn_queue) THEN 0 ELSE 1 END,
                  CASE c.status WHEN 'relearning' THEN 0 WHEN 'learning' THEN 1 ELSE 2 END,
                  c.due_at ASC LIMIT 1"
    );
    if let Some(card) = connection
        .query_row(&due_sql, params![excluded, now], card_from_row)
        .optional()?
    {
        return Ok(Some(card));
    }

    let (active_pack, comfort_tier): (String, String) = connection.query_row(
        "SELECT COALESCE(active_pack_id, (SELECT id FROM language_packs ORDER BY imported_at LIMIT 1)), current_comfort_tier
         FROM learner_profile WHERE id = 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let comfort = LevelBand::try_from(comfort_tier.as_str()).map_err(AppError::InvalidData)?;
    let completed: i64 =
        connection.query_row("SELECT COUNT(*) FROM review_logs", [], |row| row.get(0))?;
    let challenge_slot = completed % 5 == 4;
    let preferred_tier = if challenge_slot {
        LevelBand::from_index((comfort.index() + 1).min(5))
    } else {
        comfort
    };

    let pack_filter = "((?2 = 'zh-CN__en.vocabularybook.all' AND l.pack_id LIKE 'zh-CN__en.vocabularybook.%' AND NOT EXISTS (
           SELECT 1 FROM lexemes duplicate
           WHERE LOWER(TRIM(duplicate.word)) = LOWER(TRIM(l.word))
             AND duplicate.pack_id LIKE 'zh-CN__en.vocabularybook.%'
             AND (
               CAST(SUBSTR(duplicate.tier, 2) AS INTEGER) < CAST(SUBSTR(l.tier, 2) AS INTEGER)
               OR (CAST(SUBSTR(duplicate.tier, 2) AS INTEGER) = CAST(SUBSTR(l.tier, 2) AS INTEGER)
                   AND COALESCE(duplicate.frequency_rank, 2147483647) < COALESCE(l.frequency_rank, 2147483647))
               OR (CAST(SUBSTR(duplicate.tier, 2) AS INTEGER) = CAST(SUBSTR(l.tier, 2) AS INTEGER)
                   AND COALESCE(duplicate.frequency_rank, 2147483647) = COALESCE(l.frequency_rank, 2147483647)
                   AND duplicate.id < l.id)
             )
         )) OR (?2 <> 'zh-CN__en.vocabularybook.all' AND l.pack_id = ?2))";
    let new_sql = if challenge_slot {
        format!(
            "{CARD_SELECT}
             WHERE c.id != ?1 AND c.status = 'new' AND l.learning_ready = 1
               AND {pack_filter} AND l.tier = ?3
             ORDER BY l.frequency_rank ASC, l.id ASC LIMIT 1"
        )
    } else {
        format!(
            "{CARD_SELECT}
             WHERE c.id != ?1 AND c.status = 'new' AND l.learning_ready = 1
               AND {pack_filter} AND CAST(SUBSTR(l.tier, 2) AS INTEGER) <= ?3
             ORDER BY CAST(SUBSTR(l.tier, 2) AS INTEGER) DESC, l.frequency_rank ASC, l.id ASC LIMIT 1"
        )
    };
    let tier_parameter = if challenge_slot {
        preferred_tier.as_str().to_string()
    } else {
        comfort.index().to_string()
    };
    let preferred = connection
        .query_row(
            &new_sql,
            params![excluded, &active_pack, tier_parameter],
            card_from_row,
        )
        .optional()?;
    if preferred.is_some() || !challenge_slot {
        return Ok(preferred);
    }

    let fallback_sql = format!(
        "{CARD_SELECT}
         WHERE c.id != ?1 AND c.status = 'new' AND l.learning_ready = 1 AND {pack_filter}
         ORDER BY ABS(CAST(SUBSTR(l.tier, 2) AS INTEGER) - ?3), l.frequency_rank ASC LIMIT 1"
    );
    Ok(connection
        .query_row(
            &fallback_sql,
            params![excluded, &active_pack, comfort.index()],
            card_from_row,
        )
        .optional()?)
}

#[allow(clippy::too_many_arguments)]
pub fn submit_review(
    state: &DbState,
    card_id: &str,
    rating: ReviewRating,
    reviewed_at: &str,
    fsrs: &FsrsCardSnapshot,
) -> AppResult<ReviewOutcome> {
    let reviewed = DateTime::parse_from_rfc3339(reviewed_at)
        .map_err(|_| AppError::InvalidData("reviewedAt must be an RFC 3339 timestamp".into()))?
        .with_timezone(&Utc);
    DateTime::parse_from_rfc3339(&fsrs.due)
        .map_err(|_| AppError::InvalidData("fsrsCard.due must be an RFC 3339 timestamp".into()))?;
    if fsrs.stability < 0.0
        || fsrs.difficulty < 0.0
        || fsrs.elapsed_days < 0
        || fsrs.scheduled_days < 0
        || fsrs.learning_steps < 0
        || fsrs.reps < 0
        || fsrs.lapses < 0
    {
        return Err(AppError::InvalidData(
            "FSRS snapshot contains a negative value".into(),
        ));
    }

    let _guard = state
        .write_lock
        .lock()
        .map_err(|_| AppError::Resource("database write lock is poisoned".into()))?;
    let mut connection = state.connect()?;
    let transaction = connection.transaction()?;
    let old_total_xp: i64 = transaction.query_row(
        "SELECT total_xp FROM user_progress WHERE id = 1",
        [],
        |row| row.get(0),
    )?;
    let old_level = level_from_xp(old_total_xp, 0).level;
    let review_day = reviewed.date_naive().to_string();
    let prior_same_day: i64 = transaction.query_row(
        "SELECT COUNT(*) FROM review_logs WHERE card_id = ?1 AND review_day = ?2",
        params![card_id, review_day],
        |row| row.get(0),
    )?;
    let card_exists: bool = transaction.query_row(
        "SELECT EXISTS(SELECT 1 FROM study_cards WHERE id = ?1)",
        [card_id],
        |row| row.get(0),
    )?;
    if !card_exists {
        return Err(AppError::InvalidData(format!("unknown card id {card_id}")));
    }

    let xp_awarded = award_for_review(rating, prior_same_day, false);
    let status = match fsrs.state {
        0 => "new",
        1 => "learning",
        2 => "review",
        3 => "relearning",
        _ => {
            return Err(AppError::InvalidData(format!(
                "unsupported FSRS card state {}",
                fsrs.state
            )))
        }
    };
    let snapshot_json = serde_json::to_string(fsrs)?;
    let now = Utc::now().to_rfc3339();
    transaction.execute(
        "UPDATE study_cards SET status = ?2, due_at = ?3, stability = ?4, difficulty = ?5,
          elapsed_days = ?6, scheduled_days = ?7, learning_steps = ?8, reps = ?9, lapses = ?10,
          fsrs_state = ?11, last_review_at = ?12, scheduler_snapshot = ?13, updated_at = ?14 WHERE id = ?1",
        params![card_id, status, fsrs.due, fsrs.stability, fsrs.difficulty, fsrs.elapsed_days,
            fsrs.scheduled_days, fsrs.learning_steps, fsrs.reps, fsrs.lapses, fsrs.state,
            reviewed_at, snapshot_json, now],
    )?;
    transaction.execute("DELETE FROM relearn_queue WHERE card_id = ?1", [card_id])?;
    transaction.execute(
        "INSERT INTO review_logs (id, card_id, rating, reviewed_at, review_day, xp_awarded, scheduler_snapshot, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![Uuid::new_v4().to_string(), card_id, rating.as_str(), reviewed_at, review_day,
            xp_awarded, snapshot_json, now],
    )?;
    transaction.execute(
        "INSERT INTO learning_records (id, card_id, event_type, occurred_at, payload_json)
         VALUES (?1, ?2, 'review', ?3, ?4)",
        params![
            Uuid::new_v4().to_string(),
            card_id,
            reviewed.to_rfc3339(),
            serde_json::json!({
                "rating": rating.as_str(), "xpAwarded": xp_awarded, "fsrsCard": fsrs
            })
            .to_string()
        ],
    )?;
    transaction.execute(
        "INSERT INTO daily_stats (study_day, completed, xp_earned, new_cards, review_cards, updated_at)
         VALUES (?1, 1, ?2, ?3, ?4, ?5)
         ON CONFLICT(study_day) DO UPDATE SET completed = completed + 1, xp_earned = xp_earned + excluded.xp_earned,
           new_cards = new_cards + excluded.new_cards, review_cards = review_cards + excluded.review_cards,
           updated_at = excluded.updated_at",
        params![review_day, xp_awarded, i64::from(fsrs.reps <= 1), i64::from(fsrs.reps > 1), now],
    )?;

    let (streak_days, last_study_day) = update_streak(&transaction, reviewed.date_naive())?;
    let new_total_xp = old_total_xp + xp_awarded;
    let progress = level_from_xp(new_total_xp, streak_days);
    transaction.execute(
        "UPDATE user_progress SET total_xp = ?1, level = ?2, streak_days = ?3, last_study_day = ?4, updated_at = ?5 WHERE id = 1",
        params![progress.xp, progress.level, progress.streak_days, last_study_day, now],
    )?;
    transaction.commit()?;

    let card = get_card(&connection, card_id)?;
    let settings = read_settings(&connection)?;
    let daily = read_daily(&connection, settings.daily_quota)?;
    let next_card = get_next_card_with_connection(&connection, Some(card_id))?;
    Ok(ReviewOutcome {
        card,
        progress: progress.clone(),
        daily,
        xp_awarded,
        leveled_up: progress.level > old_level,
        feedback: rating.as_str().to_string(),
        next_card,
    })
}

fn update_streak(transaction: &Transaction<'_>, day: NaiveDate) -> AppResult<(i64, String)> {
    let (current, last): (i64, Option<String>) = transaction.query_row(
        "SELECT streak_days, last_study_day FROM user_progress WHERE id = 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let streak = match last
        .as_deref()
        .and_then(|value| NaiveDate::parse_from_str(value, "%Y-%m-%d").ok())
    {
        Some(previous) if previous == day => current.max(1),
        Some(previous) if previous + Duration::days(1) == day => current + 1,
        _ => 1,
    };
    Ok((streak, day.to_string()))
}

pub fn list_library(state: &DbState, query: &LibraryQuery) -> AppResult<LibraryPayload> {
    let connection = state.connect()?;
    let search = format!("%{}%", escape_like(query.search.trim()));
    let cutoff = (Utc::now() - Duration::days(7)).to_rfc3339();
    let filter = match query.status.as_str() {
        "all" => "1 = 1",
        "new" | "learning" | "review" | "relearning" => "c.status = ?2",
        "favorite" => "EXISTS(SELECT 1 FROM favorites ff WHERE ff.card_id = c.id)",
        "recent" => "c.last_review_at IS NOT NULL AND c.last_review_at >= ?2",
        other => {
            return Err(AppError::InvalidData(format!(
                "invalid library status {other}"
            )))
        }
    };
    let limit = query.limit.clamp(1, 100);
    let offset = query.offset;
    let sql = format!(
        "{CARD_SELECT} WHERE (l.word LIKE ?1 ESCAPE '\\' OR l.definition_zh LIKE ?1 ESCAPE '\\') AND {filter}
         ORDER BY COALESCE(c.last_review_at, '') DESC, l.frequency_rank ASC LIMIT {limit} OFFSET {offset}"
    );
    let mut statement = connection.prepare(&sql)?;
    let items = match query.status.as_str() {
        "new" | "learning" | "review" | "relearning" => statement
            .query_map(params![search, query.status], card_from_row)?
            .collect::<Result<Vec<_>, _>>()?,
        "recent" => statement
            .query_map(params![search, cutoff], card_from_row)?
            .collect::<Result<Vec<_>, _>>()?,
        _ => statement
            .query_map([search.clone()], card_from_row)?
            .collect::<Result<Vec<_>, _>>()?,
    };

    let mut counts = BTreeMap::new();
    counts.insert("all".into(), count_where(&connection, "1 = 1", None)?);
    for status in ["new", "learning", "review", "relearning"] {
        counts.insert(
            status.into(),
            count_where(&connection, "c.status = ?1", Some(status))?,
        );
    }
    counts.insert(
        "favorite".into(),
        count_where(
            &connection,
            "EXISTS(SELECT 1 FROM favorites f WHERE f.card_id = c.id)",
            None,
        )?,
    );
    counts.insert(
        "recent".into(),
        connection.query_row(
            "SELECT COUNT(*) FROM study_cards c WHERE c.last_review_at >= ?1",
            [cutoff.clone()],
            |row| row.get(0),
        )?,
    );
    let matching_count = match query.status.as_str() {
        "new" | "learning" | "review" | "relearning" => connection.query_row(
            &format!("SELECT COUNT(*) FROM study_cards c JOIN lexemes l ON l.id = c.lexeme_id WHERE (l.word LIKE ?1 ESCAPE '\\' OR l.definition_zh LIKE ?1 ESCAPE '\\') AND {filter}"),
            params![search, query.status],
            |row| row.get::<_, i64>(0),
        )?,
        "recent" => connection.query_row(
            &format!("SELECT COUNT(*) FROM study_cards c JOIN lexemes l ON l.id = c.lexeme_id WHERE (l.word LIKE ?1 ESCAPE '\\' OR l.definition_zh LIKE ?1 ESCAPE '\\') AND {filter}"),
            params![search, cutoff],
            |row| row.get::<_, i64>(0),
        )?,
        _ => connection.query_row(
            &format!("SELECT COUNT(*) FROM study_cards c JOIN lexemes l ON l.id = c.lexeme_id WHERE (l.word LIKE ?1 ESCAPE '\\' OR l.definition_zh LIKE ?1 ESCAPE '\\') AND {filter}"),
            params![search],
            |row| row.get::<_, i64>(0),
        )?,
    };
    Ok(LibraryPayload {
        items,
        counts,
        has_more: i64::from(offset) + i64::from(limit) < matching_count,
    })
}

fn count_where(connection: &Connection, clause: &str, value: Option<&str>) -> AppResult<i64> {
    let sql = format!("SELECT COUNT(*) FROM study_cards c WHERE {clause}");
    Ok(if let Some(value) = value {
        connection.query_row(&sql, [value], |row| row.get(0))?
    } else {
        connection.query_row(&sql, [], |row| row.get(0))?
    })
}

fn escape_like(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

pub fn toggle_favorite(state: &DbState, card_id: &str) -> AppResult<LearningCard> {
    let _guard = state
        .write_lock
        .lock()
        .map_err(|_| AppError::Resource("database write lock is poisoned".into()))?;
    let connection = state.connect()?;
    if connection.execute("DELETE FROM favorites WHERE card_id = ?1", [card_id])? == 0 {
        connection.execute(
            "INSERT INTO favorites (card_id, created_at) VALUES (?1, ?2)",
            params![card_id, Utc::now().to_rfc3339()],
        )?;
    }
    get_card(&connection, card_id)
}

pub fn queue_relearn(state: &DbState, card_id: &str) -> AppResult<LearningCard> {
    let _guard = state
        .write_lock
        .lock()
        .map_err(|_| AppError::Resource("database write lock is poisoned".into()))?;
    let connection = state.connect()?;
    let now = Utc::now().to_rfc3339();
    let changed = connection.execute(
        "UPDATE study_cards SET status = 'relearning', due_at = ?2, updated_at = ?2 WHERE id = ?1",
        params![card_id, now],
    )?;
    if changed == 0 {
        return Err(AppError::InvalidData(format!("unknown card id {card_id}")));
    }
    connection.execute(
        "INSERT INTO relearn_queue (card_id, queued_at) VALUES (?1, ?2)
         ON CONFLICT(card_id) DO UPDATE SET queued_at = excluded.queued_at",
        params![card_id, now],
    )?;
    get_card(&connection, card_id)
}

pub fn update_settings(state: &DbState, settings: &AppSettings) -> AppResult<AppSettings> {
    validate_settings(settings)?;
    let _guard = state
        .write_lock
        .lock()
        .map_err(|_| AppError::Resource("database write lock is poisoned".into()))?;
    let connection = state.connect()?;
    let pack_exists: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM language_packs WHERE id = ?1)",
        [&settings.language_pack_id],
        |row| row.get(0),
    )?;
    if !pack_exists {
        return Err(AppError::InvalidData(format!(
            "unknown language pack {}",
            settings.language_pack_id
        )));
    }
    connection.execute(
        "UPDATE settings SET reminder_interval_minutes = ?1, daily_quota = ?2, quiet_hours_enabled = ?3,
          quiet_start = ?4, quiet_end = ?5, autostart = ?6, always_on_top = ?7, language_pack_id = ?8,
          reminders_paused = ?9, quiet_mode_until = ?10, speech_enabled = ?11, pet_appearance_id = ?12,
          updated_at = ?13 WHERE id = 1",
        params![settings.reminder_interval_minutes, settings.daily_quota, settings.quiet_hours_enabled as i64,
            settings.quiet_start, settings.quiet_end, settings.autostart as i64, settings.always_on_top as i64,
            settings.language_pack_id, settings.reminders_paused as i64, settings.quiet_mode_until,
            settings.speech_enabled as i64, settings.pet_appearance_id,
            Utc::now().to_rfc3339()],
    )?;
    connection.execute(
        "UPDATE learner_profile SET active_pack_id = ?1, updated_at = ?2 WHERE id = 1",
        params![settings.language_pack_id, Utc::now().to_rfc3339()],
    )?;
    read_settings(&connection)
}

fn validate_settings(settings: &AppSettings) -> AppResult<()> {
    if !(5..=240).contains(&settings.reminder_interval_minutes) {
        return Err(AppError::InvalidData(
            "reminderIntervalMinutes must be between 5 and 240".into(),
        ));
    }
    if !(1..=200).contains(&settings.daily_quota) {
        return Err(AppError::InvalidData(
            "dailyQuota must be between 1 and 200".into(),
        ));
    }
    for (name, value) in [
        ("quietStart", settings.quiet_start.as_str()),
        ("quietEnd", settings.quiet_end.as_str()),
    ] {
        chrono::NaiveTime::parse_from_str(value, "%H:%M")
            .map_err(|_| AppError::InvalidData(format!("{name} must use HH:mm")))?;
    }
    if let Some(until) = &settings.quiet_mode_until {
        DateTime::parse_from_rfc3339(until)
            .map_err(|_| AppError::InvalidData("quietModeUntil must be RFC 3339".into()))?;
    }
    Ok(())
}

pub fn set_boolean_setting(state: &DbState, column: &str, enabled: bool) -> AppResult<bool> {
    let allowed = ["always_on_top", "reminders_paused"];
    if !allowed.contains(&column) {
        return Err(AppError::InvalidData("setting is not mutable".into()));
    }
    let _guard = state
        .write_lock
        .lock()
        .map_err(|_| AppError::Resource("database write lock is poisoned".into()))?;
    let connection = state.connect()?;
    connection.execute(
        &format!("UPDATE settings SET {column} = ?1, updated_at = ?2 WHERE id = 1"),
        params![enabled as i64, Utc::now().to_rfc3339()],
    )?;
    Ok(enabled)
}

pub fn set_quiet_until(state: &DbState, until: Option<&str>) -> AppResult<Option<String>> {
    if let Some(value) = until {
        DateTime::parse_from_rfc3339(value)
            .map_err(|_| AppError::InvalidData("until must be RFC 3339 or null".into()))?;
    }
    let _guard = state
        .write_lock
        .lock()
        .map_err(|_| AppError::Resource("database write lock is poisoned".into()))?;
    let connection = state.connect()?;
    connection.execute(
        "UPDATE settings SET quiet_mode_until = ?1, updated_at = ?2 WHERE id = 1",
        params![until, Utc::now().to_rfc3339()],
    )?;
    Ok(until.map(ToOwned::to_owned))
}

pub fn complete_placement(
    state: &DbState,
    submission: &PlacementSubmission,
) -> AppResult<BootstrapPayload> {
    let _guard = state
        .write_lock
        .lock()
        .map_err(|_| AppError::Resource("database write lock is poisoned".into()))?;
    let mut connection = state.connect()?;
    let transaction = connection.transaction()?;
    let total = submission.answers.len() as f64;
    let correct = submission
        .answers
        .iter()
        .filter(|answer| answer.correct)
        .count() as f64;
    let ratio = if total == 0.0 { 0.5 } else { correct / total };
    let adjustment = if submission.answers.len() < 4 {
        0
    } else if ratio >= 0.8 {
        1
    } else if ratio < 0.5 {
        -1
    } else {
        0
    };
    let recommended = LevelBand::from_index(submission.self_reported_tier.index() + adjustment);
    let confidence = if total == 0.0 {
        0.35
    } else {
        (0.45 + total.min(24.0) / 48.0).min(0.95)
    };
    let now = Utc::now().to_rfc3339();
    transaction.execute(
        "INSERT INTO placement_attempts (id, started_at, completed_at, question_count, highest_stable_tier,
          recommended_tier, confidence, answers_json) VALUES (?1, ?2, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![Uuid::new_v4().to_string(), now, submission.answers.len() as i64,
            submission.self_reported_tier.as_str(), recommended.as_str(), confidence,
            serde_json::to_string(&submission.answers)?],
    )?;
    transaction.execute(
        "UPDATE learner_profile SET self_reported_tier = ?1, placed_tier = ?2, placement_confidence = ?3,
          current_comfort_tier = ?2, placement_complete = 1, updated_at = ?4 WHERE id = 1",
        params![submission.self_reported_tier.as_str(), recommended.as_str(), confidence, now],
    )?;
    transaction.commit()?;
    bootstrap_with_connection(&connection)
}

pub fn read_settings(connection: &Connection) -> AppResult<AppSettings> {
    Ok(connection.query_row(
        "SELECT reminder_interval_minutes, daily_quota, quiet_hours_enabled, quiet_start, quiet_end,
                autostart, always_on_top, language_pack_id, reminders_paused, quiet_mode_until,
                speech_enabled, pet_appearance_id
         FROM settings WHERE id = 1",
        [],
        |row| {
            Ok(AppSettings {
                reminder_interval_minutes: row.get(0)?,
                daily_quota: row.get(1)?,
                quiet_hours_enabled: row.get::<_, i64>(2)? != 0,
                quiet_start: row.get(3)?,
                quiet_end: row.get(4)?,
                autostart: row.get::<_, i64>(5)? != 0,
                always_on_top: row.get::<_, i64>(6)? != 0,
                language_pack_id: row.get(7)?,
                reminders_paused: row.get::<_, i64>(8)? != 0,
                quiet_mode_until: row.get(9)?,
                speech_enabled: row.get::<_, i64>(10)? != 0,
                pet_appearance_id: row.get(11)?,
            })
        },
    )?)
}

pub fn read_progress(connection: &Connection) -> AppResult<PlayerProgress> {
    let (xp, streak): (i64, i64) = connection.query_row(
        "SELECT total_xp, streak_days FROM user_progress WHERE id = 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    Ok(level_from_xp(xp, streak))
}

pub fn read_daily(connection: &Connection, goal: i64) -> AppResult<DailyProgress> {
    let today = Utc::now().date_naive().to_string();
    let completed: i64 = connection
        .query_row(
            "SELECT completed FROM daily_stats WHERE study_day = ?1",
            [&today],
            |row| row.get(0),
        )
        .optional()?
        .unwrap_or(0);
    let reviews_due: i64 = connection.query_row(
        "SELECT COUNT(*) FROM study_cards WHERE status != 'new' AND due_at IS NOT NULL AND due_at <= ?1",
        [Utc::now().to_rfc3339()],
        |row| row.get(0),
    )?;
    Ok(DailyProgress {
        completed,
        goal,
        reviews_due,
    })
}

pub fn get_card(connection: &Connection, card_id: &str) -> AppResult<LearningCard> {
    let sql = format!("{CARD_SELECT} WHERE c.id = ?1");
    connection
        .query_row(&sql, [card_id], card_from_row)
        .optional()?
        .ok_or_else(|| AppError::InvalidData(format!("unknown card id {card_id}")))
}

fn card_from_row(row: &Row<'_>) -> rusqlite::Result<LearningCard> {
    let tier: String = row.get(7)?;
    let fsrs_state: Option<i64> = row.get(20)?;
    let fsrs = if let Some(state) = fsrs_state {
        Some(FsrsCardSnapshot {
            due: row.get::<_, Option<String>>(12)?.unwrap_or_default(),
            stability: row.get::<_, Option<f64>>(13)?.unwrap_or_default(),
            difficulty: row.get::<_, Option<f64>>(14)?.unwrap_or_default(),
            elapsed_days: row.get::<_, Option<i64>>(15)?.unwrap_or_default(),
            scheduled_days: row.get::<_, Option<i64>>(16)?.unwrap_or_default(),
            learning_steps: row.get::<_, Option<i64>>(17)?.unwrap_or_default(),
            reps: row.get(18)?,
            lapses: row.get(19)?,
            state,
            last_review: row.get(11)?,
        })
    } else {
        None
    };
    Ok(LearningCard {
        id: row.get(0)?,
        word: row.get(1)?,
        phonetic: row.get(2)?,
        part_of_speech: row.get(3)?,
        definition_zh: row.get(4)?,
        example_en: row.get(5)?,
        example_zh: row.get(6)?,
        tier: LevelBand::try_from(tier.as_str()).unwrap_or(LevelBand::T1),
        frequency_rank: row.get(8)?,
        status: row.get(9)?,
        favorite: row.get::<_, i64>(10)? != 0,
        source_pack_id: row.get(21)?,
        source_pack_name: row.get(22)?,
        last_studied_at: row.get(11)?,
        next_review_at: row.get(12)?,
        fsrs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::DbState;

    fn test_state() -> (tempfile::TempDir, DbState) {
        let directory = tempfile::tempdir().unwrap();
        let state = DbState::initialize(directory.path().join("test.sqlite")).unwrap();
        (directory, state)
    }

    fn insert_candidate(
        connection: &Connection,
        pack_id: &str,
        pack_name: &str,
        lexeme_id: &str,
        word: &str,
        tier: &str,
        frequency_rank: i64,
    ) {
        connection
            .execute(
                "INSERT INTO language_packs (id, name, source_language, target_language, explanation_language, pack_version, compatibility_version, record_count, pack_license, imported_at)
                 VALUES (?1, ?2, 'zh-CN', 'en', 'zh-CN', 'test', '2', 1, 'CC-BY-SA-4.0', '2026-08-19T00:00:00Z')",
                params![pack_id, pack_name],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO lexemes (id, pack_id, word, phonetic, part_of_speech, definition_zh, example_en, example_zh, tier, frequency_rank, record_license, source_id, source_url, provenance_json, learning_ready)
                 VALUES (?1, ?2, ?3, '', 'n.', '回声', '', '', ?4, ?5, 'CC-BY-SA-4.0', ?1, 'offline', '{}', 1)",
                params![lexeme_id, pack_id, word, tier, frequency_rank],
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO study_cards (id, lexeme_id, created_at, updated_at)
                 VALUES (?1, ?2, '2026-08-19T00:00:00Z', '2026-08-19T00:00:00Z')",
                params![format!("card:{lexeme_id}"), lexeme_id],
            )
            .unwrap();
    }

    #[test]
    fn composite_scope_deduplicates_and_prefers_easier_higher_frequency_record() {
        let (_directory, state) = test_state();
        let connection = state.connect().unwrap();
        insert_candidate(
            &connection,
            "zh-CN__en.vocabularybook.hard",
            "挑战词书",
            "duplicate-hard",
            "Echo",
            "T2",
            1,
        );
        insert_candidate(
            &connection,
            "zh-CN__en.vocabularybook.easy-slow",
            "基础词书 A",
            "duplicate-easy-slow",
            " echo ",
            "T1",
            200,
        );
        insert_candidate(
            &connection,
            "zh-CN__en.vocabularybook.easy-fast",
            "基础词书 B",
            "duplicate-easy-fast",
            "ECHO",
            "T1",
            50,
        );
        connection
            .execute(
                "UPDATE learner_profile SET active_pack_id = ?1, current_comfort_tier = 'T2' WHERE id = 1",
                [ALL_LANGUAGE_PACK_ID],
            )
            .unwrap();

        let card = get_next_card_with_connection(&connection, None)
            .unwrap()
            .unwrap();

        assert_eq!(card.id, "card:duplicate-easy-fast");
        assert_eq!(card.tier, LevelBand::T1);
        assert_eq!(card.frequency_rank, 50);
        assert_eq!(card.source_pack_name.as_deref(), Some("基础词书 B"));
    }

    #[test]
    fn review_transaction_persists_fsrs_and_xp() {
        let (_directory, state) = test_state();
        let connection = state.connect().unwrap();
        connection
            .execute("UPDATE learner_profile SET placement_complete = 1", [])
            .unwrap();
        let card = get_next_card(&state, None).unwrap().unwrap();
        let reviewed_at = "2026-08-18T08:00:00Z";
        let fsrs = FsrsCardSnapshot {
            due: "2026-08-19T08:00:00Z".into(),
            stability: 1.2,
            difficulty: 5.0,
            elapsed_days: 0,
            scheduled_days: 1,
            learning_steps: 0,
            reps: 1,
            lapses: 0,
            state: 2,
            last_review: Some(reviewed_at.into()),
        };
        let outcome =
            submit_review(&state, &card.id, ReviewRating::Good, reviewed_at, &fsrs).unwrap();
        assert_eq!(outcome.xp_awarded, 7);
        assert_eq!(outcome.card.fsrs, Some(fsrs));
        drop(connection);
        let reopened = DbState::initialize(state.path().to_path_buf()).unwrap();
        assert_eq!(read_progress(&reopened.connect().unwrap()).unwrap().xp, 7);
    }

    #[test]
    fn every_fifth_completed_review_uses_challenge_tier() {
        let (_directory, state) = test_state();
        let connection = state.connect().unwrap();
        connection
            .execute("UPDATE learner_profile SET current_comfort_tier = 'T2'", [])
            .unwrap();
        let seed_card_id: String = connection
            .query_row(
                "SELECT id FROM study_cards ORDER BY id LIMIT 1",
                [],
                |row| row.get(0),
            )
            .unwrap();
        for index in 0..4 {
            connection
                .execute(
                    "INSERT INTO review_logs (id, card_id, rating, reviewed_at, review_day, xp_awarded, scheduler_snapshot, created_at)
                     VALUES (?1, ?2, 'good', '2026-08-18T08:00:00Z', '2026-08-18', 0, '{}', '2026-08-18T08:00:00Z')",
                    params![format!("seed-{index}"), seed_card_id],
                )
                .unwrap();
        }
        let card = get_next_card_with_connection(&connection, None)
            .unwrap()
            .unwrap();
        assert_eq!(card.tier, LevelBand::T3);
    }

    #[test]
    fn second_easy_review_same_day_awards_zero() {
        let (_directory, state) = test_state();
        let card = get_next_card(&state, None).unwrap().unwrap();
        let fsrs = FsrsCardSnapshot {
            due: "2026-08-19T08:00:00Z".into(),
            stability: 1.0,
            difficulty: 1.0,
            elapsed_days: 0,
            scheduled_days: 1,
            learning_steps: 0,
            reps: 1,
            lapses: 0,
            state: 2,
            last_review: Some("2026-08-18T08:00:00Z".into()),
        };
        submit_review(
            &state,
            &card.id,
            ReviewRating::Easy,
            "2026-08-18T08:00:00Z",
            &fsrs,
        )
        .unwrap();
        let second = submit_review(
            &state,
            &card.id,
            ReviewRating::Easy,
            "2026-08-18T09:00:00Z",
            &fsrs,
        )
        .unwrap();
        assert_eq!(second.xp_awarded, 0);
    }
}
