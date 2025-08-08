use std::sync::Arc;

use actix_web::{delete, put, HttpRequest};
use actix_web::{http::StatusCode, post, web, HttpResponse, Responder};
use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;

use crate::base_types::email::Email;
use crate::contracts::auth::{
    OtpResponse, PasswordChangePayload, PasswordResetRequest, RefreshCredentials,
    RefreshTokenResponse,
};
use crate::models::refresh_token::{NewRefreshToken, RefreshToken};
use crate::services::email_service::EmailService;
use crate::services::opt_service::OtpService;
use crate::utils::jwt_helper::{create_refresh_token, get_expiration, verify_jwt_with_validation};
use crate::utils::password_helper::hash_password;
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
    println!("Login hit");
    use crate::schema::refresh_tokens::dsl::*;
    use crate::schema::users::dsl::*;

    let conn = &mut get_conn(&pool);

    // Step 1: Get user by email or phone
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
        Ok(Some(user)) => user,
        Ok(None) => {
            return HttpResponse::Unauthorized()
                .status(StatusCode::UNAUTHORIZED)
                .json(serde_json::json!({ "message": "Invalid Username or Password" }));
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({ "message": format!("Internal server error: {}", e) }));
        }
    };

    // Step 2: Verify password
    if !verify_password_hash(login_user.get_password(), &creds.password) {
        return HttpResponse::Unauthorized()
            .status(StatusCode::UNAUTHORIZED)
            .json(serde_json::json!({ "message": "Invalid Username or Password" }));
    }

    // Step 3: Create access token
    let tok = create_jwt_token(
        login_user.get_uuid().to_owned(),
        login_user.get_user_type().to_owned(),
        app_config.jwt_maxage,
        app_config.jwt_secret.to_owned(),
    )
    .await;

    let tk = match tok {
        Ok(t) => t,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({ "message": "Oops! Could not generate access token" }));
        }
    };

    // Step 4: Create refresh token
    let expiration = get_expiration((app_config.refresh_token_maxage * 24 * 60) as i64); // 7 days
    let ref_tok = create_refresh_token(
        login_user.get_uuid().to_string(),
        &app_config.refresh_token_secret,
        expiration.0,
    )
    .await;

    let rt = match ref_tok {
        Ok(t) => t,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({ "message": "Oops! Could not generate refresh token" }));
        }
    };

    // Step 5: Insert or update refresh token
    let existing_token = refresh_tokens
        .filter(user_id.eq(login_user.get_id()))
        .first::<RefreshToken>(conn)
        .optional();

    match existing_token {
        Ok(Some(_)) => {
            // Update existing token
            if let Err(_) = diesel::update(refresh_tokens.filter(user_id.eq(login_user.get_id())))
                .set((token.eq(&rt), expires_on.eq(expiration.1)))
                .execute(conn)
            {
                return HttpResponse::InternalServerError()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .json(
                        serde_json::json!({ "message": "Oops! Failed to update refresh token" }),
                    );
            }
        }
        Ok(None) => {
            // Insert new token
            let rt_new = NewRefreshToken::new(&login_user, rt.clone(), &expiration.1);
            if let Err(_) = diesel::insert_into(refresh_tokens)
                .values(&rt_new)
                .execute(conn)
            {
                return HttpResponse::InternalServerError()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .json(
                        serde_json::json!({ "message": "Oops! Failed to insert refresh token" }),
                    );
            }
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({ "message": "Oops! Could not check refresh token" }));
        }
    }

    // Step 6: Build and return response
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
}

#[post("/refresh")]
pub async fn refresh_token(
    tokens: web::Json<RefreshCredentials>,
    pool: web::Data<SqliteConnectionPool>,
    app_config: web::Data<ApplicationConfiguration>,
) -> impl Responder {
    use crate::schema::refresh_tokens::dsl::*;
    use crate::schema::users::dsl::*;
    use crate::schema::{refresh_tokens, users};

    let conn = &mut get_conn(&pool);

    let decoded = verify_jwt_with_validation(
        tokens.access_token.as_ref(),
        app_config.jwt_secret.as_bytes(),
        false,
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
                            .json(serde_json::json!({"message": "Invalid access token provided. User not found"}))
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
                            .json(serde_json::json!({"message": "Invalid access token provided. Refresh token not found"}))
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
                    .json(serde_json::json!({"message": "Invalid refresh token provided. refresh token do not match"}));
            }

            let ref_tok_expiry =
                NaiveDateTime::parse_from_str(ref_tok.get_expires_on(), "%Y-%m-%d %H:%M:%S");

            if let Err(_) = ref_tok_expiry {
                return HttpResponse::Unauthorized()
                    .status(StatusCode::UNAUTHORIZED)
                    .json(serde_json::json!({"message": "Invalid refresh token provided, refresh token expiration parsing failed"}));
            }

            let now = Utc::now().naive_utc();

            if ref_tok_expiry.unwrap() < now {
                return HttpResponse::Unauthorized()
                    .status(StatusCode::UNAUTHORIZED)
                    .json(serde_json::json!({"message": "Invalid refresh token provided: refresh token expired"}));
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

            let expiration = get_expiration((app_config.refresh_token_maxage * 24 * 60) as i64); // 7 days

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
                    .json(RefreshTokenResponse {
                        access_token: access_token.unwrap(),
                        refresh_token: refresh_tok,
                    }),
                Err(e) => HttpResponse::InternalServerError()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .json(
                        serde_json::json!({ "message": format!("Internal server error: {}", e) }),
                    ),
            }
        }
        Err(e) => {
            return HttpResponse::Unauthorized()
                .status(StatusCode::UNAUTHORIZED)
                .json(serde_json::json!({ "message": format!("Invalid access token: {}", e)}));
        }
    }
}

#[put("/change-password")]
pub async fn change_password(
    payload: web::Json<PasswordChangePayload>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    if payload.new_password != payload.confirm_password {
        return HttpResponse::BadRequest()
            .status(StatusCode::BAD_REQUEST)
            .json(serde_json::json!({"message": "New password do not match confirm password"}));
    }

    use crate::schema::users::dsl::*;

    let conn = &mut get_conn(&pool);

    let user: UserModel =
        match users
            .filter(uuid.eq(&payload.user_id))
            .select(UserModel::as_select())
            .first(conn)
            .optional()
        {
            Ok(Some(u)) => u,
            Ok(None) => return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(
                    serde_json::json!({"message": "Invalid user id provided, Couldnot find user"}),
                ),
            Err(e) => return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(
                    serde_json::json!({ "message": format!("Ops! something went wrong: {}", e)}),
                ),
        };

    if !verify_password_hash(user.get_password(), &payload.current_password) {
        return HttpResponse::Unauthorized()
            .status(StatusCode::UNAUTHORIZED)
            .json(serde_json::json!({ "message": "Invalid Username or Password" }));
    }

    let hashed_password = match hash_password(&payload.new_password) {
        Ok(hp) => hp,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({ "message": format!("Ops! something went wrong: {}", e)}))
        }
    };

    match diesel::update(&user)
        .set(password.eq(hashed_password))
        .execute(conn)
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({ "message": format!("Ops! something went wrong: {}", e)})),
    }
}

#[post("/reset-password")]
pub async fn reset_password(
    payload: web::Json<PasswordResetRequest>,
    pool: web::Data<SqliteConnectionPool>,
    email_service: web::Data<Arc<dyn EmailService>>,
) -> impl Responder {
    let email_type = match Email::from_str(payload.email.clone()) {
        Ok(e) => e,
        Err(s) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": s}))
        }
    };

    use crate::schema::users::dsl::*;

    let conn = &mut get_conn(&pool);

    let user = match users
        .filter(email.eq(&email_type.get_email()))
        .select(UserModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Invalid user email provided"}));
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong {}"}));
        }
    };

    let user_fullname = user.get_fullname().clone();
    let email_service = email_service.into_inner();

    let otp_service = OtpService::new(&pool);
    match otp_service.has_valid_otp(user.get_id()) {
        Ok(true) => {
            return HttpResponse::TooManyRequests()
                .json("An OTP has already been sent. Please wait before requesting a new one.");
        }
        Err(_) => {
            return HttpResponse::InternalServerError().json("Database error");
        }
        _ => {} // Continue
    }

    let otp_expiry_minutes = 2;
    tokio::spawn(async move {
        let otp_record =
            match otp_service.create_password_reset_otp(user.get_id(), otp_expiry_minutes * 60) {
                Ok(otp) => otp,
                Err(e) => {
                    println!(
                        "Failed to create OTP for user {}: {:?}",
                        user.get_email(),
                        e
                    );
                    return;
                }
            };

        let email_content = format!(
            r#"
            <h2>Password Reset OTP</h2>
            <p>Hi {},</p>
            <p>You requested a password reset. Your OTP code is:</p>
            <h3 style="color: #007bff; font-size: 24px; letter-spacing: 3px;">{}</h3>
            <p>This code will expire in {} minutes.</p>
            <p>If you didn't request this password reset, please ignore this email.</p>
            <p><strong>Do not share this code with anyone.</strong></p>
            "#,
            user_fullname, otp_expiry_minutes, otp_record.otp_code
        );

        if let Err(e) = email_service
            .send_html_email(user.get_email(), "Password Reset OTP", &email_content, None)
            .await
        {
            println!("Failed to send OTP email to {}: {:?}", user.get_email(), e);
        } else {
            println!("OTP email sent successfully to {}", user.get_email());
        }
    });

    HttpResponse::Ok().status(StatusCode::OK).json(OtpResponse {
        message: "OTP sent to your email address. Please check your inbox.".to_string(),
        expires_in_minutes: otp_expiry_minutes,
    })
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
