use actix_web::{http::StatusCode, HttpResponse, Responder};
use diesel::result::DatabaseErrorInformation;
use uuid::Uuid;

pub fn validate_uuid(uuid: &str) -> Result<Uuid, HttpResponse> {
    match Uuid::parse_str(uuid) {
        Ok(uid) => Ok(uid),
        Err(_) => Err(HttpResponse::BadRequest()
            .status(StatusCode::BAD_REQUEST)
            .json(serde_json::json!({"message": "Invalid uuid"}))),
    }
}

pub struct DatabaseErrorInfo {
    pub message: String,
}
impl DatabaseErrorInformation for DatabaseErrorInfo {
    fn message(&self) -> &str {
        &self.message
    }

    fn details(&self) -> Option<&str> {
        todo!()
    }

    fn hint(&self) -> Option<&str> {
        todo!()
    }

    fn table_name(&self) -> Option<&str> {
        todo!()
    }

    fn column_name(&self) -> Option<&str> {
        todo!()
    }

    fn constraint_name(&self) -> Option<&str> {
        todo!()
    }

    fn statement_position(&self) -> Option<i32> {
        todo!()
    }
}
