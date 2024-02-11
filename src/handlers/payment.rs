use ::uuid::Uuid;
use actix_web::{get, http::StatusCode, post, web, HttpResponse, Responder};
use diesel::prelude::*;

use crate::{
    base_types::payment_method::PaymentMethod,
    contracts::payment::{NewPayment, Payment},
    db::connection::{get_conn, SqliteConnectionPool},
    models::{
        customer::Customer as CustomerModel,
        order::Order as OrderModel,
        payment::{NewPayment as NewPaymentModel, Payment as PaymentModel},
    },
};

// I think any other method should not exists
#[post("")]
pub async fn create(
    payment_json: web::Json<NewPayment>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    //get a pooled connection from db
    let conn = &mut get_conn(&pool);

    use crate::schema::customers::dsl::*;
    use crate::schema::orders::dsl::*;
    use crate::schema::payments::dsl::*;
    use crate::schema::{customers, orders};

    //first check if the uuids are valid or not
    let o_uuid: Uuid = match Uuid::parse_str(&payment_json.order_id) {
        Ok(o) => o,
        Err(_) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Invalid order id"}))
        }
    };

    let c_uuid: Uuid = match Uuid::parse_str(&payment_json.customer_id) {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Invalid customer id"}))
        }
    };

    //check if the payment method provided is valid
    let pay_method: PaymentMethod = match PaymentMethod::from_str(&payment_json.payment_method) {
        Ok(pm) => pm,
        Err(e) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": e}))
        }
    };

    //check if the order_id and customer_id are valid or not
    let order: OrderModel = match orders
        .filter(orders::uuid.eq(&o_uuid.to_string()))
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

    let customer: CustomerModel = match customers
        .filter(customers::uuid.eq(&c_uuid.to_string()))
        .select(CustomerModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(Some(c)) => c,
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

    //check if customer id from order and customer id from payment are same
    if order.get_customer_id() != customer.get_id() {
        return HttpResponse::BadRequest()
            .status(StatusCode::BAD_REQUEST)
            .json(serde_json::json!({"message": "Customers do not match"}));
    }

    //if the customer and order are valid then check if the order total and payment amount matches
    if order.get_total_price() != payment_json.amount {
        return HttpResponse::BadRequest()
            .status(StatusCode::BAD_REQUEST)
            .json(serde_json::json!({"message": "Order total and payment amount did not match"}));
    }

    //aaile ko laagi eti nai validation
    //if everything went good then create a new payment
    let payment: NewPaymentModel = NewPaymentModel::new(
        &pay_method,
        &customer,
        &order,
        &payment_json.pay_date,
        payment_json.amount,
    );

    match diesel::insert_into(payments)
        .values(&payment)
        .get_result::<PaymentModel>(conn)
    {
        Ok(p) => {
            let pay: Payment = Payment {
                uuid: p.get_uuid().to_owned(),
                payment_method: p.get_payment_method().to_owned(),
                pay_date: p.get_pay_date().to_owned(),
                customer_id: customer.get_uuid().to_owned(),
                order_id: order.get_uuid().to_owned(),
                amount: p.get_amount(),
            };
            HttpResponse::Ok().status(StatusCode::OK).json(pay)
        }
        Err(_) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message": "Ops! something went wrong"})),
    }
}

#[get("/{order_id}")]
pub async fn get(
    ord_id: web::Path<(String,)>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    //first check if the order_id is valid or not
    let ord_id: String = ord_id.into_inner().0;

    let ord_id: Uuid = match Uuid::parse_str(&ord_id) {
        Ok(o) => o,
        Err(_) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Invalid order id"}))
        }
    };

    //check if payment exists or not
    use crate::schema::customers::dsl::*;
    use crate::schema::orders::dsl::*;
    use crate::schema::payments::dsl::*;
    use crate::schema::{orders, payments};

    //get a pooled connection from db
    let conn = &mut get_conn(&pool);

    let order: OrderModel = match orders
        .filter(orders::uuid.eq(&ord_id.to_string()))
        .select(OrderModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(Some(o)) => o,
        Ok(None) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Invalid order id"}))
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}))
        }
    };

    let payment: PaymentModel = match payments
        .filter(payments::order_id.eq(order.get_id()))
        .select(PaymentModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(Some(p)) => p,
        Ok(None) => {
            return HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "Payment not found for order."}))
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}))
        }
    };

    let customer: CustomerModel = match customers
        .find(payment.get_customer_id())
        .select(CustomerModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(Some(c)) => c,
        Ok(None) => {
            return HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "Invalid customer id for payment"}))
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}))
        }
    };

    let payment_response: Payment = Payment {
        uuid: payment.get_uuid().to_owned(),
        customer_id: customer.get_uuid().to_owned(),
        order_id: order.get_uuid().to_owned(),
        pay_date: payment.get_pay_date().to_owned(),
        amount: payment.get_amount(),
        payment_method: payment.get_payment_method().to_owned(),
    };

    HttpResponse::Ok()
        .status(StatusCode::OK)
        .json(payment_response)
}
