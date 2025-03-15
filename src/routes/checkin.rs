// In src/routes/checkin.rs
use axum::{
    extract::Query,
    routing::{get, post},
    Json, Router,
};
use axum::http::StatusCode;  // Add this import for StatusCode
use bson::{doc, DateTime};
use chrono::{Datelike, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use tracing::debug;
use validator::Validate;  // Add this import for the validate attribute

use crate::errors::Error;
use crate::models::checkin::{Checkin, PublicCheckin};
use crate::utils::custom_response::CustomResponseResult as Response;
use crate::utils::custom_response::{CustomResponse, CustomResponseBuilder, ResponsePagination};
use crate::utils::models::ModelExt;
use crate::utils::token::TokenUser;
use crate::utils::pagination::Pagination;

pub fn create_route() -> Router {
    Router::new()
        .route("/api/checkin", post(create_checkin))
        .route("/api/checkin", get(get_user_checkins))
}

#[derive(Debug, Deserialize, Validate)]  // Now Validate trait is properly imported
pub struct CreateCheckinRequest {
    #[validate(range(min = 1, max = 5))]
    pub mood_rating: u8,
    pub primary_emotion: String,
    #[validate(range(min = 1, max = 5))]
    pub intensity: u8,
    #[validate(range(min = 1, max = 5))]
    pub energy_level: u8,
    #[validate(range(min = 1, max = 5))]
    pub stress_level: u8,
    #[validate(range(min = 1, max = 5))]
    pub wellbeing: u8,
    pub notes: Option<String>,
}

async fn create_checkin(
    user: TokenUser, 
    Json(payload): Json<CreateCheckinRequest>
) -> Response<PublicCheckin> {
    // Validate the payload with validator
    payload.validate().map_err(|e| Error::bad_request_with_message(format!("Validation error: {:?}", e)))?;
    
    // Validate primary_emotion is in the valid set
    if !crate::models::checkin::valid_emotions().contains(&payload.primary_emotion.as_str()) {
        return Err(Error::bad_request_with_message("Invalid primary emotion".to_string()));
    }
    
    let checkin = Checkin::new(
        user.id,
        payload.mood_rating,
        payload.primary_emotion,
        payload.intensity,
        payload.energy_level,
        payload.stress_level,
        payload.wellbeing,
        payload.notes,
    );
    
    let checkin = Checkin::create(checkin).await?;
    let public_checkin = PublicCheckin::from(checkin);
    
    let res = CustomResponseBuilder::new()
        .body(public_checkin)
        .status_code(StatusCode::CREATED)
        .build();
        
    Ok(res)
}

#[derive(Debug, Deserialize)]
pub struct CheckinQueryParams {
    month: Option<u32>,  // Month number (1-12)
    year: Option<i32>,   // Year (e.g., 2025)
}

async fn get_user_checkins(
    user: TokenUser,
    Query(params): Query<CheckinQueryParams>,
    pagination: Pagination,
) -> Response<Vec<PublicCheckin>> {
    // Start with a query that filters by user
    let mut query = doc! { "user": &user.id };
    
    // If both month and year are provided, add date filtering
    if let (Some(month), Some(year)) = (params.month, params.year) {
        // Validate month
        if month < 1 || month > 12 {
            return Err(Error::bad_request_with_message("Month must be between 1 and 12".to_string()));
        }
        
        // Create start and end dates for the month
        let start_date = match NaiveDate::from_ymd_opt(year, month, 1) {
            Some(date) => date,
            None => return Err(Error::bad_request_with_message("Invalid date".to_string())),
        };
        
        // Calculate the first day of the next month
        let end_month = if month == 12 { 1 } else { month + 1 };
        let end_year = if month == 12 { year + 1 } else { year };
        let end_date = match NaiveDate::from_ymd_opt(end_year, end_month, 1) {
            Some(date) => date,
            None => return Err(Error::bad_request_with_message("Invalid date".to_string())),
        };
        
        // Convert to MongoDB datetime format
        let start_datetime = DateTime::from_chrono(start_date.and_hms_opt(0, 0, 0).unwrap().and_utc());
        let end_datetime = DateTime::from_chrono(end_date.and_hms_opt(0, 0, 0).unwrap().and_utc());
        
        // Add date range to the query
        query.insert("created_at", doc! {
            "$gte": start_datetime,
            "$lt": end_datetime
        });
    }
    
    // Set up options for pagination and sorting
    let options = wither::mongodb::options::FindOptions::builder()
        .sort(doc! { "created_at": -1_i32 })  // Newest first
        .skip(pagination.offset)
        .limit(pagination.limit as i64)
        .build();
    
    // Find checkins matching the query
    let (checkins, count) = Checkin::find_and_count(query, options).await?;
    
    // Convert to public format
    let checkins = checkins.into_iter().map(Into::into).collect::<Vec<PublicCheckin>>();
    
    // Build response with pagination
    let res = CustomResponseBuilder::new()
        .body(checkins)
        .pagination(ResponsePagination {
            count,
            offset: pagination.offset,
            limit: pagination.limit,
        })
        .build();
    
    debug!("Returning user checkins");
    Ok(res)
}