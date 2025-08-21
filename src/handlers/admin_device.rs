use actix_web::{http::StatusCode, post, web, HttpResponse, Responder};
use diesel::prelude::*;

use crate::{
    contracts::admin_device::AdminDevice,
    db::connection::{get_conn, SqliteConnectionPool},
    models::{admin_device::NewAdminDevice, user::User},
};

#[post("/register-fcm-token")]
pub async fn register_fcm_token(
    token: web::Json<AdminDevice>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    use crate::schema::users::dsl::*;
    use crate::schema::{admin_devices, users};

    let conn = &mut get_conn(&pool);

    let user = match users
        .filter(users::uuid.eq(&token.user_id))
        .select(User::as_select())
        .first(conn)
        .optional()
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            return HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "User not found"}))
        }
        Err(_) => return HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(
                serde_json::json!({"message": "Ops! something went wrong. Please try again later"}),
            ),
    };

    if !user.is_admin() {
        return HttpResponse::Forbidden()
            .status(StatusCode::FORBIDDEN)
            .json(serde_json::json!({"message": "Request user is not an amdin"}));
    }

    let admin_dev = NewAdminDevice::new(&user, token.fcm_token.clone());

    match diesel::insert_into(admin_devices::table)
        .values(&admin_dev)
        .on_conflict(admin_devices::user_id)
        .do_update()
        .set(admin_devices::fcm_token.eq(&token.fcm_token))
        .execute(conn)
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(
                serde_json::json!({"message": "Ops! something went wrong. Please try again later"}),
            ),
    }
}
