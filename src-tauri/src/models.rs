use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LevelBand {
    T1,
    T2,
    T3,
    T4,
    T5,
}

impl LevelBand {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::T1 => "T1",
            Self::T2 => "T2",
            Self::T3 => "T3",
            Self::T4 => "T4",
            Self::T5 => "T5",
        }
    }

    pub fn index(self) -> i64 {
        match self {
            Self::T1 => 1,
            Self::T2 => 2,
            Self::T3 => 3,
            Self::T4 => 4,
            Self::T5 => 5,
        }
    }

    pub fn from_index(value: i64) -> Self {
        match value.clamp(1, 5) {
            1 => Self::T1,
            2 => Self::T2,
            3 => Self::T3,
            4 => Self::T4,
            _ => Self::T5,
        }
    }
}

impl TryFrom<&str> for LevelBand {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "T1" => Ok(Self::T1),
            "T2" => Ok(Self::T2),
            "T3" => Ok(Self::T3),
            "T4" => Ok(Self::T4),
            "T5" => Ok(Self::T5),
            _ => Err(format!("invalid learning tier: {value}")),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ReviewRating {
    Again,
    Hard,
    Good,
    Easy,
}

impl ReviewRating {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Again => "again",
            Self::Hard => "hard",
            Self::Good => "good",
            Self::Easy => "easy",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FsrsCardSnapshot {
    pub due: String,
    pub stability: f64,
    pub difficulty: f64,
    pub elapsed_days: i64,
    pub scheduled_days: i64,
    pub learning_steps: i64,
    pub reps: i64,
    pub lapses: i64,
    pub state: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_review: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LearningCard {
    pub id: String,
    pub word: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phonetic: Option<String>,
    pub part_of_speech: String,
    pub definition_zh: String,
    pub example_en: String,
    pub example_zh: String,
    pub tier: LevelBand,
    pub frequency_rank: i64,
    pub status: String,
    pub favorite: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_studied_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_review_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fsrs: Option<FsrsCardSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DailyProgress {
    pub completed: i64,
    pub goal: i64,
    pub reviews_due: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PlayerProgress {
    pub xp: i64,
    pub level: i64,
    pub xp_into_level: i64,
    pub xp_for_next_level: i64,
    pub streak_days: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LanguagePackSummary {
    pub id: String,
    pub name: String,
    pub source_language: String,
    pub target_language: String,
    pub explanation_language: String,
    pub pack_version: String,
    pub compatibility_version: String,
    pub record_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub reminder_interval_minutes: i64,
    pub daily_quota: i64,
    pub quiet_hours_enabled: bool,
    pub quiet_start: String,
    pub quiet_end: String,
    pub autostart: bool,
    pub always_on_top: bool,
    pub language_pack_id: String,
    pub reminders_paused: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quiet_mode_until: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapPayload {
    pub progress: PlayerProgress,
    pub daily: DailyProgress,
    pub settings: AppSettings,
    pub language_packs: Vec<LanguagePackSummary>,
    pub placement_complete: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placement_tier: Option<LevelBand>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_card: Option<LearningCard>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReviewOutcome {
    pub card: LearningCard,
    pub progress: PlayerProgress,
    pub daily: DailyProgress,
    pub xp_awarded: i64,
    pub leveled_up: bool,
    pub feedback: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_card: Option<LearningCard>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LibraryQuery {
    pub search: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LibraryPayload {
    pub items: Vec<LearningCard>,
    pub counts: BTreeMap<String, i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PlacementAnswer {
    pub question_id: String,
    pub correct: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PlacementSubmission {
    pub self_reported_tier: LevelBand,
    #[serde(default)]
    pub answers: Vec<PlacementAnswer>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LicenseDocument {
    pub id: String,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    pub path: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguagePack {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub source_language: String,
    pub target_language: String,
    pub explanation_language: String,
    pub pack_version: String,
    pub compatibility_version: String,
    pub pack_license: String,
    pub records: Vec<LanguageRecord>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageRecord {
    pub id: String,
    pub word: String,
    #[serde(default)]
    pub phonetic: Option<String>,
    pub part_of_speech: String,
    pub definition_zh: String,
    pub example_en: String,
    pub example_zh: String,
    pub tier: LevelBand,
    pub frequency_rank: i64,
    pub license: String,
    pub source_id: String,
    pub source_url: String,
    pub provenance: Value,
    #[serde(default = "default_learning_ready")]
    pub learning_ready: bool,
}

fn default_learning_ready() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupDocument {
    pub schema_version: u32,
    pub product: String,
    pub exported_at: String,
    pub database_user_version: i64,
    pub tables: BTreeMap<String, Vec<Value>>,
}
