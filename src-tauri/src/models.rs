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
    pub speech_enabled: bool,
    pub pet_appearance_id: String,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PetAssetState {
    Default,
    Idle,
    Click,
    Error,
    Correct,
    Upgrade,
}

impl PetAssetState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Idle => "idle",
            Self::Click => "click",
            Self::Error => "error",
            Self::Correct => "correct",
            Self::Upgrade => "upgrade",
        }
    }
}

impl TryFrom<&str> for PetAssetState {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, String> {
        match value {
            "default" => Ok(Self::Default),
            "idle" => Ok(Self::Idle),
            "click" => Ok(Self::Click),
            "error" => Ok(Self::Error),
            "correct" => Ok(Self::Correct),
            "upgrade" => Ok(Self::Upgrade),
            _ => Err(format!("invalid pet asset state: {value}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PetAsset {
    pub id: String,
    pub appearance_id: String,
    pub state: PetAssetState,
    pub mime_type: String,
    pub width: u32,
    pub height: u32,
    pub data_url: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PetAppearanceSummary {
    pub id: String,
    pub bundled: bool,
    pub selected: bool,
    pub available_states: Vec<PetAssetState>,
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
    #[serde(deserialize_with = "deserialize_part_of_speech")]
    pub part_of_speech: String,
    pub definition_zh: String,
    #[serde(default)]
    pub example_en: Option<String>,
    #[serde(default)]
    pub example_zh: Option<String>,
    pub tier: LevelBand,
    #[serde(default)]
    pub frequency_rank: Option<i64>,
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

fn deserialize_part_of_speech<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::String(text) => Ok(text),
        serde_json::Value::Array(values) => Ok(values
            .into_iter()
            .filter_map(|value| value.as_str().map(str::to_owned))
            .collect::<Vec<_>>()
            .join(" ")),
        serde_json::Value::Null => Ok(String::new()),
        _ => Err(serde::de::Error::custom(
            "partOfSpeech must be a string or array",
        )),
    }
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
