use actix_web::{get, http::StatusCode, web, HttpResponse};
use diesel::prelude::*;

use crate::{
    base_types::shipment_status::ShipmentStatus,
    contracts::shipment::Shipment,
    db::connection::{get_conn, SqliteConnectionPool},
    middlewares::user_info::UserInfo,
};

#[get("")]
pub async fn get(pool: web::Data<SqliteConnectionPool>, user: UserInfo) -> HttpResponse {
    use crate::schema::{orders, shipments, users};

    println!("User id is {}", user.user_id);

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
        ))
        .load::<Shipment>(conn)
    {
        Ok(s) => HttpResponse::Ok().json(serde_json::json!({ "shipments": s })),
        Err(_) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({ "message": "Oops! Something went wrong." })),
    }
}
