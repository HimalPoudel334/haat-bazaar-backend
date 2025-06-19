use actix_web::{delete, HttpRequest};
use actix_web::{http::StatusCode, post, web, HttpResponse, Responder};
use diesel::prelude::*;

use crate::{
    config::ApplicationConfiguration,
    contracts::{
        auth::{LoginCredentials, LoginResponse},
        user::User,
    },
    db::connection::{get_conn, SqliteConnectionPool},
    models::user::User as UserModel,
    utils::{jwt_helper::create_jwt_token, password_helper::verify_password_hash},
};

#[post("/login")]
pub async fn login(
    creds: web::Json<LoginCredentials>,
    pool: web::Data<SqliteConnectionPool>,
    app_config: web::Data<ApplicationConfiguration>,
) -> impl Responder {
    use crate::schema::users::dsl::*;

    let conn = &mut get_conn(&pool);

    let login_user = match users
        .filter(
            email
                .eq(&creds.username)
                .or(phone_number.eq(&creds.username)),
        )
        .select(UserModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(user) => match user {
            Some(u) => u,
            None => {
                return HttpResponse::Unauthorized()
                    .status(StatusCode::UNAUTHORIZED)
                    .json(serde_json::json!({"message": "Invalid Username or Password"}))
            }
        },
        Err(e) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": format!("Internal server error: {}", e)}))
        }
    };

    //verify password
    let is_valid = verify_password_hash(login_user.get_password(), &creds.password);
    if !is_valid {
        return HttpResponse::Unauthorized()
            .status(StatusCode::UNAUTHORIZED)
            .json(serde_json::json!({"message": "Invalid Username or Password"}));
    }
    println!("User {} logged in successfully", login_user.get_email());
    //jwt token
    let token = create_jwt_token(
        login_user.get_uuid().to_owned(),
        login_user.get_user_type().to_owned(),
        app_config.jwt_maxage,
        app_config.jwt_secret.to_owned(),
    )
    .await;

    match token {
        Ok(token) => {
            let login_response = LoginResponse {
                token,
                user: User {
                    uuid: login_user.get_uuid().to_owned(),
                    first_name: login_user.get_first_name().to_owned(),
                    last_name: login_user.get_last_name().to_owned(),
                    phone_number: login_user.get_phone_number().to_owned(),
                    email: login_user.get_email().to_owned(),
                    user_type: login_user.get_user_type().to_owned(),
                    location: login_user.get_location().map(|s| s.to_owned()),
                    nearest_landmark: login_user.get_nearest_landmark().map(|s| s.to_owned()),
                },
            };
            HttpResponse::Ok()
                .status(StatusCode::OK)
                .json(serde_json::json!(login_response))
        }
        Err(_) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(
                serde_json::json!({"message": "Ops! something went wrong. Please try again later"}),
            ),
    }
}

#[delete("/logout")]
pub async fn logout(req: HttpRequest) -> impl Responder {
    // Extract the token from the Authorization header
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let _token = &auth_str[7..];
                // Here you would typically invalidate the token, e.g., by adding it to a blacklist
                // or removing it from a database/cache if you are storing active tokens.
                // For now, we'll just simulate this step.

                return HttpResponse::Ok()
                    .status(StatusCode::OK)
                    .json(serde_json::json!({"message": "Logged out successfully"}));
            }
        }
    }

    HttpResponse::BadRequest()
        .status(StatusCode::BAD_REQUEST)
        .json(serde_json::json!({"message": "Invalid or missing token"}))
}
