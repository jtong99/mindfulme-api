pub const EMOTION_JOY: &str = "joy";
pub const EMOTION_SADNESS: &str = "sadness";
pub const EMOTION_ANGER: &str = "anger";
pub const EMOTION_FEAR: &str = "fear";
pub const EMOTION_DISGUST: &str = "disgust";
pub const EMOTION_SURPRISE: &str = "surprise";

pub fn valid_emotions() -> Vec<&'static str> {
    vec![
        EMOTION_JOY,
        EMOTION_SADNESS,
        EMOTION_ANGER,
        EMOTION_FEAR,
        EMOTION_DISGUST,
        EMOTION_SURPRISE,
    ]
}

use bson::serde_helpers::bson_datetime_as_rfc3339_string;
use bson::serde_helpers::serialize_object_id_as_hex_string;
use serde::{Deserialize, Serialize};
use validator::Validate;
use wither::bson::{doc, oid::ObjectId};
use wither::Model as WitherModel;

use crate::utils::date;
use crate::utils::date::Date;
use crate::utils::models::ModelExt;

impl ModelExt for Checkin {}

#[derive(Debug, Clone, Serialize, Deserialize, WitherModel, Validate)]
#[model(index(keys = r#"doc!{ "user": 1, "created_at": 1 }"#))]
pub struct Checkin {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub user: ObjectId,
    
    // Core mood data
    #[validate(range(min = 1, max = 5))]
    pub mood_rating: u8,
    pub primary_emotion: String,
    #[validate(range(min = 1, max = 5))]
    pub intensity: u8,
    
    // Well-being metrics
    #[validate(range(min = 1, max = 5))]
    pub energy_level: u8,
    #[validate(range(min = 1, max = 5))]
    pub stress_level: u8,
    #[validate(range(min = 1, max = 5))]
    pub wellbeing: u8,
    
    // Note/journal field - optional
    pub notes: Option<String>,
    
    // Timestamps
    pub updated_at: Date,
    pub created_at: Date,
}

impl Checkin {
    pub fn new(
        user: ObjectId, 
        mood_rating: u8,
        primary_emotion: String,
        intensity: u8,
        energy_level: u8,
        stress_level: u8,
        wellbeing: u8,
        notes: Option<String>,
    ) -> Self {
        let now = date::now();
        Self {
            id: None,
            user,
            mood_rating,
            primary_emotion,
            intensity,
            energy_level,
            stress_level,
            wellbeing,
            notes,
            updated_at: now,
            created_at: now,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublicCheckin {
    #[serde(alias = "_id", serialize_with = "serialize_object_id_as_hex_string")]
    pub id: ObjectId,
    #[serde(serialize_with = "serialize_object_id_as_hex_string")]
    pub user: ObjectId,
    pub mood_rating: u8,
    pub primary_emotion: String,
    pub intensity: u8,
    pub energy_level: u8,
    pub stress_level: u8,
    pub wellbeing: u8,
    pub notes: Option<String>,
    #[serde(with = "bson_datetime_as_rfc3339_string")]
    pub updated_at: Date,
    #[serde(with = "bson_datetime_as_rfc3339_string")]
    pub created_at: Date,
}

impl From<Checkin> for PublicCheckin {
    fn from(checkin: Checkin) -> Self {
        Self {
            id: checkin.id.unwrap(),
            user: checkin.user,
            mood_rating: checkin.mood_rating,
            primary_emotion: checkin.primary_emotion,
            intensity: checkin.intensity,
            energy_level: checkin.energy_level,
            stress_level: checkin.stress_level,
            wellbeing: checkin.wellbeing,
            notes: checkin.notes,
            updated_at: checkin.updated_at,
            created_at: checkin.created_at,
        }
    }
}