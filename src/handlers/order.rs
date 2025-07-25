use std::{collections::HashMap, sync::Arc};

use ::uuid::Uuid;
use actix_web::{get, http::StatusCode, patch, post, put, web, HttpResponse, Responder};
use diesel::prelude::*;

use crate::{
    base_types::{
        delivery_status::DeliveryStatus, order_status::OrderStatus, payment_method::PaymentMethod,
        payment_status::PaymentStatus,
    },
    contracts::order::{
        AllOrderResponse1, CartCheckout, CategoryResponse, DateFilterParams, Order, OrderCreate,
        OrderDeliveryStatus, OrderEdit, OrderItemResponse, OrderResponse,
        OrderStatus as OrderStatusUpdate, PaymentResponse, ProductResponse, ShipmentResponse,
        UserOrderResponse, UserResponse,
    },
    db::connection::{get_conn, SqliteConnectionPool},
    models::{
        cart::Cart as CartModel,
        invoice::{Invoice, NewInvoice},
        invoice_item::NewInvoiceItem,
        order::{NewOrder, Order as OrderModel},
        order_item::NewOrderItem as NewOrderItemModel,
        payment::{NewPayment, Payment as PaymentModel},
        product::Product as ProductModel,
        shipment::NewShipment,
        user::User as UserModel,
    },
    services::notification_service::{NewOrderPayload, NotificationEvent, NotificationService},
    utils::uuid_validator,
};

pub const DELIVERY_CHARGE: f64 = 100.0;

#[get("")]
pub async fn get_orders(
    filters: web::Query<DateFilterParams>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    let conn = &mut get_conn(&pool);

    let final_date = filters.final_date.clone().unwrap_or_else(|| {
        let dt = chrono::Utc::now()
            .with_timezone(&chrono::FixedOffset::east_opt(5 * 3600 + 45 * 60).unwrap());
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    });

    let results = diesel::sql_query(
        r#"
        SELECT
            o.uuid,
            o.created_on,
            o.fulfilled_on,
            o.delivery_charge,
            o.delivery_location,
            o.delivery_status,
            o.total_price,
            o.discount,
            o.amount,
            o.status,
            o.quantity,
            p.unit,
            p.image,
            p.name
        FROM orders o
        JOIN order_items oi ON oi.order_id = o.id
        JOIN products p ON p.id = oi.product_id
        WHERE (o.id, oi.product_id) IN (
            SELECT order_id, MIN(product_id)
            FROM order_items
            GROUP BY order_id
        )
        AND o.created_on BETWEEN ? AND ?
        ORDER BY o.created_on;
    "#,
    )
    .bind::<diesel::sql_types::Text, _>(filters.init_date.clone())
    .bind::<diesel::sql_types::Text, _>(final_date)
    .load::<AllOrderResponse1>(conn);

    match results {
        Ok(r) =>
            HttpResponse::Ok()
                .status(StatusCode::OK)
                .json(serde_json::json!({"orders": r})),
    Err(e) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message": format!("Ops! something went wrong when fetching orders: {}", e)}))
    }
}

#[get("/count")]
pub async fn get_orders_count(
    filters: web::Query<DateFilterParams>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    use diesel::dsl::count;

    //get a pooled connection from db
    let conn = &mut get_conn(&pool);

    use crate::schema::order_items::dsl::*;
    use crate::schema::orders::dsl::*;
    use crate::schema::products::dsl::*;
    use crate::schema::{order_items, orders, products};

    let final_date = filters.final_date.clone().unwrap_or_else(|| {
        let dt = chrono::Utc::now()
            .with_timezone(&chrono::FixedOffset::east_opt(5 * 3600 + 45 * 60).unwrap());
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    });

    let count = orders
        .inner_join(order_items.on(order_items::order_id.eq(orders::id)))
        .inner_join(products.on(order_items::product_id.eq(products::id)))
        .filter(orders::created_on.between(&filters.init_date, &final_date))
        .order_by(orders::created_on)
        .select(count(orders::id))
        .first::<i64>(conn)
        .expect("Error counting orders");

    HttpResponse::Ok()
        .status(StatusCode::OK)
        .json(serde_json::json!({"count": count}))
}

#[get("/{ord_id}")]
pub async fn get_order(
    ord_id: web::Path<(String,)>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    let ord_id: String = ord_id.into_inner().0;

    let ord_id: Uuid = match Uuid::parse_str(&ord_id) {
        Ok(o) => o,
        Err(_) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Invalid order id"}))
        }
    };

    use crate::schema::categories::dsl::*;
    use crate::schema::invoices::dsl::*;
    use crate::schema::order_items::dsl::*;
    use crate::schema::orders::dsl::*;
    use crate::schema::payments::dsl::*;
    use crate::schema::products::dsl::*;
    use crate::schema::shipments::dsl::*;
    use crate::schema::users::dsl::*;
    use crate::schema::{
        categories, invoices, order_items, orders, payments, products, shipments, users,
    };

    let conn = &mut get_conn(&pool);

    type OrderTuple = (
        String,
        String,
        String,
        f64,
        String,
        String,
        f64,
        f64,
        f64,
        String,
        (String, String, String, String, String, String),
        (
            String,
            f64,
            f64,
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
        Option<(String, String, String, f64, String)>,
        Option<(String, String, String)>,
        Option<String>,
    );

    let result = orders
        .inner_join(users.on(orders::user_id.eq(users::id)))
        .inner_join(order_items.on(order_items::order_id.eq(orders::id)))
        .inner_join(products.on(order_items::product_id.eq(products::id)))
        .inner_join(categories.on(products::category_id.eq(categories::id)))
        .left_join(payments.on(payments::order_id.eq(orders::id)))
        .left_join(shipments.on(shipments::order_id.eq(orders::id)))
        .left_join(invoices.on(invoices::order_id.eq(orders::id)))
        .filter(orders::uuid.eq(ord_id.to_string()))
        .select((
            orders::uuid,
            orders::created_on,
            orders::fulfilled_on,
            orders::delivery_charge,
            orders::delivery_location,
            orders::delivery_status,
            orders::total_price,
            orders::discount,
            orders::amount,
            orders::status,
            (
                users::uuid,
                users::first_name,
                users::last_name,
                users::phone_number,
                users::email,
                users::user_type,
            ),
            (
                order_items::uuid,
                order_items::quantity,
                order_items::price,
                order_items::discount,
                order_items::amount,
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
            (
                payments::uuid,
                payments::payment_method,
                payments::transaction_id,
                payments::amount,
                payments::status,
            )
                .nullable(),
            (shipments::uuid, shipments::status, shipments::ship_date).nullable(),
            invoices::uuid.nullable(),
        ))
        .load::<OrderTuple>(conn);

    match result {
        Ok(order_rows) if order_rows.is_empty() => HttpResponse::NotFound()
            .status(StatusCode::NOT_FOUND)
            .json(serde_json::json!({"message": "Order not found"})),
        Ok(order_rows) => {
            let mut map: HashMap<String, OrderResponse> = HashMap::new();

            for (
                order_uuid,
                order_created_on,
                order_fulfilled_on,
                order_delivery_charge,
                order_delivery_location,
                order_delivery_status,
                order_total_price,
                order_total_discount,
                order_total_amount,
                order_status,
                (user_uuid, user_first_name, user_last_name, user_phone_number, user_email, utype),
                (
                    order_item_uuid,
                    order_item_quantity,
                    order_item_price,
                    order_item_discount,
                    order_item_amount,
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
                payment_opt,
                shipment_opt,
                invoice_uuid,
            ) in order_rows
            {
                let entry = map
                    .entry(order_uuid.clone())
                    .or_insert_with(|| OrderResponse {
                        uuid: order_uuid,
                        created_on: order_created_on,
                        fulfilled_on: order_fulfilled_on,
                        delivery_charge: order_delivery_charge,
                        delivery_location: order_delivery_location,
                        delivery_status: order_delivery_status,
                        total_price: order_total_price,
                        discount: order_total_discount,
                        amount: order_total_amount,
                        status: order_status,
                        user: UserResponse {
                            uuid: user_uuid,
                            first_name: user_first_name,
                            last_name: user_last_name,
                            phone_number: user_phone_number,
                            email: user_email,
                            user_type: utype,
                        },
                        order_items: vec![],
                        payment: payment_opt.map(|(pay_uuid, method, tran_id, amt, pay_status)| {
                            PaymentResponse {
                                uuid: pay_uuid,
                                payment_method: method,
                                transaction_id: tran_id,
                                amount: amt,
                                status: pay_status,
                            }
                        }),
                        shipment: shipment_opt.map(|(ship_uuid, ship_status, ship_dt)| {
                            ShipmentResponse {
                                uuid: ship_uuid,
                                status: ship_status,
                                ship_date: ship_dt,
                            }
                        }),
                        invoice_id: invoice_uuid,
                    });

                entry.order_items.push(OrderItemResponse {
                    uuid: order_item_uuid,
                    quantity: order_item_quantity,
                    price: order_item_price,
                    discount: order_item_discount,
                    amount: order_item_amount,
                    product: ProductResponse {
                        uuid: product_uuid,
                        name: product_name,
                        description: product_description,
                        image: product_image,
                        price: product_price,
                        unit: product_unit,
                        category: CategoryResponse {
                            uuid: category_uuid,
                            name: category_name,
                        },
                    },
                });
            }

            // Since we're filtering by a single order ID, there should be only one entry
            let response = map.into_iter().next().unwrap().1;

            HttpResponse::Ok()
                .status(StatusCode::OK)
                .json(serde_json::json!({"order": response}))
        }
        Err(_) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message": "Ops! something went wrong"})),
    }
}

#[get("/user/{cust_id}")]
pub async fn get_user_orders(
    cust_id: web::Path<(String,)>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    let cust_id_str: String = cust_id.into_inner().0;

    let cust_id_uuid: Uuid = match Uuid::parse_str(&cust_id_str) {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Invalid user id"}));
        }
    };

    use crate::schema::categories::dsl::*;
    use crate::schema::order_items::dsl::*;
    use crate::schema::products::dsl::*;
    use crate::schema::users::dsl::*;
    use crate::schema::{categories, order_items, orders, products, users};

    let conn = &mut get_conn(&pool);

    let cust: UserModel = match users
        .filter(users::uuid.eq(cust_id_uuid.to_string()))
        .select(UserModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(Some(c)) => c,
        Ok(None) => {
            return HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "User not found"}));
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}));
        }
    };

    type OrderTuple = (
        String,
        String,
        String,
        f64,
        String,
        String,
        f64,
        f64,
        f64,
        String,
        (
            String,
            f64,
            f64,
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

    match OrderModel::belonging_to(&cust)
        .inner_join(order_items.on(order_items::order_id.eq(orders::id))) // Fix: Use order_items::order_id
        .inner_join(products.on(order_items::product_id.eq(products::id)))
        .inner_join(categories.on(products::category_id.eq(categories::id)))
        .select((
            orders::uuid,
            orders::created_on,
            orders::fulfilled_on,
            orders::delivery_charge,
            orders::delivery_location,
            orders::delivery_status,
            orders::total_price,
            orders::discount,
            orders::amount,
            orders::status,
            (
                order_items::uuid,
                order_items::quantity,
                order_items::price,
                order_items::discount,
                order_items::amount,
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
    {
        Ok(ords) => {
            let mut grouped_orders: HashMap<String, UserOrderResponse> = HashMap::new();

            for (
                order_uuid,
                order_created_on,
                order_fulfilled_on,
                order_delivery_charge,
                order_delivery_location,
                order_delivery_status,
                order_total_price,
                order_total_discount,
                order_total_amount,
                order_status,
                (
                    order_item_uuid,
                    order_item_quantity,
                    order_item_price,
                    order_item_discount,
                    order_item_amount,
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
            ) in ords.into_iter()
            {
                let order_item = OrderItemResponse {
                    uuid: order_item_uuid,
                    quantity: order_item_quantity,
                    price: order_item_price,
                    discount: order_item_discount,
                    amount: order_item_amount,
                    product: ProductResponse {
                        uuid: product_uuid,
                        name: product_name,
                        description: product_description,
                        image: product_image,
                        price: product_price,
                        unit: product_unit,
                        category: CategoryResponse {
                            uuid: category_uuid,
                            name: category_name,
                        },
                    },
                };

                let order_entry =
                    grouped_orders
                        .entry(order_uuid.clone())
                        .or_insert_with(|| UserOrderResponse {
                            uuid: order_uuid,
                            created_on: order_created_on,
                            fulfilled_on: order_fulfilled_on,
                            delivery_charge: order_delivery_charge,
                            delivery_location: order_delivery_location,
                            delivery_status: order_delivery_status,
                            total_price: order_total_price,
                            discount: order_total_discount,
                            amount: order_total_amount,
                            status: order_status,
                            order_items: Vec::new(),
                        });
                order_entry.order_items.push(order_item);
            }

            let user_orders: Vec<UserOrderResponse> = grouped_orders.into_values().collect();

            if user_orders.is_empty() {
                return HttpResponse::NotFound().status(StatusCode::NOT_FOUND).json(
                    serde_json::json!({"message": "Order not found. Looks user hasn't ordered anything yet"}),
                );
            }

            HttpResponse::Ok()
                .status(StatusCode::OK)
                .json(serde_json::json!({"orders": user_orders}))
        }
        Err(diesel::result::Error::NotFound) => {
            // This case handles if belong_to returns no orders for the user
            return HttpResponse::NotFound().status(StatusCode::NOT_FOUND).json(
               serde_json::json!({"message": "Order not found. Looks user hasn't ordered anything yet"}),
            );
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}));
        }
    }
}

#[post("")]
pub async fn create(
    order_json: web::Json<OrderCreate>,
    pool: web::Data<SqliteConnectionPool>,
    notification_service: web::Data<Arc<dyn NotificationService>>,
) -> impl Responder {
    use crate::schema::{
        invoice_items, invoices, order_items, orders, payments, products, shipments, users,
    };

    let user_uuid = match uuid_validator::validate_uuid(&order_json.user_id) {
        Ok(uid) => uid,
        Err(e) => return e,
    };

    let pay_method = match PaymentMethod::from_str(&order_json.payment.payment_method) {
        Ok(pm) => pm,
        Err(e) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": e.to_string()}))
        }
    };

    if order_json.order_items.is_empty() {
        return HttpResponse::BadRequest()
            .status(StatusCode::BAD_REQUEST)
            .json(serde_json::json!({"message": "Invalid order creation request. Order must containt order items"}));
    }

    let conn = &mut get_conn(&pool);

    let mut created_order_id: String = String::new();

    let (order_total, order_quantity, order_discount) =
        order_json
            .order_items
            .iter()
            .fold((0.0, 0.0, 0.0), |(total, quantity, discount), od| {
                (
                    total + od.price,
                    quantity + od.quantity,
                    discount + od.discount,
                )
            });

    if (order_total + DELIVERY_CHARGE) != order_json.total_price {
        return HttpResponse::BadRequest()
            .status(StatusCode::BAD_REQUEST)
            .json(serde_json::json!({"message": "Order data tempered. Price of items and order total do not match"}));
    }

    let user: UserModel = match users::table
        .filter(users::uuid.eq(user_uuid.to_string()))
        .select(UserModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(Some(c)) => c,
        Ok(None) => {
            return HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "User not found"}));
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}));
        }
    };

    match conn.transaction::<HttpResponse, diesel::result::Error, _>(|con| {
        let new_order = NewOrder::new(
            &user,
            &order_json.created_on,
            order_json.delivery_charge,
            DeliveryStatus::Pending,
            &order_json.delivery_location,
            order_total,
            order_quantity,
            OrderStatus::PaymentPending,
            order_discount,
        );

        let order: OrderModel = diesel::insert_into(orders::table)
            .values(&new_order)
            .get_result(con)?;

        created_order_id = order.get_uuid().to_string().clone();

        let shipment = NewShipment::new(&order_json.delivery_location, &order);
        diesel::insert_into(shipments::table)
            .values(&shipment)
            .execute(con)?;

        let tran_id = if order_json.payment.payment_method == PaymentMethod::Cash.value() {
            Uuid::new_v4().to_string().replace('-', "")
        } else {
            String::new()
        };

        let new_payment = NewPayment::new(
            &pay_method,
            &tran_id,
            &user,
            &order,
            order.get_total_price(),
            order.get_total_price(),
            PaymentStatus::Pending.value().to_string(),
        );

        let payment = diesel::insert_into(payments::table)
            .values(&new_payment)
            .get_result::<PaymentModel>(con)?;

        diesel::update(&order)
            .set(orders::status.eq(OrderStatus::Processed.value()))
            .execute(con)?;

        let nepal_time = chrono::Utc::now()
            .with_timezone(&chrono::FixedOffset::east_opt(5 * 3600 + 45 * 60).unwrap());
        let inv_date = nepal_time.format("%Y-%m-%d %H:%M:%S").to_string();

        let new_inv = NewInvoice::new(
            &inv_date,
            order.get_total_price(),
            0.0,
            &order,
            &user,
            &payment,
        );

        let inv = diesel::insert_into(invoices::table)
            .values(&new_inv)
            .get_result::<Invoice>(con)?;

        for order_item in &order_json.order_items {
            let product: ProductModel = match products::table
                .filter(products::uuid.eq(&order_item.product_id))
                .select(ProductModel::as_select())
                .first(con)
                .optional()?
            {
                Some(p) => p,
                None => {
                    return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                        "message": "Product not found for order"
                    })));
                }
            };

            if product.get_stock() < order_item.quantity {
                return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "message": "Product out of stock"
                })));
            }

            let new_order_item =
                NewOrderItemModel::new(order_item.quantity, order_item.discount, &product, &order);
            diesel::insert_into(order_items::table)
                .values(&new_order_item)
                .execute(con)?;

            diesel::update(&product)
                .set(products::stock.eq(products::stock - order_item.quantity))
                .execute(con)?;

            let new_inv_item = NewInvoiceItem::new(&product, &inv, product.get_price(), 0.0, 0.0);

            diesel::insert_into(invoice_items::table)
                .values(&new_inv_item)
                .execute(con)?;
        }

        let order_vm = Order {
            user_id: format!("{};{}", user_uuid.to_string(), user.get_fullname()),
            created_on: order.get_created_on().to_owned(),
            fulfilled_on: order.get_fulfilled_on().to_owned(),
            total_price: order.get_total_price(),
            uuid: order.get_uuid().to_string(),
            delivery_charge: order.get_delivery_charge(),
            delivery_location: order.get_delivery_location().to_owned(),
            delivery_status: order.get_delivery_status().to_owned(),
        };

        Ok(HttpResponse::Ok()
            .status(StatusCode::OK)
            .json(serde_json::json!({"order": order_vm})))
    }) {
        Ok(response) => {
            if PaymentMethod::from_str(&order_json.payment.payment_method).unwrap()
                != PaymentMethod::Cash
            {
                return response;
            }

            let order_created_payload = NewOrderPayload {
                order_id: created_order_id.clone(),
                customer_name: user.get_fullname(),
                total_amount: order_json.total_price,
            };

            tokio::spawn(async move {
                match notification_service
                    .send_notification(NotificationEvent::NewOrder(order_created_payload))
                    .await
                {
                    Ok(_) => println!(
                        "Notification for order {} dispatched successfully.",
                        created_order_id
                    ),
                    Err(e) => eprintln!(
                        "Failed to dispatch notification for order {}: {:?}",
                        created_order_id, e
                    ),
                }
            });

            response
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "message": format!("Failed to process order transaction: {}", e)
        })),
    }
}

#[post("/cart/create-order")]
pub async fn create_orders_from_cart(
    carts_json: web::Json<CartCheckout>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    let pay_method: PaymentMethod = match PaymentMethod::from_str(&carts_json.payment_method) {
        Ok(pm) => pm,
        Err(e) => {
            return HttpResponse::BadRequest().json(serde_json::json!({"message": e.to_string()}))
        }
    };

    let user_uuid = match uuid_validator::validate_uuid(&carts_json.user_id) {
        Ok(uid) => uid,
        Err(e) => return e,
    };

    for cart in &carts_json.cart_ids {
        match uuid_validator::validate_uuid(cart) {
            Ok(_) => (),
            Err(res) => return res,
        };
    }

    use crate::schema::carts::dsl::*;
    use crate::schema::order_items::dsl::*;
    use crate::schema::orders::dsl::*;
    use crate::schema::products::dsl::*;
    use crate::schema::users::dsl::*;
    use crate::schema::{carts, invoice_items, invoices, payments, products, shipments, users};

    //get a pooled connection from db
    let conn = &mut get_conn(&pool);

    //validate user exists
    let user: UserModel = match users
        .filter(users::uuid.eq(user_uuid.to_string()))
        .select(UserModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(Some(c)) => c,
        Ok(None) => {
            return HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "User not found"}));
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}));
        }
    };

    if user.get_location().is_none() {
        return HttpResponse::BadRequest().json(serde_json::json!({
            "message": "User's location is missing. Please update location"
        }));
    }

    // validate cart items exists
    let result = conn.transaction::<HttpResponse, diesel::result::Error, _>(|con| {
        let mut order_items_vec: Vec<(CartModel, ProductModel)> = vec![];
        let mut order_total_price = 0.0;
        let mut order_total_quantity = 0.0;
        let mut order_total_discount = 0.0;

        for cart_id in &carts_json.cart_ids {
            let cart: CartModel = match carts
                .filter(carts::uuid.eq(&cart_id))
                .first::<CartModel>(con)
                .optional()?
            {
                Some(c) => c,
                None => {
                    return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                        "message": format!("Cart not found: {}", cart_id)
                    })));
                }
            };

            let product: ProductModel = match products
                .filter(products::uuid.eq(&cart.get_uuid()))
                .first::<ProductModel>(con)
                .optional()?
            {
                Some(p) => p,
                None => {
                    return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                        "message": "Product not found for order"
                    })));
                }
            };

            if product.get_stock() < cart.get_quantity() {
                return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "message": "Product out of stock"
                })));
            }

            order_total_price += cart.get_quantity() as f64 * product.get_price();
            order_total_quantity += cart.get_quantity();
            order_total_discount += cart.get_discount();

            order_items_vec.push((cart, product));
        }

        let nepal_time = chrono::Utc::now()
            .with_timezone(&chrono::FixedOffset::east_opt(5 * 3600 + 45 * 60).unwrap());
        let formatted_time = nepal_time.format("%Y-%m-%d %H:%M:%S").to_string();

        let order = NewOrder::new(
            &user,
            &formatted_time,
            DELIVERY_CHARGE, //delivery charge
            DeliveryStatus::Pending,
            &user.get_location().unwrap().to_owned(),
            order_total_price,
            order_total_quantity,
            OrderStatus::PaymentPending,
            order_total_discount,
        );

        let inserted_order: OrderModel =
            diesel::insert_into(orders).values(&order).get_result(con)?;

        if pay_method == PaymentMethod::Cash {
            let user_location = user.get_location().unwrap_or_default();
            let shipment = NewShipment::new(user_location, &inserted_order);

            diesel::insert_into(shipments::table)
                .values(&shipment)
                .execute(con)?;

            let tran_id = Uuid::new_v4().to_string().replace('-', "");
            let new_payment = NewPayment::new(
                &pay_method,
                &tran_id,
                &user,
                &inserted_order,
                inserted_order.get_total_price(),
                inserted_order.get_total_price(),
                carts_json.payment_status.to_owned(),
            );

            let payment = diesel::insert_into(payments::table)
                .values(&new_payment)
                .get_result::<PaymentModel>(con)?;

            let new_invoice: NewInvoice = NewInvoice::new(
                &formatted_time,
                inserted_order.get_total_price(),
                0.0,
                &inserted_order,
                &user,
                &payment,
            );

            let inserted_inv = diesel::insert_into(invoices::table)
                .values(&new_invoice)
                .get_result::<Invoice>(con)?;

            // Now insert all order items
            for (cart, product) in &order_items_vec {
                let new_order_item = NewOrderItemModel::new(
                    cart.get_quantity(),
                    cart.get_discount(),
                    &product,
                    &inserted_order,
                );

                diesel::insert_into(order_items)
                    .values(&new_order_item)
                    .execute(con)?;

                //update the product stock if order creation successful
                diesel::update(&product)
                    .set(products::stock.eq(products::stock - cart.get_quantity()))
                    .execute(con)?;

                let inv_item =
                    NewInvoiceItem::new(&product, &inserted_inv, cart.get_quantity(), 0.0, 0.0);
                diesel::insert_into(invoice_items::table)
                    .values(&inv_item)
                    .execute(con)?;

                //delete from cart
                diesel::delete(&cart).execute(con)?;

                return Ok(HttpResponse::Ok().json(serde_json::json!({
                    "message": "Order created successfully",
                    "order_id": &inserted_order.get_uuid()
                })));
            }
        }

        // Now insert all order items
        for (cart, product) in order_items_vec {
            let new_order_item = NewOrderItemModel::new(
                cart.get_quantity(),
                cart.get_discount(),
                &product,
                &inserted_order,
            );

            diesel::insert_into(order_items)
                .values(&new_order_item)
                .execute(con)?;

            //update the product stock if order creation successful
            diesel::update(&product)
                .set(products::stock.eq(products::stock - cart.get_quantity()))
                .execute(con)?;

            //delete from cart
            diesel::delete(&cart).execute(con)?;
        }

        Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "Order created successfully",
            "order_id": inserted_order.get_uuid()
        })))
    });

    match result {
        Ok(response) => response,
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "message": "Failed to process order transaction"
        })),
    }
}

#[put("/{order_id}")]
pub async fn edit(
    order_id: web::Path<(String,)>,
    order_json: web::Json<OrderEdit>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    let order_uid: String = order_id.into_inner().0;

    let user_uuid: Uuid = match Uuid::parse_str(&order_json.user_id) {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Invalid user id"}));
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

    use crate::schema::orders::dsl::*;
    use crate::schema::users::dsl::*;
    use crate::schema::{orders, users};

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

    //find the user for the provided user id
    let user: UserModel = match users
        .filter(users::uuid.eq(user_uuid.to_string()))
        .select(UserModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(Some(c)) => c,
        Ok(None) => {
            return HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "User not found"}));
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}));
        }
    };

    match diesel::update(&order)
        .set((
            user_id.eq(user.get_id()),
            fulfilled_on.eq(&order_json.fulfilled_on),
            delivery_status.eq(&order_json.delivery_status),
            delivery_location.eq(&order_json.delivery_location),
            total_price.eq(order_json.total_price),
        ))
        .get_result::<OrderModel>(conn)
    {
        Ok(o) => {
            let order: Order = Order {
                user_id: user_uuid.to_string(),
                created_on: o.get_created_on().to_owned(),
                fulfilled_on: o.get_fulfilled_on().to_owned(),
                total_price: o.get_total_price(),
                uuid: order_uid.to_string(),
                delivery_charge: o.get_delivery_charge(),
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

#[patch("/{order_id}/order-status/update")]
pub async fn update_order_status(
    order_id: web::Path<(String,)>,
    order_status: web::Query<OrderStatusUpdate>,
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

    let order_status: OrderStatus = match OrderStatus::from_str(&order_status.status) {
        Ok(s) => s,
        Err(e) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": format!("{}", e)}))
        }
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
        .set(status.eq(order_status.value()))
        .get_result::<OrderModel>(conn)
    {
        Ok(o) => {
            let order: Order = Order {
                user_id: String::from("N/A"),
                created_on: o.get_created_on().to_owned(),
                fulfilled_on: o.get_fulfilled_on().to_owned(),
                total_price: o.get_total_price(),
                uuid: order_uid.to_string(),
                delivery_charge: o.get_delivery_charge(),
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

    let deli_status = match DeliveryStatus::from_str(&order_delivery_status.delivery_status) {
        Ok(s) => s,
        Err(e) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": format!("{}", e)}))
        }
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
        .set(delivery_status.eq(deli_status.value()))
        .get_result::<OrderModel>(conn)
    {
        Ok(o) => {
            let order: Order = Order {
                user_id: String::from("N/A"),
                created_on: o.get_created_on().to_owned(),
                fulfilled_on: o.get_fulfilled_on().to_owned(),
                total_price: o.get_total_price(),
                uuid: order_uid.to_string(),
                delivery_charge: o.get_delivery_charge(),
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
