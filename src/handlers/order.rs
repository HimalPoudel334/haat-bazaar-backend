use ::uuid::Uuid;
use actix_web::{get, http::StatusCode, patch, post, put, web, HttpResponse, Responder};
use diesel::prelude::*;

use crate::{
    base_types::delivery_status::DeliveryStatus,
    contracts::order::{
        CategoryN, CustomerN, Order, OrderCreate, OrderDeliveryStatus, OrderEdit, OrderItemsN,
        OrderN, ProductN,
    },
    db::connection::{get_conn, SqliteConnectionPool},
    models::{
        customer::Customer as CustomerModel,
        order::{NewOrder, Order as OrderModel},
        order_detail::NewOrderDetail as NewOrderDetailModel,
        product::Product as ProductModel,
    },
};

#[get("")]
pub async fn get_orders(pool: web::Data<SqliteConnectionPool>) -> impl Responder {
    use crate::schema::categories::dsl::*;
    use crate::schema::customers::dsl::*;
    use crate::schema::order_details::dsl::*;
    use crate::schema::orders::dsl::*;
    use crate::schema::products::dsl::*;
    use crate::schema::{categories, customers, order_details, orders, products};

    //get a pooled connection from db
    let conn = &mut get_conn(&pool);

    // let all_orders = orders.inner_join(order_details.inner_join(products)).inner_join(customers).select((OrderDetailsModel::as_select(), CustomerModel::as_select(), OrderModel::as_select())).load::<(OrderDetailsModel, CustomerModel, OrderModel)>(conn).unwrap();
    // println!("order-orderItems pairs: {all_orders:?}");

    // let orders_vec: Vec<Order> = match orders
    //     .inner_join(customers)
    //     .select((
    //         orders::uuid,
    //         created_on,
    //         fulfilled_on,
    //         delivery_location,
    //         delivery_status,
    //         total_price,
    //         customers::uuid,
    //     ))
    //     .load::<Order>(conn)
    // {
    //     Ok(o) => o,
    //     Err(_) => {
    //         return HttpResponse::InternalServerError()
    //             .status(StatusCode::INTERNAL_SERVER_ERROR)
    //             .json(serde_json::json!({"message": "Ops! something went wrong"}));
    //     }
    // };

    // let orders_vec: Vec<crate::contracts::order::OrderN> = match orders
    //     .inner_join(customers)
    //     .inner_join(order_details.inner_join(products.inner_join(categories)))
    //     .select((
    //         orders::uuid,
    //         created_on,
    //         fulfilled_on,
    //         delivery_location,
    //         delivery_status,
    //         total_price,
    //         (
    //             customers::uuid,
    //             customers::first_name,
    //             customers::last_name,
    //             customers::phone_number,
    //         ),
    //         (
    //             order_details::uuid,
    //             order_details::quantity,
    //             order_details::price,
    //             (
    //                 products::uuid,
    //                 products::name,
    //                 products::description,
    //                 products::image,
    //                 (categories::uuid, categories::name),
    //             ),
    //         ),
    //     ))
    //     .load::<crate::contracts::order::OrderN>(conn)
    // {
    //     Ok(o) => o,
    //     Err(_) => {
    //         return HttpResponse::InternalServerError()
    //             .status(StatusCode::INTERNAL_SERVER_ERROR)
    //             .json(serde_json::json!({"message": "Ops! something went wrong"}));
    //     }
    // };

    type OrderTuple = (
        String,
        String,
        String,
        String,
        String,
        f64,
        (String, String, String, String),
        (
            String,
            f64,
            f64,
            (
                String,
                String,
                String,
                String,
                f64,
                String,
                (String, String),
            ),
        ),
    );

    let orders_vec = orders
        .inner_join(customers.on(customer_id.eq(customers::id)))
        .inner_join(order_details.on(order_id.eq(orders::id)))
        .inner_join(products.on(order_details::product_id.eq(products::id)))
        .inner_join(categories.on(products::category_id.eq(categories::id)))
        .select((
            orders::uuid,
            orders::created_on,
            orders::fulfilled_on,
            orders::delivery_location,
            orders::delivery_status,
            orders::total_price,
            (
                customers::uuid,
                customers::first_name,
                customers::last_name,
                customers::phone_number,
            ),
            (
                order_details::uuid,
                order_details::quantity,
                order_details::price,
                (
                    products::uuid,
                    products::name,
                    products::description,
                    products::image,
                    products::price,
                    products::unit,
                    (categories::uuid, categories::name),
                ),
            ),
        ))
        .load::<OrderTuple>(conn)
        .expect("Error loading orders");

    let mut order_res: Vec<OrderN> = Vec::new();

    for (
        order_uuid,
        order_created_on,
        order_fulfilled_on,
        order_delivery_location,
        order_delivery_status,
        order_total_price,
        (customer_uuid, customer_first_name, customer_last_name, customer_phone_number),
        (
            order_item_uuid,
            order_item_quantity,
            order_item_price,
            (
                product_uuid,
                product_name,
                product_description,
                product_image,
                product_price,
                product_unit,
                (category_uuid, category_name),
            ),
        ),
    ) in orders_vec
    {
        let ordn = OrderN {
            uuid: order_uuid,
            created_on: order_created_on,
            fulfilled_on: order_fulfilled_on,
            delivery_location: order_delivery_location,
            delivery_status: order_delivery_status,
            total_price: order_total_price,
            customer: CustomerN {
                uuid: customer_uuid,
                first_name: customer_first_name,
                last_name: customer_last_name,
                phone_number: customer_phone_number,
            },
            order_items: vec![OrderItemsN {
                uuid: order_item_uuid,
                quantity: order_item_quantity,
                price: order_item_price,
                product: ProductN {
                    uuid: product_uuid,
                    name: product_name,
                    description: product_description,
                    image: product_image,
                    price: product_price,
                    unit: product_unit,
                    category: CategoryN {
                        uuid: category_uuid,
                        name: category_name,
                    },
                },
            }],
        };
        order_res.push(ordn);
    }

    // Map the results to OrderN
    //calling into_iter() moves the values so i used for loop
    /*orders_vec
        .into_iter()
        .map(
            |(
                order_uuid,
                order_created_on,
                order_fulfilled_on,
                order_delivery_location,
                order_delivery_status,
                order_total_price,
                (customer_uuid, customer_first_name, customer_last_name, customer_phone_number),
                (
                    order_item_uuid,
                    order_item_quantity,
                    order_item_price,
                    (
                        product_uuid,
                        product_name,
                        product_description,
                        product_image,
                        product_price,
                        product_unit,
                        (category_uuid, category_name),
                    ),
                ),
            )| {
                OrderN {
                    uuid: order_uuid,
                    created_on: order_created_on,
                    fulfilled_on: order_fulfilled_on,
                    delivery_location: order_delivery_location,
                    delivery_status: order_delivery_status,
                    total_price: order_total_price,
                    customer: CustomerN {
                        uuid: customer_uuid,
                        first_name: customer_first_name,
                        last_name: customer_last_name,
                        phone_number: customer_phone_number,
                    },
                    order_items: vec![OrderItemsN {
                        uuid: order_item_uuid,
                        quantity: order_item_quantity,
                        price: order_item_price,
                        product: ProductN {
                            uuid: product_uuid,
                            name: product_name,
                            description: product_description,
                            image: product_image,
                            price: product_price,
                            unit: product_unit,
                            category: CategoryN {
                                uuid: category_uuid,
                                name: category_name,
                            },
                        },
                    }],
                }
            },
        )
        .collect::<Vec<OrderN>>();
    */
    HttpResponse::Ok()
        .status(StatusCode::OK)
        .json(serde_json::json!({"orders": order_res}))
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
