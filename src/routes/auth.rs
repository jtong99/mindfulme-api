use axum::{routing::post, Json, Router};
use bson::doc;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::errors::Error;
use crate::models::user;
use crate::models::user::{PublicUser, User};
use crate::settings::SETTINGS;
use crate::utils::models::ModelExt;
use crate::utils::token;

pub fn create_route() -> Router {
    Router::new()
        .route("/api/auth/signup", post(signup))
        .route("/api/auth/signin", post(signin))
}

#[derive(Debug, Deserialize, Validate)]
pub struct SignupRequest {
    #[validate(email)]
    email: String,
    #[validate(length(min = 8))]
    password: String,
    #[validate(length(min = 1))]
    #[serde(rename = "firstName")]
    first_name: String,
    #[validate(length(min = 1))]
    #[serde(rename = "lastName")]
    last_name: String,
}

#[derive(Debug, Serialize)]
pub struct SignupResponseData {
    #[serde(rename = "userId")]
    user_id: String,
    email: String,
    #[serde(rename = "firstName")]
    first_name: String,
    #[serde(rename = "lastName")]
    last_name: String,
    #[serde(rename = "createdAt")]
    created_at: String,
    token: String,
}

#[derive(Debug, Serialize)]
pub struct SignupResponse {
    success: bool,
    message: String,
    data: SignupResponseData,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    success: bool,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SigninRequest {
    #[validate(email)]
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
pub struct SigninResponseData {
    #[serde(rename = "userId")]
    user_id: String,
    email: String,
    #[serde(rename = "firstName")]
    first_name: String,
    #[serde(rename = "lastName")]
    last_name: String,
    token: String,
}

#[derive(Debug, Serialize)]
pub struct SigninResponse {
    success: bool,
    message: String,
    data: SigninResponseData,
}


async fn signup(Json(payload): Json<SignupRequest>) -> Result<Json<SignupResponse>, Error> {
    // Check if user with email already exists
    let existing_user = User::find_one(doc! { "email": &payload.email }, None).await?;
    if existing_user.is_some() {
        return Err(Error::bad_request_with_message("Email already registered".to_string()));
    }

    // Hash the password
    let password = payload.password.clone(); // Clone the password to extend its lifetime
    let password_hash = user::hash_password(password).await?;
    
    // Create new user
    let user = User::new(
        payload.first_name,
        payload.last_name,
        payload.email,
        password_hash,
    );
    
    // Save user to database
    let user = User::create(user).await?;
    let public_user = PublicUser::from(user.clone());
    
    // Generate JWT token
    let secret = SETTINGS.auth.secret.as_str();
    let token = token::create(user, secret)?;
    
    // Format created_at date - convert it to rfc3339 string format
    let created_at = public_user.created_at.to_chrono().to_rfc3339();
    
    // Prepare response
    let response = SignupResponse {
        success: true,
        message: "User registered successfully".to_string(),
        data: SignupResponseData {
            user_id: public_user.id.to_hex(),
            email: public_user.email,
            first_name: public_user.first_name,
            last_name: public_user.last_name,
            created_at,
            token,
        },
    };
    
    Ok(Json(response))
}


async fn signin(Json(payload): Json<SigninRequest>) -> Result<Json<SigninResponse>, Error> {
    // Find user by email
    let user = User::find_one(doc! { "email": &payload.email }, None).await?
        .ok_or_else(|| Error::unauthorized_with_message("Invalid email or password".to_string()))?;

    // Verify password
    let is_valid = user::verify_password(payload.password, user.password.clone()).await?;
    if !is_valid {
        return Err(Error::unauthorized_with_message("Invalid email or password".to_string()));
    }
    
    // Generate JWT token
    let secret = SETTINGS.auth.secret.as_str();
    let token = token::create(user.clone(), secret)?;
    
    let public_user = PublicUser::from(user);
    
    // Prepare response
    let response = SigninResponse {
        success: true,
        message: "User signed in successfully".to_string(),
        data: SigninResponseData {
            user_id: public_user.id.to_hex(),
            email: public_user.email,
            first_name: public_user.first_name,
            last_name: public_user.last_name,
            token,
        },
    };
    
    Ok(Json(response))
}