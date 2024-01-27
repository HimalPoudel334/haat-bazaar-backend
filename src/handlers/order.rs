use ::uuid::Uuid;
use actix_web::{get, http::StatusCode, patch, post, put, web, HttpResponse, Responder};
use diesel::prelude::*;

use crate::{
    base_types::delivery_status::DeliveryStatus,
    contracts::order::{Order, OrderCreate, OrderDeliveryStatus, OrderEdit},
    db::connection::{get_conn, SqliteConnectionPool},
    models::{
        customer::Customer as CustomerModel, order::NewOrder, order::Order as OrderModel,
        order_details::NewOrderDetail as NewOrderDetailModel, product::Product as ProductModel,
    },
};

#[get("")]
pub async fn get_orders(pool: web::Data<SqliteConnectionPool>) -> impl Responder {
    use crate::schema::customers::dsl::*;
    use crate::schema::orders::dsl::*;
    use crate::schema::{customers, orders};

    //get a pooled connection from db
    let conn = &mut get_conn(&pool);

    let orders_vec: Vec<Order> = match orders
        .inner_join(customers)
        .select((
            orders::uuid,
            created_on,
            fulfilled_on,
            delivery_location,
            delivery_status,
            total_price,
            customers::uuid,
        ))
        .load::<Order>(conn)
    {
        Ok(o) => o,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}));
        }
    };

    HttpResponse::Ok()
        .status(StatusCode::OK)
        .json(serde_json::json!({"orders": orders_vec}))
}

#[get("/{order_id}")]
pub async fn get_order(
    order_id: web::Path<(String,)>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    let order_id: String = order_id.into_inner().0;

    //check if the order id is a valid uuid
    let order_id: Uuid = match Uuid::parse_str(&order_id) {
        Ok(o) => o,
        Err(_) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Invalid order id"}))
        }
    };

    use crate::schema::customers::dsl::*;
    use crate::schema::orders::dsl::*;
    use crate::schema::{customers, orders};

    //get a pooled connection from db
    let conn = &mut get_conn(&pool);

    match orders
        .inner_join(customers)
        .filter(orders::uuid.eq(&order_id.to_string()))
        .select((
            orders::uuid,
            created_on,
            fulfilled_on,
            delivery_location,
            delivery_status,
            total_price,
            customers::uuid,
        ))
        .first::<Order>(conn)
        .optional()
    {
        Ok(Some(o)) => HttpResponse::Ok().status(StatusCode::OK).json(o),
        Ok(None) => HttpResponse::NotFound()
            .status(StatusCode::NOT_FOUND)
            .json(serde_json::json!({"message": "Order not found"})),
        Err(_) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message": "Ops! something went wrong"})),
    }
}
#[post("")]
pub async fn create(
    order_json: web::Json<OrderCreate>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    //first validate the customer exists or not
    //before that lets check whether the provided customer id is a valid guid or not
    let customer_uuid: Uuid = match Uuid::parse_str(&order_json.customer_id) {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Invalid customer id"}));
        }
    };

    let mut order_total: f64 = 0.0;
    (&order_json.order_details).into_iter().for_each(|od| {
        order_total += od.price;
    });

    if order_total != order_json.total_price {
        return HttpResponse::BadRequest()
            .status(StatusCode::BAD_REQUEST)
            .json(serde_json::json!({"message": "Order data tempered.\nPrice of items and order total do not match"}));
    }

    use crate::schema::customers::dsl::*;
    use crate::schema::order_details::dsl::*;
    use crate::schema::orders::dsl::*;
    use crate::schema::products::dsl::*;
    use crate::schema::{customers, products};

    //get a pooled connection from db
    let conn = &mut get_conn(&pool);

    let customer: CustomerModel = match customers
        .filter(customers::uuid.eq(customer_uuid.to_string()))
        .select(CustomerModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(Some(c)) => c,
        Ok(None) => {
            return HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "Customer not found"}));
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}));
        }
    };

    let order: NewOrder = NewOrder::new(
        &customer,
        order_json.created_on.to_owned(),
        order_json.total_price.to_owned(),
        DeliveryStatus::Pending,
        order_json.delivery_location.to_owned(),
    );

    match diesel::insert_into(orders)
        .values(&order)
        .get_result::<OrderModel>(conn)
    {
        Ok(o) => {
            //if any one of this failed, then god will help
            for order_detail in &order_json.order_details {
                let pr: ProductModel = match products
                    .filter(products::uuid.eq(&order_detail.product_id))
                    .select(ProductModel::as_select())
                    .first(conn)
                    .optional()
                {
                    Ok(Some(p)) => p,
                    Ok(None) => {
                        return HttpResponse::NotFound()
                            .status(StatusCode::NOT_FOUND)
                            .json(serde_json::json!({"message": "Product not found"}))
                    }
                    Err(_) => {
                        return HttpResponse::InternalServerError()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .json(serde_json::json!({"message": "Ops! something went wrong"}))
                    }
                };

                if pr.get_stock() < order_detail.quantity {
                    return HttpResponse::BadRequest()
                            .status(StatusCode::BAD_REQUEST)
                            .json(serde_json::json!({"message": "Ordered product quantity is greater than stock"}));
                }

                let od: NewOrderDetailModel =
                    NewOrderDetailModel::new(order_detail.quantity, order_detail.price, &pr, &o);

                diesel::insert_into(order_details)
                    .values(&od)
                    .execute(conn)
                    .unwrap();

                //update the product stock if order creation successful
                diesel::update(&pr)
                    .set(products::stock.eq(products::stock - order_detail.quantity))
                    .execute(conn)
                    .unwrap();
            }

            let order: Order = Order {
                customer_id: customer_uuid.to_string(),
                created_on: o.get_created_on().to_owned(),
                fulfilled_on: o.get_fulfilled_on().to_owned(),
                total_price: o.get_total_price(),
                uuid: o.get_uuid().to_string(),
                delivery_location: o.get_delivery_location().to_owned(),
                delivery_status: o.get_delivery_status().to_owned(),
            };
            HttpResponse::Ok().status(StatusCode::OK).json(order)
        }
        Err(_) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message": "Ops! something went wrong"})),
    }
}

#[put("/{order_id}")]
pub async fn edit(
    order_id: web::Path<(String,)>,
    order_json: web::Json<OrderEdit>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    let order_uid: String = order_id.into_inner().0;

    //first validate the customer exists or not
    //before that lets check whether the provided customer id is a valid guid or not
    let customer_uuid: Uuid = match Uuid::parse_str(&order_json.customer_id) {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Invalid customer id"}));
        }
    };

    let order_uid: Uuid = match Uuid::parse_str(&order_uid) {
        Ok(o) => o,
        Err(_) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Invalid order id"}));
        }
    };

    use crate::schema::customers::dsl::*;
    use crate::schema::orders::dsl::*;
    use crate::schema::{customers, orders};

    //get a pooled connection from db
    let conn = &mut get_conn(&pool);

    //find the order
    let order: OrderModel = match orders
        .filter(orders::uuid.eq(&order_uid.to_string()))
        .select(OrderModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(Some(o)) => o,
        Ok(None) => {
            return HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "Order not found"        }))
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}))
        }
    };

    //find the customer for the provided customer id
    let customer: CustomerModel = match customers
        .filter(customers::uuid.eq(customer_uuid.to_string()))
        .select(CustomerModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(Some(c)) => c,
        Ok(None) => {
            return HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "Customer not found"}));
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}));
        }
    };

    match diesel::update(&order)
        .set((
            customer_id.eq(customer.get_id()),
            fulfilled_on.eq(&order_json.fulfilled_on),
            delivery_status.eq(&order_json.delivery_status),
            delivery_location.eq(&order_json.delivery_location),
            total_price.eq(order_json.total_price),
        ))
        .get_result::<OrderModel>(conn)
    {
        Ok(o) => {
            let order: Order = Order {
                customer_id: customer_uuid.to_string(),
                created_on: o.get_created_on().to_owned(),
                fulfilled_on: o.get_fulfilled_on().to_owned(),
                total_price: o.get_total_price(),
                uuid: order_uid.to_string(),
                delivery_location: o.get_delivery_location().to_owned(),
                delivery_status: o.get_delivery_status().to_owned(),
            };
            HttpResponse::Ok().status(StatusCode::OK).json(order)
        }
        Err(_) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message": "Ops! something went wrong"})),
    }
}

#[patch("/{order_id}/delivery-status/update")]
pub async fn update_delivery_status(
    order_id: web::Path<(String,)>,
    order_delivery_status: web::Query<OrderDeliveryStatus>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    let order_uid: String = order_id.into_inner().0;

    //check if the given id is a valid guid or not
    let order_uid: Uuid = match Uuid::parse_str(&order_uid) {
        Ok(o_id) => o_id,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}));
        }
    };

    let status: DeliveryStatus = match order_delivery_status.delivery_status.as_str() {
        "Pending" => DeliveryStatus::Pending,
        "On the way" => DeliveryStatus::OnTheWay,
        "Fulfilled" => DeliveryStatus::Fulfilled,
        "Cancelled" => DeliveryStatus::Cancelled,
        _ => return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Invalid delivery status. Valid values are 'Peding', 'Cancelled', 'On the way', 'Fulfilled'"}))

    };

    //find the order
    use crate::schema::orders::dsl::*;

    //get a pooled connection from db
    let conn = &mut get_conn(&pool);

    let order: OrderModel = match orders
        .filter(uuid.eq(&order_uid.to_string()))
        .select(OrderModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(Some(o)) => o,
        Ok(None) => {
            return HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "Order not found"        }))
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}))
        }
    };

    match diesel::update(&order)
        .set(delivery_status.eq(status.value()))
        .get_result::<OrderModel>(conn)
    {
        Ok(o) => {
            let order: Order = Order {
                customer_id: String::from("N/A"),
                created_on: o.get_created_on().to_owned(),
                fulfilled_on: o.get_fulfilled_on().to_owned(),
                total_price: o.get_total_price(),
                uuid: order_uid.to_string(),
                delivery_location: o.get_delivery_location().to_owned(),
                delivery_status: o.get_delivery_status().to_owned(),
            };
            HttpResponse::Ok().status(StatusCode::OK).json(order)
        }
        Err(_) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message": "Ops! something went wrong"})),
    }
}
