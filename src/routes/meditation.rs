use axum::{
    extract::{Json, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use std::{env, io::Write, path::PathBuf, fs::File};
use tracing::{debug, info, error};
use uuid::Uuid;

use crate::errors::Error;
use crate::utils::token::TokenUser;
use axum::extract::Path;
use axum::routing::get;

// Request for music generation
#[derive(Debug, Deserialize)]
pub struct GenerateMusicRequest {
    // Duration in minutes
    duration: u32,
    // Type of meditation (mindfulness, breath, body_scan, etc.)
    meditation_type: String,
    // Music style (nature, ambient, piano, binaural, bowls, minimal)
    music_atmosphere: String,
    // What user wants to focus on (anxiety, sleep, focus, gratitude, etc.)
    focus_area: String,
    // Background setting (forest, beach, mountain, garden, space)
    background: String,
}

// Response with the music file path
#[derive(Debug, Serialize)]
pub struct GenerateMusicResponse {
    music_url: String,
}

// Application state
#[derive(Clone)]
pub struct AppState {
    hf_token: String,
    music_dir: PathBuf,
}

pub fn create_route() -> Router {
    // Create music directory if it doesn't exist
    let music_dir = PathBuf::from("./meditation_music");
    std::fs::create_dir_all(&music_dir).expect("Failed to create music directory");
    
    // Get HuggingFace token from environment
    let hf_token = env::var("HUGGINGFACE_API_TOKEN")
        .unwrap_or_else(|_| "DEFAULT_TOKEN_REPLACE_ME".to_string());
    
    let state = AppState {
        hf_token,
        music_dir: music_dir.clone(),
    };

    Router::new()
        .route("/api/meditation/generate-music", post(generate_music))
        // Add the route for serving audio files directly here
        .route("/api/meditation/music/:filename", get(serve_audio_file))
        .with_state(state)
}

async fn generate_music(
    user: TokenUser, // Add user authentication
    State(state): State<AppState>,
    Json(payload): Json<GenerateMusicRequest>,
) -> Result<Json<GenerateMusicResponse>, Error> {
    // Generate music prompt based on preferences
    let prompt = generate_music_prompt(&payload);
    debug!("Generated music prompt: {}", prompt);

    // Generate a unique filename
    let filename = format!("{}.mp3", Uuid::new_v4());
    let file_path = state.music_dir.join(&filename);
    
    // Prepare request to HuggingFace API
    let api_url = "https://router.huggingface.co/hf-inference/models/facebook/musicgen-small";
    let api_payload = serde_json::json!({
        "inputs": prompt,
    });

    info!("Calling HuggingFace API to generate music");
    
    // Build the request manually without using reqwest
    let client = reqwest::Client::new();
    let response = client
        .post(api_url)
        .header("Authorization", format!("Bearer {}", state.hf_token))
        .json(&api_payload)
        .send()
        .await
        .map_err(|e| Error::bad_request_with_message(format!("API request failed: {}", e)))?;

    // Check if the request was successful
    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(Error::bad_request_with_message(format!("API returned error: {}", error_text)));
    }

    // Get the response bytes (audio file)
    let audio_bytes = response.bytes().await
        .map_err(|e| Error::bad_request_with_message(format!("Failed to get response bytes: {}", e)))?;

    // Save the audio file
    let mut file = File::create(&file_path)
        .map_err(|e| Error::bad_request_with_message(format!("Failed to create file: {}", e)))?;
    
    file.write_all(&audio_bytes)
        .map_err(|e| Error::bad_request_with_message(format!("Failed to write file: {}", e)))?;

    // Return the URL to the generated music
    let music_url = format!("/v1/meditation/music/{}", filename);
    
    Ok(Json(GenerateMusicResponse { music_url }))
}

// Generate music prompt based on preferences
fn generate_music_prompt(preferences: &GenerateMusicRequest) -> String {
    let base_prompt = match preferences.music_atmosphere.as_str() {
        "nature" => "peaceful nature sounds with gentle flowing water, soft bird calls, and light forest ambience",
        "ambient" => "ambient ethereal soundscape with subtle drones and gentle atmospheric textures",
        "piano" => "soft minimalist piano with gentle reverb and occasional gentle string accompaniment",
        "binaural" => "binaural beats at alpha frequency range with soft ambient pads and gentle oscillations",
        "bowls" => "tibetan singing bowls and bells with long sustains and harmonically rich tones",
        "minimal" => "minimal ambient soundscape with occasional soft tones and comfortable silence",
        _ => "calm meditation music with soft ambient elements",
    };

    let mood = match preferences.focus_area.as_str() {
        "anxiety" => "calming, soothing, stress-reducing",
        "sleep" => "extremely gentle, hypnotic, sleep-inducing",
        "focus" => "subtly focusing, clear, present",
        "gratitude" => "warm, uplifting, gentle positivity",
        "compassion" => "heartwarming, loving, kind",
        "pain" => "healing, pain-relieving, distracting",
        "energy" => "subtly energizing, refreshing, revitalizing",
        _ => "peaceful, calming, centered",
    };

    let elements = match preferences.background.as_str() {
        "forest" => "with subtle woodland elements and gentle breeze sounds",
        "beach" => "with distant soft waves and occasional ocean elements",
        "mountain" => "with subtle high-altitude wind and open space feeling",
        "garden" => "with gentle garden ambience and subtle natural elements",
        "space" => "with cosmic overtones and vast spacious feeling",
        _ => "with gentle natural elements",
    };

    format!(
        "{} - {}, perfect for {} meditation, {}. The music should last at least {} minutes.",
        base_prompt, mood, preferences.meditation_type, elements, preferences.duration
    )
}

// Serve audio files
async fn serve_audio_file(
    Path(filename): Path<String>,
    State(state): State<AppState>,
) -> Result<(HeaderMap, Vec<u8>), Error> {
    let path = state.music_dir.join(&filename);
    
    // Check if file exists
    if !path.exists() {
        return Err(Error::not_found());
    }
    
    // Read file
    let audio_data = tokio::fs::read(path)
        .await
        .map_err(|e| Error::bad_request_with_message(format!("Failed to read file: {}", e)))?;
    
    // Set headers
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "audio/mpeg".parse().unwrap());
    headers.insert(
        header::CONTENT_DISPOSITION, 
        format!("attachment; filename=\"{}\"", filename).parse().unwrap()
    );
    
    Ok((headers, audio_data))
}