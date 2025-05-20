use ::uuid::Uuid;
use actix_web::{get, http::StatusCode, patch, post, put, web, HttpResponse, Responder};
use diesel::prelude::*;

use crate::{
    base_types::{
        delivery_status::DeliveryStatus, order_status::OrderStatus, payment_method::PaymentMethod,
    },
    contracts::order::{
        CartCheckout, CategoryResponse, Order, OrderCreate, OrderDeliveryStatus, OrderEdit,
        OrderItemResponse, OrderResponse, OrderStatus as OrderStatusUpdate, ProductResponse,
        UserOrderResponse, UserResponse,
    },
    db::connection::{get_conn, SqliteConnectionPool},
    models::{
        cart::Cart as CartModel, invoice::{Invoice, NewInvoice}, invoice_item::NewInvoiceItem, order::{NewOrder, Order as OrderModel}, order_item::NewOrderItem as NewOrderItemModel, payment::{NewPayment, Payment as PaymentModel}, product::Product as ProductModel, shipment::NewShipment, user::User as UserModel
    },
    utils::uuid_validator::{self},
};

#[get("")]
pub async fn get_orders(pool: web::Data<SqliteConnectionPool>) -> impl Responder {
    //get a pooled connection from db
    let conn = &mut get_conn(&pool);

    use crate::schema::categories::dsl::*;
    use crate::schema::order_items::dsl::*;
    use crate::schema::orders::dsl::*;
    use crate::schema::products::dsl::*;
    use crate::schema::users::dsl::*;
    use crate::schema::{categories, order_items, orders, products, users};

    type OrderTuple = (
        String,
        String,
        String,
        f64,
        String,
        String,
        f64,
        String,
        (String, String, String, String, String, String),
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
        .inner_join(users.on(user_id.eq(users::id)))
        .inner_join(order_items.on(order_id.eq(orders::id)))
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

    // Map the results to OrderN
    let orders_vec: Vec<OrderResponse> = orders_vec
        .into_iter()
        .map(
            |(
                order_uuid,
                order_created_on,
                order_fulfilled_on,
                order_delivery_charge,
                order_delivery_location,
                order_delivery_status,
                order_total_price,
                order_status,
                (user_uuid, user_first_name, user_last_name, user_phone_number, user_email, utype),
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
                OrderResponse {
                    uuid: order_uuid,
                    created_on: order_created_on,
                    fulfilled_on: order_fulfilled_on,
                    delivery_charge: order_delivery_charge,
                    delivery_location: order_delivery_location,
                    delivery_status: order_delivery_status,
                    total_price: order_total_price,
                    status: order_status,
                    customer: UserResponse {
                        uuid: user_uuid,
                        first_name: user_first_name,
                        last_name: user_last_name,
                        phone_number: user_phone_number,
                        email: user_email,
                        user_type: utype,
                    },
                    order_items: vec![OrderItemResponse {
                        uuid: order_item_uuid,
                        quantity: order_item_quantity,
                        price: order_item_price,
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
                    }],
                }
            },
        )
        .collect::<Vec<OrderResponse>>();

    HttpResponse::Ok()
        .status(StatusCode::OK)
        .json(serde_json::json!({"orders": orders_vec}))
}

#[get("/{ord_id}")]
pub async fn get_order(
    ord_id: web::Path<(String,)>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    let ord_id: String = ord_id.into_inner().0;

    //check if the order id is a valid uuid
    let ord_id: Uuid = match Uuid::parse_str(&ord_id) {
        Ok(o) => o,
        Err(_) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Invalid order id"}))
        }
    };

    use crate::schema::categories::dsl::*;
    use crate::schema::order_items::dsl::*;
    use crate::schema::orders::dsl::*;
    use crate::schema::products::dsl::*;
    use crate::schema::users::dsl::*;
    use crate::schema::{categories, order_items, orders, products, users};

    //get a pooled connection from db
    let conn = &mut get_conn(&pool);

    type OrderTuple = (
        String,
        String,
        String,
        f64,
        String,
        String,
        f64,
        String,
        (String, String, String, String, String, String),
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

    match orders
        .inner_join(users.on(user_id.eq(users::id)))
        .inner_join(order_items.on(order_id.eq(orders::id)))
        .inner_join(products.on(order_items::product_id.eq(products::id)))
        .inner_join(categories.on(products::category_id.eq(categories::id)))
        .filter(orders::uuid.eq(&ord_id.to_string()))
        .select((
            orders::uuid,
            orders::created_on,
            orders::fulfilled_on,
            orders::delivery_charge,
            orders::delivery_location,
            orders::delivery_status,
            orders::total_price,
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
        .first::<OrderTuple>(conn)
        .optional()
    {
        Ok(Some(o)) => {
            let (
                order_uuid,
                order_created_on,
                order_fulfilled_on,
                order_delivery_charge,
                order_delivery_location,
                order_delivery_status,
                order_total_price,
                order_status,
                (user_uuid, user_first_name, user_last_name, user_phone_number, user_email, utype),
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
            ) = o;
            let ord_res: OrderResponse = OrderResponse {
                uuid: order_uuid,
                created_on: order_created_on,
                fulfilled_on: order_fulfilled_on,
                delivery_charge: order_delivery_charge,
                delivery_location: order_delivery_location,
                delivery_status: order_delivery_status,
                total_price: order_total_price,
                status: order_status,
                customer: UserResponse {
                    uuid: user_uuid,
                    first_name: user_first_name,
                    last_name: user_last_name,
                    phone_number: user_phone_number,
                    email: user_email,
                    user_type: utype,
                },
                order_items: vec![OrderItemResponse {
                    uuid: order_item_uuid,
                    quantity: order_item_quantity,
                    price: order_item_price,
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
                }],
            };
            HttpResponse::Ok().status(StatusCode::OK).json(ord_res)
        }
        Ok(None) => HttpResponse::NotFound()
            .status(StatusCode::NOT_FOUND)
            .json(serde_json::json!({"message": "Order not found"})),
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
    let cust_id: String = cust_id.into_inner().0;

    //first validate the user exists or not
    //before that lets check whether the provided user id is a valid guid or not
    let cust_id: Uuid = match Uuid::parse_str(&cust_id) {
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

    //get a pooled connection from db
    let conn = &mut get_conn(&pool);

    //find the user for the provided user id
    let cust: UserModel = match users
        .filter(users::uuid.eq(cust_id.to_string()))
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
        String,
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

    match OrderModel::belonging_to(&cust)
            .inner_join(order_items.on(order_id.eq(orders::id)))
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
                orders::status,
                (
                    order_items::uuid,
                    order_items::quantity,
                    order_items::price,
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
            .optional() {
                Ok(Some(ords)) => {
                    let user_orders: Vec<UserOrderResponse> = ords.into_iter().map(
                        |(
                            order_uuid,
                            order_created_on,
                            order_fulfilled_on,
                            order_delivery_charge,
                            order_delivery_location,
                            order_delivery_status,
                            order_total_price,
                            order_status,
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
                            UserOrderResponse {
                                uuid: order_uuid,
                                created_on: order_created_on,
                                fulfilled_on: order_fulfilled_on,
                                delivery_charge: order_delivery_charge,
                                delivery_location: order_delivery_location,
                                delivery_status: order_delivery_status,
                                total_price: order_total_price,
                                status: order_status,
                                order_items: vec![OrderItemResponse {
                                    uuid: order_item_uuid,
                                    quantity: order_item_quantity,
                                    price: order_item_price,
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
                                }],
                            }
                        },
                    )
                    .collect::<Vec<UserOrderResponse>>();
                HttpResponse::Ok().status(StatusCode::OK).json(serde_json::json!({"orders": user_orders}))
            
                },
                    Ok(None) => {
                        return HttpResponse::NotFound()
                            .status(StatusCode::NOT_FOUND)
                            .json(serde_json::json!({"message": "Order not found. Looks user hasn't ordered anything yet"}))
                    }
                    Err(_) => {
                        return HttpResponse::InternalServerError()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .json(serde_json::json!({"message": "Ops! something went wrong"}))
                    }
            }
}

#[post("")]
pub async fn create(
    order_json: web::Json<OrderCreate>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    // Validate the user UUID
    let user_uuid = match uuid_validator::validate_uuid(&order_json.user_id) {
        Ok(uid) => uid,
        Err(e) => return e,
    };

    let pay_method: PaymentMethod = match PaymentMethod::from_str(&order_json.payment_method) {
        Ok(pm) => pm,
        Err(e) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": e}))
        }
    };

    use crate::schema::order_items::dsl::*;
    use crate::schema::orders::dsl::*;
    use crate::schema::products::dsl::*;
    use crate::schema::payments::dsl::*;
    use crate::schema::shipments::dsl::*;
    use crate::schema::users::dsl::*;
    use crate::schema::{products, users};

    //get a pooled connection from db
    let conn = &mut get_conn(&pool);

    // Validate user existence
    let mut order_total: f64 = 0.0;
    let mut order_quantity: f64 = 0.0;
    for od in &order_json.order_items {
        order_total += od.price;
        order_quantity += od.quantity;
    }

    if order_total != order_json.total_price - 100.0 {
        //the 100 is delivery charge which is hard coded for now.
        return HttpResponse::BadRequest()
            .status(StatusCode::BAD_REQUEST)
            .json(serde_json::json!({"message": "Order data tempered. Price of items and order total do not match"}));
    }

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

    // Create a new order
    let new_order = NewOrder::new(
        &user,
        order_json.created_on.to_owned(),
        order_json.delivery_charge,
        DeliveryStatus::Pending,
        order_json.delivery_location.to_owned(),
        order_total,
        order_quantity,
        OrderStatus::PaymentPending,
    );

    // Insert the order and order details in a transaction
    match conn.transaction::<HttpResponse, diesel::result::Error, _>(|con| {
        let mut prods = vec![];
        let order: OrderModel = diesel::insert_into(orders)
            .values(&new_order)
            .get_result(con)?;

        for order_item in &order_json.order_items {
            let product: ProductModel = match products
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

            let new_order_item: NewOrderItemModel =
                NewOrderItemModel::new(order_item.quantity, &product, &order);

            diesel::insert_into(order_items)
                .values(&new_order_item)
                .execute(con)?;

            //update the product stock if order creation successful
            diesel::update(&product)
                .set(products::stock.eq(products::stock - order_item.quantity))
                .execute(con)?;

            prods.push(product);
        }

        // If payment method is cash create shipment and invoice else create from payment
        if pay_method == PaymentMethod::Cash {
            let user_location = user.get_location().unwrap_or("");
            let shipment: NewShipment = NewShipment::new(user_location, &order);
            diesel::insert_into(shipments)
                .values(&shipment)
                .execute(con)?;

            //create payment
            let tran_id = Uuid::new_v4().to_string().replace("-", "");
            let new_payment: NewPayment = NewPayment::new(
                &pay_method,
                &tran_id,
                &user,
                &order,
                order.get_total_price(),
                order.get_total_price(),
            ); //amount  and tendered same same

            let payment = diesel::insert_into(payments)
                .values(&new_payment)
                .get_result::<PaymentModel>(con)?;

            //update order status to payment completed
            diesel::update(&order)
                .set(status.eq(OrderStatus::Processed.value()))
                .execute(con)?;

            //create invoice
            let inv_date = chrono::Utc::now().naive_utc().format("%Y-%m-%d %H:%M:%S").to_string();
            let new_inv: NewInvoice = NewInvoice::new(
                &inv_date,
                order.get_total_price(),
                0.0, //vat percent
                &order,
                &user,
                &payment
            );

            let inv = diesel::insert_into(crate::schema::invoices::table)
                .values(&new_inv)
                .get_result::<Invoice>(con)?;

            for prod in prods {
                let new_inv_item: NewInvoiceItem = NewInvoiceItem::new(
                    &prod,
                    &inv,
                    prod.get_price(),
                    0.0, //discount percent
                    0.0, //discount amount
                );

                diesel::insert_into(crate::schema::invoice_items::table)
                    .values(&new_inv_item)
                    .execute(con)?;
            }
        }

        let order_vm: Order = Order {
            user_id: user_uuid.to_string(),
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
        Ok(response) => response,
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "message": "Failed to process order transaction"
        })),
    }
}

#[post("/cart/create-order")]
pub async fn create_orders_from_cart(
    carts_json: web::Json<CartCheckout>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    for cart in &carts_json.cart_ids {
        match uuid_validator::validate_uuid(cart) {
            Ok(_) => (),
            Err(res) => return res,
        };
    }

    let user_uuid = match uuid_validator::validate_uuid(&carts_json.user_id) {
        Ok(uid) => uid,
        Err(e) => return e,
    };

    use crate::schema::carts::dsl::*;
    use crate::schema::order_items::dsl::*;
    use crate::schema::orders::dsl::*;
    use crate::schema::products::dsl::*;
    use crate::schema::users::dsl::*;
    use crate::schema::{carts, products, users};

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

            order_items_vec.push((cart, product));
        }

        let nepal_time = chrono::Utc::now()
            .with_timezone(&chrono::FixedOffset::east_opt(5 * 3600 + 45 * 60).unwrap());
        let formatted_time = nepal_time.format("%Y-%m-%d %H:%M:%S").to_string();

        let order = NewOrder::new(
            &user,
            formatted_time,
            100.0, //delivery charge
            DeliveryStatus::Pending,
            user.get_location().unwrap().to_owned(),
            order_total_price,
            order_total_quantity,
            OrderStatus::PaymentPending,
        );

        let inserted_order: OrderModel =
            diesel::insert_into(orders).values(&order).get_result(con)?;

        // Now insert all order items
        for (cart, product) in order_items_vec {
            let new_order_item =
                NewOrderItemModel::new(cart.get_quantity(), &product, &inserted_order);

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

    //first validate the user exists or not
    //before that lets check whether the provided user id is a valid guid or not
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
