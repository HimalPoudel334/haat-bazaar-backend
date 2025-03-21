use ::uuid::Uuid;
use actix_web::{get, http::StatusCode, patch, post, put, web, HttpResponse, Responder};
use diesel::prelude::*;

use crate::{
    base_types::delivery_status::DeliveryStatus,
    contracts::order::{
        CategoryResponse, UserOrderResponse, UserResponse, Order, OrderCreate,
        OrderDeliveryStatus, OrderEdit, OrderItemResponse, OrderResponse, ProductResponse,
    },
    db::connection::{get_conn, SqliteConnectionPool},
    models::{
        user::User as UserModel,
        order::{NewOrder, Order as OrderModel},
        order_item::NewOrderItem as NewOrderItemModel,
        product::Product as ProductModel,
    },
    utils::{self, uuid_validator::DatabaseErrorInfo},
};

#[get("")]
pub async fn get_orders(pool: web::Data<SqliteConnectionPool>) -> impl Responder {
    //get a pooled connection from db
    let conn = &mut get_conn(&pool);

    use crate::schema::categories::dsl::*;
    use crate::schema::users::dsl::*;
    use crate::schema::order_items::dsl::*;
    use crate::schema::orders::dsl::*;
    use crate::schema::products::dsl::*;
    use crate::schema::{categories, users, order_items, orders, products};

    type OrderTuple = (
        String,
        String,
        String,
        f64,
        String,
        String,
        f64,
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
            (
                users::uuid,
                users::first_name,
                users::last_name,
                users::phone_number,
                users::email,
                users::user_type
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

    // let mut order_res: Vec<OrderN> = Vec::new();
    // for (
    //     order_uuid,
    //     order_created_on,
    //     order_fulfilled_on,
    //     order_delivery_location,
    //     order_delivery_status,
    //     order_total_price,
    //     (user_uuid, user_first_name, user_last_name, user_phone_number, user_email),
    //     (
    //         order_item_uuid,
    //         order_item_quantity,
    //         order_item_price,
    //         (
    //             product_uuid,
    //             product_name,
    //             product_description,
    //             product_image,
    //             product_price,
    //             product_unit,
    //             (category_uuid, category_name),
    //         ),
    //     ),
    // ) in orders_vec
    // {
    //     let ordn = OrderN {
    //         uuid: order_uuid,
    //         created_on: order_created_on,
    //         fulfilled_on: order_fulfilled_on,
    //         delivery_location: order_delivery_location,
    //         delivery_status: order_delivery_status,
    //         total_price: order_total_price,
    //         user: UserResponse {
    //             uuid: user_uuid,
    //             first_name: user_first_name,
    //             last_name: user_last_name,
    //             phone_number: user_phone_number,
    //             email: user_email
    //         },
    //         order_items: vec![OrderItemResponse {
    //             uuid: order_item_uuid,
    //             quantity: order_item_quantity,
    //             price: order_item_price,
    //             product: ProductResponse {
    //                 uuid: product_uuid,
    //                 name: product_name,
    //                 description: product_description,
    //                 image: product_image,
    //                 price: product_price,
    //                 unit: product_unit,
    //                 category: CategoryN {
    //                     uuid: category_uuid,
    //                     name: category_name,
    //                 },
    //             },
    //         }],
    //     };
    //     order_res.push(ordn);
    // }

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
                    customer: UserResponse {
                        uuid: user_uuid,
                        first_name: user_first_name,
                        last_name: user_last_name,
                        phone_number: user_phone_number,
                        email: user_email,
                        user_type : utype
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
    use crate::schema::users::dsl::*;
    use crate::schema::order_items::dsl::*;
    use crate::schema::orders::dsl::*;
    use crate::schema::products::dsl::*;
    use crate::schema::{categories, users, order_items, orders, products};

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
            (
                users::uuid,
                users::first_name,
                users::last_name,
                users::phone_number,
                users::email,
                users::user_type
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
                customer: UserResponse {
                    uuid: user_uuid,
                    first_name: user_first_name,
                    last_name: user_last_name,
                    phone_number: user_phone_number,
                    email: user_email,
                    user_type : utype
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
    use crate::schema::users::dsl::*;
    use crate::schema::order_items::dsl::*;
    use crate::schema::products::dsl::*;
    use crate::schema::{categories, users, order_items, orders, products};

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
    let user_uuid = match utils::uuid_validator::validate_uuid(&order_json.user_id) {
        Ok(uid) => uid,
        Err(e) => return e,
    };

    use crate::schema::users::dsl::*;
    use crate::schema::order_items::dsl::*;
    use crate::schema::orders::dsl::*;
    use crate::schema::products::dsl::*;
    use crate::schema::{users, products};

    // Validate user existence
    let mut order_total: f64 = 0.0;
    (&order_json.order_items).into_iter().for_each(|od| {
        order_total += od.price;
    });

    if order_total != order_json.total_price - 100.0 {
        //the 100 is delivery charge which is hard coded for now.
        return HttpResponse::BadRequest()
            .status(StatusCode::BAD_REQUEST)
            .json(serde_json::json!({"message": "Order data tempered.\nPrice of items and order total do not match"}));
    }

    //get a pooled connection from db
    let conn = &mut get_conn(&pool);

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
        order_total
    );

    // Insert the order and order details in a transaction
    match conn.transaction::<_, diesel::result::Error, _>(|conn| {
        let order: OrderModel = diesel::insert_into(orders)
            .values(&new_order)
            .get_result(conn)?;

        for order_detail in &order_json.order_items {
            let product: ProductModel = products
                .filter(products::uuid.eq(&order_detail.product_id))
                .select(ProductModel::as_select())
                .first(conn)?;

            if product.get_stock() < order_detail.quantity {
                return Err(diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::Unknown,
                    Box::new(DatabaseErrorInfo {
                        message: "Ordered quantity is greater than product stock".into(),
                    }),
                ));
            }

            let od: NewOrderItemModel =
                NewOrderItemModel::new(order_detail.quantity, &product, &order);

            diesel::insert_into(order_items)
                .values(&od)
                .execute(conn)?;

            //update the product stock if order creation successful
            diesel::update(&product)
                .set(products::stock.eq(products::stock - order_detail.quantity))
                .execute(conn)?;
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
        Ok(http_response) => http_response,
        Err(e) => match e {
            diesel::result::Error::NotFound => HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message":"Product not found"})),
            diesel::result::Error::DatabaseError(_, c) => HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message":c.message()})),
            _ => HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message":"Ops! something went wrong"})),
        },
    }
}

#[post("")]
pub async fn create_backup(
    order_json: web::Json<OrderCreate>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
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

    use crate::schema::users::dsl::*;
    use crate::schema::order_items::dsl::*;
    use crate::schema::orders::dsl::*;
    use crate::schema::products::dsl::*;
    use crate::schema::{users, products};

    let mut order_total: f64 = 0.0;
    (&order_json.order_items).into_iter().for_each(|od| {
        order_total += od.price;
    });

    if order_total != order_json.total_price {
        return HttpResponse::BadRequest()
            .status(StatusCode::BAD_REQUEST)
            .json(serde_json::json!({"message": "Order data tempered.\nPrice of items and order total do not match"}));
    }

    //get a pooled connection from db
    let conn = &mut get_conn(&pool);

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

    let order: NewOrder = NewOrder::new(
        &user,
        order_json.created_on.to_owned(),
        order_json.delivery_charge,
        DeliveryStatus::Pending,
        order_json.delivery_location.to_owned(),
        order_total
    );

    match diesel::insert_into(orders)
        .values(&order)
        .get_result::<OrderModel>(conn)
    {
        Ok(o) => {
            //if any one of this failed, then god will help
            for order_detail in &order_json.order_items {
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

                let od: NewOrderItemModel =
                    NewOrderItemModel::new(order_detail.quantity, &pr, &o);

                diesel::insert_into(order_items)
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
                user_id: user_uuid.to_string(),
                created_on: o.get_created_on().to_owned(),
                fulfilled_on: o.get_fulfilled_on().to_owned(),
                total_price: o.get_total_price(),
                uuid: o.get_uuid().to_string(),
                delivery_charge: o.get_delivery_charge(),
                delivery_location: o.get_delivery_location().to_owned(),
                delivery_status: o.get_delivery_status().to_owned(),
            };
            HttpResponse::Ok()
                .status(StatusCode::OK)
                .json(serde_json::json!({"order": order}))
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

    use crate::schema::users::dsl::*;
    use crate::schema::orders::dsl::*;
    use crate::schema::{users, orders};

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
