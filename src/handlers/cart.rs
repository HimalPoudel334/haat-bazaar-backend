use actix_web::{http::StatusCode, web, HttpResponse, Responder};
use diesel::prelude::*;
use diesel::SelectableHelper;
use uuid::Uuid;

use crate::contracts::cart::Cart;
use crate::{
    db::connection::{get_conn, SqliteConnectionPool},
    models::customer::Customer as CustomerModel,
};

pub async fn get(
    cust_id: web::Path<(String,)>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    let cust_id: String = cust_id.into_inner().0;

    //check if the customer id is valid uuid or not
    let cust_id: Uuid = match Uuid::parse_str(&cust_id) {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Invalid customer id"}))
        }
    };

    //check if customer exists or not
    //maybe not needed

    use crate::schema::carts::dsl::*;
    use crate::schema::customers::dsl::*;
    use crate::schema::products::dsl::*;
    use crate::schema::{carts, customers, products};

    let customer: CustomerModel = match customers
        .filter(customers::uuid.eq(&cust_id.to_string()))
        .select(CustomerModel::as_select())
        .first(&mut get_conn(&pool))
        .optional()
    {
        Ok(Some(c)) => c,
        Ok(None) => {
            return HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "Customer not found"}))
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}))
        }
    };

    let cart: Cart = match carts
        .inner_join(customers)
        .inner_join(products)
        .filter(carts::customer_id.eq(&customer.get_id()))
        .select((
            carts::uuid,
            products::uuid,
            customers::uuid,
            carts::quantity,
            carts::created_on,
        ))
        .first::<Cart>(&mut get_conn(&pool))
        .optional()
    {
        Ok(Some(c)) => c,
        Ok(None) => {
            return HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "Customer not found"}))
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}))
        }
    };

    HttpResponse::Ok().json(cart)
}
