//this api should be hit by payment providers

use std::time::Duration;

use ::uuid::Uuid;
use actix_web::{get, http::StatusCode, post, web, HttpResponse, Responder};
use diesel::prelude::*;
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE},
    Client,
};

use crate::{
    base_types::{
        delivery_status::DeliveryStatus, order_status::OrderStatus, payment_method::PaymentMethod,
    },
    config::ApplicationConfiguration,
    contracts::{
        khalti_payment::{
            AmountBreakdown, KhaltiPaymentPayload, KhaltiResponse, KhaltiResponseCamelCase,
            ProductDetail, UserInfo,
        },
        order::{
            CategoryResponse, OrderItemResponse, OrderResponse, ProductResponse, UserResponse,
        },
        payment::{
            EsewaCallbackResponse, KhaltiPaymentConfirmPayload, KhaltiPaymentLookupResponse,
            KhaltiPidxPayload, NewPayment, Payment,
        },
    },
    db::connection::{get_conn, SqliteConnectionPool},
    models::{
        invoice::{Invoice, NewInvoice},
        invoice_item::NewInvoiceItem,
        order::Order as OrderModel,
        order_item::OrderItem,
        payment::{NewPayment as NewPaymentModel, Payment as PaymentModel},
        product::Product,
        shipment::NewShipment,
        user::User as UserModel,
    },
    utils,
};

// I think any other method should not exists
#[post("")]
pub async fn create(
    payment_json: web::Json<NewPayment>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    create_payment(payment_json, &pool).await
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
    use crate::schema::orders::dsl::*;
    use crate::schema::payments::dsl::*;
    use crate::schema::users::dsl::*;
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

    let user: UserModel = match users
        .find(payment.get_user_id())
        .select(UserModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(Some(c)) => c,
        Ok(None) => {
            return HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "Invalid user id for payment"}))
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}))
        }
    };

    let payment_response: Payment = Payment {
        uuid: payment.get_uuid().to_owned(),
        user_id: user.get_uuid().to_owned(),
        order_id: order.get_uuid().to_owned(),
        pay_date: payment.get_pay_date().to_owned(),
        amount: payment.get_amount(),
        payment_method: payment.get_payment_method().to_owned(),
        tendered: payment.get_tendered(),
        change: payment.get_change(),
        discount: payment.get_discount(),
        transaction_id: payment.get_transaction_id().to_owned(),
    };

    HttpResponse::Ok()
        .status(StatusCode::OK)
        .json(payment_response)
}

#[post("/esewa")]
pub async fn esewa_payment_confirmation(
    req_body: web::Json<EsewaCallbackResponse>,
    client: web::Data<Client>,
    pool: web::Data<SqliteConnectionPool>,
    app_config: web::Data<ApplicationConfiguration>,
) -> impl Responder {
    println!("Hit by esewa");
    let txn_ref_id = req_body.transaction_details.reference_id.clone();
    println!("Transaction ref id is {txn_ref_id}");

    //Call the verification API with txn_ref_id
    let verification_result = verify_transaction(txn_ref_id, client, app_config).await;

    let response: HttpResponse = match verification_result {
        Ok(vr) => match vr.transaction_details.status.as_str() {
            "COMPLETE" => {
                let order_id = vr.product_id.clone();
                let order: OrderResponse = match get_order_details(&order_id, &pool).await {
                    Ok(o) => o,
                    Err(http_response) => return http_response,
                };

                let payment: NewPayment = NewPayment {
                    payment_method: PaymentMethod::Esewa.value().to_string(),
                    user_id: order.customer.uuid,
                    order_id: order.uuid,
                    amount: vr.total_amount.parse::<f64>().unwrap_or(0.0),
                    tendered: vr.total_amount.parse::<f64>().unwrap_or(0.0), //same as amount in case of payment providers
                    transaction_id: Some(vr.transaction_details.reference_id.to_owned()),
                };

                return create_payment(web::Json(payment), &pool).await;
            }
            _ => HttpResponse::Ok().json(serde_json::json!({
                "status": "success",
                "verification": "incomplete",
                "transaction_status": vr.transaction_details.status
            })),
        },
        Err(e) => {
            eprintln!("{e:?}");
            HttpResponse::BadRequest().json(serde_json::json!({
                "status": "failure",
                "verification": "incomplete",
                "errorMessage": e.to_string()
            }))
        }
    };

    response
}

async fn verify_transaction(
    txn_ref_id: String,
    client: web::Data<Client>,
    app_config: web::Data<ApplicationConfiguration>,
) -> Result<EsewaCallbackResponse, reqwest::Error> {
    println!("Hit by esewa");

    // Extract specific headers from the incoming request
    let mut headers = HeaderMap::new();

    headers.insert(
        "merchantId",
        HeaderValue::from_str(&app_config.esewa_merchant_id)
            .expect("Error setting esewa merchant id"),
    );

    headers.insert(
        "merchantSecret",
        HeaderValue::from_str(&app_config.esewa_merchant_secret)
            .expect("Error setting esewa merchant secret"),
    );

    headers.insert("Content-Type", HeaderValue::from_static("application/json"));

    let url = format!(
        "{}?txnRefId={}",
        app_config.esewa_payment_verification_url, txn_ref_id
    );

    // I don't know why array is returned
    let response: Vec<EsewaCallbackResponse> = client
        .get(url)
        .headers(headers)
        .timeout(Duration::from_secs(10))
        .send()
        .await?
        .json::<Vec<EsewaCallbackResponse>>()
        .await?;

    println!("----");
    println!("response: {response:?}");
    println!("-----");

    Ok(response[0].clone())
}

//khalti payment integration
#[get("/khalti")]
pub async fn khalti_payment_get_pidx(
    pidx_payload: web::Query<KhaltiPidxPayload>,
    pool: web::Data<SqliteConnectionPool>,
    client: web::Data<Client>,
    app_config: web::Data<ApplicationConfiguration>,
) -> impl Responder {
    println!("Hit by android khalti get pidx");

    let order_id: String = match utils::uuid_validator::validate_uuid(&pidx_payload.order_id) {
        Ok(c) => c,
        Err(http_response) => return http_response,
    };

    let order_details: OrderResponse = match get_order_details(&order_id, &pool).await {
        Ok(o) => o,
        Err(http_response) => return http_response,
    };

    let user_info = UserInfo {
        name: format!(
            "{} {}",
            order_details.customer.first_name, order_details.customer.last_name
        ),
        email: order_details.customer.email,
        phone: order_details.customer.phone_number,
    };

    let product_details = order_details
        .order_items
        .iter()
        .map(|item| {
            ProductDetail::new(
                item.uuid.to_owned(),
                item.product.name.to_owned(),
                item.quantity * item.price,
                item.price,
                item.quantity,
            )
        })
        .collect::<Vec<ProductDetail>>();

    let khalti_payment_payload = KhaltiPaymentPayload::create(
        &app_config.khalti_payment_confirm_callback_url,
        &app_config.khalti_payment_confirm_callback_webiste_url,
        order_details.total_price,
        order_details.uuid.into(),
        format!("{}'s Order", user_info.name),
        user_info,
        Some(vec![
            AmountBreakdown::new("Delivery Charge".into(), order_details.delivery_charge),
            AmountBreakdown::new(
                "Product Charge".into(),
                order_details.total_price - order_details.delivery_charge,
            ),
        ]),
        Some(product_details),
        "Himal Poudel".into(), ////merchant username
        String::from(""),      //merchant extra
    );

    println!("khalti_payment_payload: {khalti_payment_payload:?}");

    let response_result = client
        .post(&app_config.khalti_pidx_url)
        .header(
            AUTHORIZATION,
            &format!("key {}", &app_config.khalti_live_secret_key),
        )
        .header(CONTENT_TYPE, "application/json")
        .timeout(Duration::from_secs(10))
        .json(&khalti_payment_payload)
        .send()
        .await;

    let response: KhaltiResponseCamelCase = match response_result {
        Ok(res) => match res.status() {
            reqwest::StatusCode::OK => match res.json::<KhaltiResponse>().await {
                Ok(r) => r.into(),
                Err(er) => {
                    eprintln!("{er}");
                    return HttpResponse::InternalServerError()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .json(serde_json::json!({
                            "message": format!("Error parsing response from khalti: {}", er)
                        }));
                }
            },
            _ => match res.json::<serde_json::Value>().await {
                Ok(v) => {
                    println!("response: {v}");
                    return HttpResponse::Unauthorized()
                        .status(StatusCode::UNAUTHORIZED)
                        .json(v);
                }
                Err(e) => {
                    eprintln!("{e}");
                    return HttpResponse::InternalServerError()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .json(serde_json::json!({
                            "message": format!("Error parsing error response from khalti: {}", e)
                        }));
                }
            },
        },
        Err(e) => {
            eprintln!("{e}");
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({
                    "message": format!("Error getting response from khalti: {}", e)
                }));
        }
    };

    println!("----");
    println!("response: {response:?}");
    println!("-----");

    HttpResponse::Ok().status(StatusCode::OK).json(response)
}

#[post("/khalti/confirmation")]
pub async fn khalti_payment_confirmation(
    payload: web::Json<KhaltiPaymentConfirmPayload>,
    client: web::Data<Client>,
    pool: web::Data<SqliteConnectionPool>,
    app_config: web::Data<ApplicationConfiguration>,
) -> impl Responder {
    println!("Hit by khalti confirmation");

    //hit khalti lookup api for payment confirmation
    // let khalti_url = "https://a.khalti.com/api/v2/epayment/lookup";

    let data = serde_json::json!({
        "pidx": payload.pidx
    });

    let khalti_response_result = client
        .post(&app_config.khalti_payment_confirm_lookup_url)
        .header(
            AUTHORIZATION,
            &format!("key {}", &app_config.khalti_live_secret_key),
        )
        .header(CONTENT_TYPE, "application/json")
        .timeout(Duration::from_secs(10))
        .json(&data)
        .send()
        .await;

    match khalti_response_result {
        Ok(res) => match res.status() {
            reqwest::StatusCode::OK => match res.json::<KhaltiPaymentLookupResponse>().await {
                Ok(khalti_response) => {
                    println!("Confirmation response: {:?}", khalti_response);

                    match khalti_response.status.as_str() {
                        "Completed" => {
                            let order: OrderResponse =
                                match get_order_details(&payload.order_id, &pool).await {
                                    Ok(o) => o,
                                    Err(http_response) => return http_response,
                                };

                            let payment: NewPayment = NewPayment {
                                payment_method: PaymentMethod::Khalti.value().to_string(),
                                user_id: order.customer.uuid,
                                order_id: order.uuid,
                                amount: khalti_response.total_amount,
                                tendered: khalti_response.total_amount, // Same as amount
                                transaction_id: Some(khalti_response.transaction_id.to_owned()),
                            };

                            return create_payment(web::Json(payment), &pool).await;
                        }
                        _ => {
                            return HttpResponse::BadRequest()
                                .status(StatusCode::BAD_REQUEST)
                                .json(serde_json::json!({
                                    "message": "Payment is not completed"
                                }));
                        }
                    }
                }
                Err(er) => {
                    eprintln!("{er}");
                    return HttpResponse::InternalServerError()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .json(serde_json::json!({
                            "message": format!("Error parsing response from Khalti: {}", er)
                        }));
                }
            },
            _ => match res.json::<serde_json::Value>().await {
                Ok(v) => {
                    println!("Error response: {v}");
                    return HttpResponse::Unauthorized()
                        .status(StatusCode::UNAUTHORIZED)
                        .json(v);
                }
                Err(e) => {
                    eprintln!("{e}");
                    return HttpResponse::InternalServerError()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .json(serde_json::json!({
                            "message": format!("Error parsing error response from Khalti: {}", e)
                        }));
                }
            },
        },
        Err(e) => {
            eprintln!("{e}");
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({
                    "message": format!("Error getting response from Khalti: {}", e)
                }));
        }
    }
}

pub async fn create_payment(
    payment_json: web::Json<NewPayment>,
    pool: &web::Data<SqliteConnectionPool>,
) -> HttpResponse {
    use crate::schema::{invoice_items, invoices, shipments};
    use crate::schema::{
        orders::dsl as orders_dsl, payments::dsl::*, products::dsl as products_dsl,
        users::dsl as users_dsl,
    };
    use diesel::prelude::*;

    let conn = &mut get_conn(pool);

    // Parse UUIDs
    let o_uuid = match Uuid::parse_str(&payment_json.order_id) {
        Ok(u) => u,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "message": "Invalid order ID"
            }))
        }
    };

    let c_uuid = match Uuid::parse_str(&payment_json.user_id) {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "message": "Invalid user ID"
            }))
        }
    };

    // Determine payment method and transaction ID
    let (pay_method, tran_id) = match PaymentMethod::from_str(&payment_json.payment_method) {
        Ok(PaymentMethod::Cash) => (
            PaymentMethod::Cash,
            Some(Uuid::new_v4().to_string().replace("-", "")),
        ),
        Ok(method) => (method, payment_json.transaction_id.clone()),
        Err(e) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "message": e
            }))
        }
    };

    // Start DB transaction
    let result = conn.transaction::<HttpResponse, diesel::result::Error, _>(|con| {
        // Fetch order
        let order: OrderModel = match orders_dsl::orders
            .filter(orders_dsl::uuid.eq(&o_uuid.to_string()))
            .select(OrderModel::as_select())
            .first(con)
            .optional()?
        {
            Some(o) => o,
            None => {
                return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "message": "Order not found"
                })))
            }
        };

        // Fetch user
        let user: UserModel = match users_dsl::users
            .filter(users_dsl::uuid.eq(&c_uuid.to_string()))
            .select(UserModel::as_select())
            .first(con)
            .optional()?
        {
            Some(u) => u,
            None => {
                return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "message": "User not found"
                })))
            }
        };

        // Verify ownership
        if order.get_user_id() != user.get_id() {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "message": "Users do not match"
            })));
        }

        // Verify amount
        if order.get_total_price() != payment_json.amount {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "message": "Order total and payment amount did not match"
            })));
        }

        // Update order status
        diesel::update(&order)
            .set((
                orders_dsl::status.eq(OrderStatus::Pending.value()),
                orders_dsl::delivery_status.eq(DeliveryStatus::Pending.value()),
            ))
            .execute(con)?;

        // Create payment
        let payment_model = NewPaymentModel::new(
            &pay_method,
            &tran_id.unwrap(),
            &user,
            &order,
            payment_json.amount,
            payment_json.tendered,
        );

        let inserted: PaymentModel = diesel::insert_into(payments)
            .values(&payment_model)
            .get_result::<PaymentModel>(con)?;

        let user_location = user.get_location().unwrap_or_default();
        let shipment = NewShipment::new(user_location, &order);
        diesel::insert_into(shipments::table)
            .values(&shipment)
            .execute(con)?;

        let nepal_time = chrono::Utc::now()
            .with_timezone(&chrono::FixedOffset::east_opt(5 * 3600 + 45 * 60).unwrap());
        let inv_date = nepal_time.format("%Y-%m-%d %H:%M:%S").to_string();

        let new_inv: NewInvoice = NewInvoice::new(
            &inv_date,
            payment_json.amount,
            0.0,
            &order,
            &user,
            &inserted,
        );

        let invoice: Invoice = diesel::insert_into(invoices::table)
            .values(&new_inv)
            .get_result::<Invoice>(con)?;

        //extract order_items and create invoice item for each product
        let order_items = OrderItem::belonging_to(&order)
            .select(OrderItem::as_select())
            .load(con)?;

        for item in order_items {
            let product = products_dsl::products
                .find(item.get_product_id())
                .select(Product::as_select())
                .first(con)?; //might get runtime here but hey who cares

            let inv_item = NewInvoiceItem::new(&product, &invoice, item.get_quantity(), 0.0, 0.0);

            diesel::insert_into(invoice_items::table)
                .values(&inv_item)
                .execute(con)?;
        }

        // Prepare API response
        let response = Payment {
            uuid: inserted.get_uuid().to_owned(),
            payment_method: inserted.get_payment_method().to_owned(),
            pay_date: inserted.get_pay_date().to_owned(),
            user_id: user.get_uuid().to_owned(),
            order_id: order.get_uuid().to_owned(),
            amount: inserted.get_amount(),
            transaction_id: inserted.get_transaction_id().to_owned(),
            tendered: inserted.get_tendered(),
            change: inserted.get_change(),
            discount: inserted.get_discount(),
        };

        Ok(HttpResponse::Ok().json(response))
    });

    // Handle transaction result
    match result {
        Ok(response) => response,
        Err(_) => HttpResponse::InternalServerError().json(serde_json::json!({
            "message": "Oops! Something went wrong"
        })),
    }
}

async fn get_order_details(
    ord_id: &String,
    pool: &web::Data<SqliteConnectionPool>,
) -> Result<OrderResponse, HttpResponse> {
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
        Option<String>,
        Option<String>,
        Option<String>,
    );

    let result = orders
        .inner_join(users.on(orders::user_id.eq(users::id)))
        .inner_join(order_items.on(order_items::order_id.eq(orders::id)))
        .inner_join(products.on(order_items::product_id.eq(products::id)))
        .inner_join(categories.on(products::category_id.eq(categories::id)))
        .left_join(payments.on(payments::order_id.eq(orders::id)))
        .left_join(invoices.on(invoices::order_id.eq(orders::id)))
        .left_join(shipments.on(invoices::order_id.eq(orders::id)))
        .filter(orders::uuid.eq(ord_id.to_string()))
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
            payments::uuid.nullable(),
            invoices::uuid.nullable(),
            shipments::uuid.nullable(),
        ))
        .load::<OrderTuple>(conn);

    match result {
        Ok(order_rows) if order_rows.is_empty() => Err(HttpResponse::NotFound()
            .status(StatusCode::NOT_FOUND)
            .json(serde_json::json!({"message": "Order not found"}))),
        Ok(order_rows) => {
            use std::collections::HashMap;

            let mut map: HashMap<String, OrderResponse> = HashMap::new();

            for (
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
                payment_uuid,
                invoice_uuid,
                shipment_uuid,
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
                        status: order_status,
                        customer: UserResponse {
                            uuid: user_uuid,
                            first_name: user_first_name,
                            last_name: user_last_name,
                            phone_number: user_phone_number,
                            email: user_email,
                            user_type: utype,
                        },
                        order_items: vec![],
                        payment_id: payment_uuid,
                        invoice_id: invoice_uuid,
                        shipment_id: shipment_uuid,
                    });

                entry.order_items.push(OrderItemResponse {
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
                });
            }

            // Since we're filtering by a single order ID, there should be only one entry
            let response = map.into_iter().next().unwrap().1;
            Ok(response)
        }
        Err(_) => Err(HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message": "Ops! something went wrong"}))),
    }
}
