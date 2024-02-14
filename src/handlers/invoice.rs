use actix_web::{http::StatusCode, web, HttpResponse, Responder};
use diesel::prelude::*;

use crate::{
    contracts::invoice::NewInvoice,
    db::connection::{get_conn, SqliteConnectionPool},
    models::{
        customer::Customer as CustomerModel, invoice::NewInvoice as NewInvoiceModel,
        order::Order as OrderModel, payment::Payment as PaymentModel,
    },
};

pub fn create(
    inv_json: web::Json<NewInvoice>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    use crate::schema::customers::dsl::*;
    use crate::schema::orders;
    use crate::schema::orders::dsl::*;
    use crate::schema::payments::dsl::*;

    //get a database connection for pool
    let conn = &mut get_conn(&pool);

    //first check if order exists or not
    let order: OrderModel = match orders
        .filter(orders::uuid.eq(&inv_json.order_id))
        .select(OrderModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(Some(o)) => o,
        Ok(None) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Order not found"}))
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}))
        }
    };

    //check if payment is done or not
    let payment: PaymentModel = match payments
        .find(order.get_id())
        .select(PaymentModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(Some(p)) => {
            if p.get_uuid().eq(&inv_json.payment_id) {
                p
            } else {
                return HttpResponse::BadRequest()
                    .status(StatusCode::BAD_REQUEST)
                    .json(serde_json::json!({"message": "Invalid payment id provided"}));
            }
        }
        Ok(None) => return HttpResponse::BadRequest()
            .status(StatusCode::BAD_REQUEST)
            .json(
                serde_json::json!({"message": "Payment not done for the order. Please pay first"}),
            ),
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}))
        }
    };

    let customer: CustomerModel = match customers
        .find(order.get_customer_id())
        .select(CustomerModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(Some(c)) => {
            if c.get_uuid().eq(&inv_json.customer_id) {
                c
            } else {
                return HttpResponse::BadRequest()
                    .status(StatusCode::BAD_REQUEST)
                    .json(serde_json::json!({"message": "Invalid customer id provided"}));
            }
        }
        Ok(None) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Customer not found"}))
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}))
        }
    };

    let new_invoice: NewInvoiceModel = NewInvoiceModel::new(
        &inv_json.invoice_date,
        inv_json.sub_total,
        inv_json.vat_percent,
        &order,
        &customer,
        &payment,
    );

    HttpResponse::Ok().into()
}
