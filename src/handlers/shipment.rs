use actix_web::{get, http::StatusCode, patch, web, HttpResponse, Responder};
use diesel::prelude::*;

use crate::{
    base_types::shipment_status::ShipmentStatus,
    contracts::shipment::{AssingShipment, Shipment},
    db::connection::{get_conn, SqliteConnectionPool},
    middlewares::user_info::UserInfo,
    models::{shipment::Shipment as ShipmentModel, user::User},
    utils::uuid_validator,
};

#[get("")]
pub async fn get(pool: web::Data<SqliteConnectionPool>, user: UserInfo) -> impl Responder {
    use crate::schema::{orders, shipments, users};

    println!("User id is {}", user.user_id);

    assert!(user.roles.contains("Admin"));
    assert_eq!(ShipmentStatus::Pending.value(), "Pending");

    let conn = &mut get_conn(&pool);

    let mut shipments_query = shipments::table
        .inner_join(orders::table.on(shipments::order_id.eq(orders::id)))
        .left_join(users::table.on(shipments::assigned_to.eq(users::id.nullable())))
        .order(shipments::ship_date.desc())
        .into_boxed(); //some runtime performance cost, if we want to remove that we have to create
                       //query in if and else condition.

    if !user.roles.contains("Admin") {
        shipments_query = shipments_query
            .filter(
                users::uuid
                    .eq(user.user_id.clone())
                    .or(shipments::assigned_to.is_null()),
            )
            .filter(shipments::status.eq(ShipmentStatus::Pending.value().to_string()));
    }

    match shipments_query
        .select((
            shipments::uuid,
            shipments::ship_date,
            shipments::address,
            shipments::city,
            shipments::state,
            shipments::country,
            shipments::zip_code,
            shipments::status,
            orders::uuid,
            users::uuid.nullable(),
            users::first_name.nullable(),
            users::last_name.nullable(),
        ))
        .load::<Shipment>(conn)
    {
        Ok(s) => HttpResponse::Ok().json(serde_json::json!({ "shipments": s })),
        Err(_) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({ "message": "Oops! Something went wrong." })),
    }
}

#[patch("assign")]
pub async fn assing_user_to_shipment(
    payload: web::Data<AssingShipment>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    let ship_id = match uuid_validator::validate_uuid(&payload.shipment_id) {
        Ok(u) => u,
        Err(res) => return res,
    };

    let u_id = match uuid_validator::validate_uuid(&payload.user_id) {
        Ok(u) => u,
        Err(res) => return res,
    };

    use crate::schema::shipments::dsl::*;
    use crate::schema::users::dsl::*;
    use crate::schema::{shipments, users};

    let conn = &mut get_conn(&pool);

    let user: User = match users
        .filter(users::uuid.eq(&u_id))
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
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong."}))
        }
    };

    let shipment: ShipmentModel = match shipments
        .filter(shipments::uuid.eq(&ship_id))
        .filter(shipments::status.eq(ShipmentStatus::Pending.value()))
        .select(ShipmentModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(Some(s)) => s,
        Ok(None) => {
            return HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "Shipment not found"}))
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong."}))
        }
    };

    match diesel::update(&shipment)
        .set(shipments::assigned_to.eq(user.get_id()))
        .execute(conn)
    {
        Ok(_) => HttpResponse::Ok().status(StatusCode::OK).finish(),
        Err(_) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message": "Ops! something went wrong."})),
    }
}
