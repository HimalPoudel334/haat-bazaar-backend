use actix_web::{delete, HttpRequest};
use actix_web::{http::StatusCode, post, web, HttpResponse, Responder};
use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;

use crate::contracts::auth::RefreshCredentials;
use crate::models::refresh_token::{NewRefreshToken, RefreshToken};
use crate::utils::jwt_helper::{create_refresh_token, get_expiration, verify_jwt};
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
    use crate::schema::refresh_tokens::dsl::*;
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
    let tok = create_jwt_token(
        login_user.get_uuid().to_owned(),
        login_user.get_user_type().to_owned(),
        app_config.jwt_maxage,
        app_config.jwt_secret.to_owned(),
    )
    .await;

    match tok {
        Ok(tk) => {
            let expiration = get_expiration(7 * 24 * 60);

            let ref_tok = create_refresh_token(
                login_user.get_uuid().to_string(),
                &app_config.refresh_token_secret,
                expiration.0,
            )
            .await;

            match ref_tok {
                Ok(rt) => {
                    let rt_new = NewRefreshToken::new(&login_user, rt.clone(), &expiration.1);
                    match diesel::insert_into(refresh_tokens).values(&rt_new).execute(conn) {
                        Ok(_) => {
                            let login_response = LoginResponse {
                                access_token: tk,
                                refresh_token: rt,
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
                        },
                        Err(_) => HttpResponse::InternalServerError()
                                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                                    .json(
                                        serde_json::json!({"message": "Ops! something went wrong. Please try again later"}),
                                    ),

                    }
                }
                Err(_) => return HttpResponse::InternalServerError()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .json(
                        serde_json::json!({"message": "Ops! something went wrong. Please try again later"}),
                    )
            }
        }
        Err(_) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(
                serde_json::json!({"message": "Ops! something went wrong. Please try again later"}),
            ),
    }
}

#[post("/refresh")]
pub async fn refresh_token(
    tokens: web::Data<RefreshCredentials>,
    pool: web::Data<SqliteConnectionPool>,
    app_config: web::Data<ApplicationConfiguration>,
) -> impl Responder {
    use crate::schema::refresh_tokens::dsl::*;
    use crate::schema::users::dsl::*;
    use crate::schema::{refresh_tokens, users};

    let conn = &mut get_conn(&pool);

    let decoded = verify_jwt(
        tokens.access_token.as_ref(),
        app_config.refresh_token_secret.as_bytes(),
    )
    .await;

    match decoded {
        Ok(claims) => {
            let usr_id = claims.sub;
            //find the refresh token and check for validity and issue a new access token.
            let user = match users
                .filter(users::uuid.eq(&usr_id))
                .select(UserModel::as_select())
                .first::<UserModel>(conn)
                .optional()
            {
                Ok(us) => match us {
                    Some(u) => u,
                    None => {
                        return HttpResponse::Unauthorized()
                            .status(StatusCode::UNAUTHORIZED)
                            .json(serde_json::json!({"message": "Invalid access token provided"}))
                    }
                },
                Err(e) => {
                    return HttpResponse::InternalServerError()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .json(
                            serde_json::json!({"message": format!("Internal server error: {}", e)}),
                        )
                }
            };

            let ref_tok = match refresh_tokens
                .filter(refresh_tokens::user_id.eq(user.get_id()))
                .select(RefreshToken::as_select())
                .first::<RefreshToken>(conn)
                .optional()
            {
                Ok(rt) => match rt {
                    Some(t) => t,
                    None => {
                        return HttpResponse::Unauthorized()
                            .status(StatusCode::UNAUTHORIZED)
                            .json(serde_json::json!({"message": "Invalid access token provided"}))
                    }
                },
                Err(e) => {
                    return HttpResponse::InternalServerError()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .json(
                            serde_json::json!({"message": format!("Internal server error: {}", e)}),
                        )
                }
            };

            if ref_tok.get_token() != tokens.refresh_token {
                return HttpResponse::Unauthorized()
                    .status(StatusCode::UNAUTHORIZED)
                    .json(serde_json::json!({"message": "Invalid refresh token provided"}));
            }

            let ref_tok_expiry =
                NaiveDateTime::parse_from_str(ref_tok.get_expires_on(), "%Y-%m-%d %H:%M:%S");

            if let Err(_) = ref_tok_expiry {
                return HttpResponse::Unauthorized()
                    .status(StatusCode::UNAUTHORIZED)
                    .json(serde_json::json!({"message": "Invalid refresh token provided"}));
            }

            let now = Utc::now().naive_utc();

            if ref_tok_expiry.unwrap() < now {
                return HttpResponse::Unauthorized()
                    .status(StatusCode::UNAUTHORIZED)
                    .json(serde_json::json!({"message": "Invalid refresh token provided"}));
            }

            let access_token = create_jwt_token(
                user.get_uuid().to_owned(),
                user.get_user_type().to_owned(),
                app_config.jwt_maxage,
                app_config.jwt_secret.to_owned(),
            )
            .await;

            if let Err(e) = access_token {
                return HttpResponse::InternalServerError()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .json(serde_json::json!({"message": format!("Internal server error: {}", e)}));
            }

            let expiration = get_expiration(7 * 24 * 60);

            let refresh_tok = create_refresh_token(
                user.get_uuid().to_owned(),
                &app_config.refresh_token_secret,
                expiration.0,
            )
            .await;

            if let Err(e) = &refresh_tok {
                return HttpResponse::InternalServerError()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .json(
                        serde_json::json!({ "message": format!("Internal server error: {}", e) }),
                    );
            }

            // Safe to unwrap now, because we handled the error case
            let refresh_tok = refresh_tok.unwrap();

            match diesel::update(&ref_tok)
                .set((
                    refresh_tokens::expires_on.eq(expiration.1),
                    refresh_tokens::token.eq(&refresh_tok),
                ))
                .execute(conn)
            {
                Ok(_) => HttpResponse::Ok()
                    .status(StatusCode::OK)
                    .json(serde_json::json!({
                        "accessToken": access_token.unwrap(),
                        "refreshToken": refresh_tok
                    })),
                Err(e) => HttpResponse::InternalServerError()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .json(
                        serde_json::json!({ "message": format!("Internal server error: {}", e) }),
                    ),
            }
        }
        Err(_) => {
            return HttpResponse::Unauthorized()
                .status(StatusCode::UNAUTHORIZED)
                .json(serde_json::json!({ "message": "Invalid access token" }));
        }
    }
}

#[delete("/logout")]
pub async fn logout(req: HttpRequest) -> impl Responder {
    // Extract the token from the Authorization header
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let _token = &auth_str[7..];
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
