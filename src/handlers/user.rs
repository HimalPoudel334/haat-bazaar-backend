use actix_web::{delete, get, http::StatusCode, post, put, web, HttpResponse, Responder};
use diesel::prelude::*;
use uuid::Uuid;

use crate::{
    base_types::{email::Email, phone_number::PhoneNumber},
    contracts::user::{User, UserCreate, UserPendingShipments},
    db::connection::{get_conn, SqliteConnectionPool},
    models::user::{NewUser, User as UserModel},
};

#[get("")]
pub async fn get(pool: web::Data<SqliteConnectionPool>) -> impl Responder {
    use crate::schema::users::dsl::*;

    //get a pooled connection from db
    let conn = &mut get_conn(&pool);

    let user_vec = users.select(User::as_select()).load::<User>(conn);

    match user_vec {
        Ok(uv) => HttpResponse::Ok()
            .status(StatusCode::OK)
            .json(serde_json::json!({"users": uv})),
        Err(e) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message": e.to_string()})),
    }
}

#[post("")]
pub async fn create(
    user: web::Json<UserCreate>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    use crate::schema::users::dsl::*;

    let phone_num: PhoneNumber = match PhoneNumber::from_str(user.phone_number.to_owned()) {
        Ok(phone) => phone,
        Err(e) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": e}));
        }
    };

    // Location validation
    //Location should in the below format address,city,state,country
    let is_valid_location = match &user.location {
        Some(loc) => {
            if loc.matches(',').count() != 4 {
                false
            } else {
                loc.split(',').all(|part| !part.trim().is_empty())
            }
        }
        None => false,
    };

    if !is_valid_location {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "message": "Invalid location format. Expected: 'zip code, street, city, state, country'"
        }));
    }

    if let Err(e) = Email::from_str(&user.email) {
        return HttpResponse::BadRequest()
            .status(StatusCode::BAD_REQUEST)
            .json(serde_json::json!({"message": e}));
    }

    //get a pooled connection from db
    let conn = &mut get_conn(&pool);

    match users
        .filter(
            phone_number
                .eq(&user.phone_number)
                .or(email.eq(&user.email)),
        )
        .select(UserModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(Some(_)) => HttpResponse::BadRequest()
            .status(StatusCode::BAD_REQUEST)
            .json(serde_json::json!({"message": "Email or Phone number already used"})),
        Ok(None) => {
            let new_user = match NewUser::new(
                user.first_name.to_owned(),
                user.last_name.to_owned(),
                phone_num,
                user.email.to_owned(),
                user.password.to_owned(),
                user.location.to_owned(),
                user.nearest_landmark.to_owned(),
            ) {
                Ok(u) => u,
                Err(_e) => return HttpResponse::InternalServerError()
                   .status(StatusCode::INTERNAL_SERVER_ERROR)
                   .json(serde_json::json!({"message": "Something went wrong while creating a new user"})),
            };

            match diesel::insert_into(users)
                .values(&new_user)
                .get_result::<UserModel>(conn)
            {
                Ok(c) => {
                    let user_created: User = User {
                        first_name: c.get_first_name().to_owned(),
                        last_name: c.get_last_name().to_owned(),
                        uuid: c.get_uuid().to_owned(),
                        phone_number: c.get_phone_number().to_owned(),
                        email: c.get_email().into(),
                        user_type: c.get_user_type().to_owned(),
                        location: c.get_location().map(|s| s.to_owned()),
                        nearest_landmark: c.get_nearest_landmark().map(|s| s.to_owned()),
                    };
                    HttpResponse::Ok().status(StatusCode::OK).json(user_created)
                }
                Err(e) => HttpResponse::InternalServerError()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .json(serde_json::json!({"message": format!("{}", e)})),
            }
        }
        Err(e) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message": format!("{}", e)})),
    }
}

#[get("/{user_id}")]
pub async fn get_user(
    user_id: web::Path<(String,)>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    let uid: String = user_id.into_inner().0;
    //check if the user_id is valid uuid or not before trip to db
    let uid: Uuid = match Uuid::parse_str(uid.as_str()) {
        Ok(u) => u,
        Err(_) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Invalid user id"}));
        }
    };

    use crate::schema::users::dsl::*;

    //get a pooled connection from db
    let conn = &mut get_conn(&pool);

    match users
        .filter(uuid.eq(uid.to_string()))
        .select(User::as_select())
        .first(conn)
        .optional()
    {
        Ok(user) => match user {
            Some(u) => HttpResponse::Ok().status(StatusCode::OK).json(u),
            None => HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "User not found"})),
        },
        Err(e) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message": format!("Internal server error: {}", e)})),
    }
}

#[get("/phone-number/{phone_num}")]
pub async fn get_user_from_phone_number(
    phone_num: web::Path<(String,)>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    let phone_num_string: String = phone_num.into_inner().0;
    //check if the user_id is valid uuid or not before trip to db
    let ph_num: PhoneNumber = match PhoneNumber::from_str(phone_num_string) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": e}));
        }
    };

    use crate::schema::users::dsl::*;

    //get a pooled connection from db
    let conn = &mut get_conn(&pool);

    match users
        .filter(phone_number.eq(&ph_num.get_number()))
        .select(User::as_select())
        .first(conn)
        .optional()
    {
        Ok(user) => match user {
            Some(u) => HttpResponse::Ok().status(StatusCode::OK).json(u),
            None => HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "User not found"})),
        },
        Err(e) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message": format!("Internal server error: {}", e)})),
    }
}

#[get("/email/{email_str}")]
pub async fn get_user_from_email(
    email_str: web::Path<(String,)>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    let email_str: String = email_str.into_inner().0;
    //check if the user_id is valid uuid or not before trip to db
    if let Err(e) = Email::from_str(&email_str) {
        return HttpResponse::BadRequest()
            .status(StatusCode::BAD_REQUEST)
            .json(serde_json::json!({"message": e}));
    }

    use crate::schema::users;
    use crate::schema::users::dsl::*;

    //get a pooled connection from db
    let conn = &mut get_conn(&pool);

    match users
        .filter(users::email.eq(&email_str))
        .select(User::as_select())
        .first(conn)
        .optional()
    {
        Ok(user) => match user {
            Some(u) => HttpResponse::Ok().status(StatusCode::OK).json(u),
            None => HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "User not found"})),
        },
        Err(e) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message": format!("Internal server error: {}", e)})),
    }
}

#[get("/staffs")]
pub async fn get_staff_users(pool: web::Data<SqliteConnectionPool>) -> impl Responder {
    let conn = &mut get_conn(&pool);

    match diesel::sql_query(
        r#"
    SELECT 
        users.uuid AS user_id,
        users.first_name || ' ' || users.last_name AS full_name,
        users.location,
        users.nearest_landmark,
        COALESCE(
            SUM(CASE WHEN shipments.status = 'pending' THEN 1 ELSE 0 END), 
            0
        ) AS pending_shipment_count
    FROM users
    LEFT JOIN shipments 
      ON shipments.assigned_to = users.id
    WHERE users.user_type = ?
    GROUP BY users.uuid, users.first_name, users.last_name, users.location, users.nearest_landmark;
    "#,
    )
    .bind::<diesel::sql_types::Text, _>(UserModel::USERTYPE_DELIVERY)
    .load::<UserPendingShipments>(conn)
    {
        Ok(uv) => HttpResponse::Ok()
            .status(StatusCode::OK)
            .json(serde_json::json!({"users": uv})),
        Err(e) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message": e.to_string()})),
    }
}

#[put("/{user_id}")]
pub async fn edit(
    user_id: web::Path<(String,)>,
    user_update: web::Json<User>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    let uid: String = user_id.into_inner().0;
    //check if the user_id is valid uuid or not before trip to db
    let uid: Uuid = match Uuid::parse_str(uid.as_str()) {
        Ok(u) => u,
        Err(_) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Invalid user id"}));
        }
    };

    //check if the phone number is valid or not
    let _ = match PhoneNumber::from_str(user_update.phone_number.to_owned()) {
        Ok(_) => (),
        Err(e) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": e}));
        }
    };

    use crate::schema::users::dsl::*;

    //get a pooled connection from db
    let conn = &mut get_conn(&pool);

    //check if the new phone number is already used or not
    let user_from_phone = match users
        .filter(phone_number.eq(user_update.phone_number.to_string()))
        .select(UserModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(cu) => match cu {
            Some(c) => c,
            None => Default::default(), //return the default User struct if not found
        },
        Err(e) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": format!("Internal server error: {}", e)}));
        }
    };

    let user: UserModel = match users
        .filter(uuid.eq(uid.to_string()))
        .select(UserModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(cu) => match cu {
            Some(c) => {
                if c == user_from_phone {
                    c
                } else {
                    return HttpResponse::Conflict()
                        .status(StatusCode::CONFLICT)
                        .json(serde_json::json!({"message": "Phone number already used"}));
                }
            }
            None => {
                return HttpResponse::NotFound()
                    .status(StatusCode::NOT_FOUND)
                    .json(serde_json::json!({"message": "User not found"}));
            }
        },
        Err(e) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": format!("Internal server error: {}", e)}));
        }
    };

    let mut nearest_landmark_value = user.get_nearest_landmark().clone();

    if let Some(ref new_landmark) = user_update.nearest_landmark {
        nearest_landmark_value = Some(new_landmark);
    }

    match diesel::update(&user)
        .set((
            first_name.eq(&user_update.first_name),
            last_name.eq(&user_update.last_name),
            phone_number.eq(&user_update.phone_number),
            location.eq(&user_update.location),
            nearest_landmark.eq(nearest_landmark_value),
        ))
        .execute(conn)
    {
        Ok(_) => HttpResponse::Ok().json(user_update),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "message": format!("Internal server error: {}", e)
        })),
    }
}

#[delete("/{user_id}")]
pub async fn delete(user_id: web::Path<(String,)>) -> impl Responder {
    let _id: String = user_id.into_inner().0;
    HttpResponse::BadRequest()
        .status(StatusCode::BAD_REQUEST)
        .json(serde_json::json!({"message": "Cannot delete a resource"}))
}
