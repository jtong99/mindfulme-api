pub mod cat;
pub mod user;
pub mod checkin;

use crate::utils::models::ModelExt;
use crate::errors::Error;

pub async fn sync_indexes() -> Result<(), Error> {
    user::User::sync_indexes().await?;
    cat::Cat::sync_indexes().await?;
    checkin::Checkin::sync_indexes().await?;

    Ok(())
}