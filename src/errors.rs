use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use bcrypt::BcryptError;
use serde_json::json;
use tokio::task::JoinError;
use wither::bson;
use wither::mongodb::error::Error as MongoError;
use wither::WitherError;

#[derive(thiserror::Error, Debug)]
#[error("...")]
pub enum Error {
    #[error("{0}")]
    Wither(#[from] WitherError),

    #[error("{0}")]
    Mongo(#[from] MongoError),

    #[error("Error parsing ObjectID {0}")]
    ParseObjectID(String),

    #[error("{0}")]
    SerializeMongoResponse(#[from] bson::de::Error),

    #[error("{0}")]
    Authenticate(#[from] AuthenticateError),

    #[error("{0}")]
    BadRequest(#[from] BadRequest),

    #[error("{0}")]
    NotFound(#[from] NotFound),

    #[error("{0}")]
    RunSyncTask(#[from] JoinError),

    #[error("{0}")]
    HashPassword(#[from] BcryptError),
    
    #[error("{0}")]
    TokenCreation(String),

    #[error("Error invalid password {0}")]
    InvalidPassword(String),
}

impl Error {
    fn get_codes(&self) -> (StatusCode, u16) {
        match *self {
            // 4XX Errors
            Error::ParseObjectID(_) => (StatusCode::BAD_REQUEST, 40001),
            Error::BadRequest(_) => (StatusCode::BAD_REQUEST, 40002),
            Error::NotFound(_) => (StatusCode::NOT_FOUND, 40003),
            Error::Authenticate(AuthenticateError::WrongCredentials) => {
                (StatusCode::UNAUTHORIZED, 40004)
            }
            Error::Authenticate(AuthenticateError::InvalidToken) => {
                (StatusCode::UNAUTHORIZED, 40005)
            }
            Error::Authenticate(AuthenticateError::Locked) => (StatusCode::LOCKED, 40006),
            Error::TokenCreation(_) => (StatusCode::INTERNAL_SERVER_ERROR, 40007),

            // 5XX Errors
            Error::Authenticate(AuthenticateError::TokenCreation) => {
                (StatusCode::INTERNAL_SERVER_ERROR, 5001)
            }
            Error::Wither(_) => (StatusCode::INTERNAL_SERVER_ERROR, 5002),
            Error::Mongo(_) => (StatusCode::INTERNAL_SERVER_ERROR, 5003),
            Error::SerializeMongoResponse(_) => (StatusCode::INTERNAL_SERVER_ERROR, 5004),
            Error::RunSyncTask(_) => (StatusCode::INTERNAL_SERVER_ERROR, 5005),
            Error::HashPassword(_) => (StatusCode::INTERNAL_SERVER_ERROR, 5006),
            Error::InvalidPassword(_) => (StatusCode::UNAUTHORIZED, 40008),
        }
    }

    pub fn bad_request() -> Self {
        Error::BadRequest(BadRequest {
            message: "Bad Request".to_string(),
        })
    }
    
    pub fn bad_request_with_message(message: String) -> Self {
        Error::BadRequest(BadRequest { message })
    }

    pub fn not_found() -> Self {
        Error::NotFound(NotFound {})
    }

    pub fn unauthorized_with_message(message: String) -> Self {
        Error::Authenticate(AuthenticateError::WrongCredentials)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status_code, code) = self.get_codes();
        let message = self.to_string();
        
        // Create error response format in line with the app's expected format
        let body = Json(json!({
            "success": false,
            "message": message,
            "error": code.to_string()
        }));

        (status_code, body).into_response()
    }
}

#[derive(thiserror::Error, Debug)]
#[error("...")]
pub enum AuthenticateError {
    #[error("Wrong authentication credentials")]
    WrongCredentials,
    #[error("Failed to create authentication token")]
    TokenCreation,
    #[error("Invalid authentication credentials")]
    InvalidToken,
    #[error("User is locked")]
    Locked,
}

#[derive(thiserror::Error, Debug)]
#[error("{message}")]
pub struct BadRequest {
    pub message: String,
}

#[derive(thiserror::Error, Debug)]
#[error("Not found")]
pub struct NotFound {}