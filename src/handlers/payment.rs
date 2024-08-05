use std::time::Duration;

use ::uuid::Uuid;
use actix_web::{get, http::StatusCode, post, web, HttpRequest, HttpResponse, Responder};
use diesel::prelude::*;
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE},
    Client,
};

use crate::{
    base_types::payment_method::PaymentMethod,
    config::ApplicationConfiguration,
    contracts::{
        khalti_payment::{
            AmountBreakdown, CustomerInfo, KhaltiPayment, KhaltiResponse, KhaltiResponseCamelCase,
            ProductDetail,
        },
        order::OrderCreate,
        payment::{EsewaCallbackResponse, EsewaTransactionResponse, NewPayment, Payment},
    },
    db::connection::{get_conn, SqliteConnectionPool},
    models::{
        customer::Customer as CustomerModel,
        order::Order as OrderModel,
        payment::{NewPayment as NewPaymentModel, Payment as PaymentModel},
    },
    utils,
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

#[post("/payment/confirmation")]
pub async fn esewa_payment_confirmation(
    req: HttpRequest,
    req_body: web::Json<EsewaCallbackResponse>,
    client: web::Data<Client>,
    app_config: web::Data<ApplicationConfiguration>,
) -> impl Responder {
    println!("Hit by esewa");
    let txn_ref_id = req_body.transaction_details.reference_id.clone();
    println!("Transaction ref id is {txn_ref_id}");

    //Call the verification API with txn_ref_id
    let verification_result = verify_transaction(txn_ref_id, req, client, app_config).await;

    match verification_result {
        Ok(status) if status == "COMPLETE" => {
            // Handle successful verification
            HttpResponse::Ok()
                .json(serde_json::json!({"status": "success", "verification": "complete"}))
        }
        Ok(_) => HttpResponse::Ok()
            .json(serde_json::json!({"status": "Khai k", "verification": "khai K"})),
        Err(e) => {
            eprintln!("{e:?}");
            HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"status": "failure", "verification": "incomplete", "errorMessage":e.to_string()}))
        }
    }
}

async fn verify_transaction(
    txn_ref_id: String,
    req: HttpRequest,
    client: web::Data<Client>,
    app_config: web::Data<ApplicationConfiguration>,
) -> Result<String, reqwest::Error> {
    println!("Hit by esewa");

    // Extract headers from the incoming request
    // let mut headers = HeaderMap::new();
    // for (key, value) in req.headers().iter() {
    //     if let (Ok(header_name), Ok(header_value)) = (
    //         HeaderName::from_str(key.as_str()),
    //         HeaderValue::from_str(value.to_str().unwrap_or("")),
    //     ) {
    //         println!("{header_name}:{:#?}", header_value);
    //         headers.insert(header_name, header_value);
    //     }
    // }

    // Extract specific headers from the incoming request
    let mut headers = HeaderMap::new();
    if let Some(merchant_id) = req.headers().get("merchantId") {
        headers.insert(
            "merchantId",
            HeaderValue::from_bytes(merchant_id.as_bytes()).unwrap_or(
                HeaderValue::from_str(&app_config.esewa_merchant_id)
                    .expect("Error setting esewa merchant id 1"),
            ),
        );
    } else {
        headers.insert(
            "merchantId",
            HeaderValue::from_str(&app_config.esewa_merchant_id)
                .expect("Error setting esewa merchant id 2"),
        );
    }
    if let Some(merchant_secret) = req.headers().get("merchantSecret") {
        headers.insert(
            "merchantSecret",
            HeaderValue::from_bytes(merchant_secret.as_bytes()).unwrap_or(
                HeaderValue::from_str(&app_config.esewa_merchant_secret)
                    .expect("Error setting esewa merchant secret 1"),
            ),
        );
    } else {
        headers.insert(
            "merchantSecret",
            HeaderValue::from_str(&app_config.esewa_merchant_secret)
                .expect("Error setting esewa merchant secret 1"),
        );
    }

    headers.insert("Content-Type", HeaderValue::from_static("application/json"));

    let url = format!(
        "https://rc.esewa.com.np/mobile/transaction?txnRefId={}",
        txn_ref_id
    );

    // I don't know why array is returned
    let response: Vec<EsewaTransactionResponse> = client
        .get(url)
        .headers(headers)
        .timeout(Duration::from_secs(10))
        .send()
        .await?
        .json::<Vec<EsewaTransactionResponse>>()
        .await?;

    println!("----");
    println!("response: {response:?}");
    println!("-----");

    //Ok(response.transaction_details.status.to_string())
    Ok(response[0].transaction_details.status.to_string())
}

//khalti payment integration
#[post("/payment")]
pub async fn khalti_payment_get_pidx(
    //order_json: web::Json<OrderCreate>,
    pool: web::Data<SqliteConnectionPool>,
    client: web::Data<Client>,
    app_config: web::Data<ApplicationConfiguration>,
) -> impl Responder {
    use crate::schema::customers;
    use crate::schema::customers::dsl::*;

    println!("Hit by android");

    // //get a pooled connection from db
    // let conn = &mut get_conn(&pool);

    // // let customer_id: String = match utils::uuid_validator::validate_uuid(&order_json.customer_id) {
    // //     Ok(c) => c,
    // //     Err(http_response) => return http_response,
    // // };
    // //
    // // let customer: CustomerModel = match customers
    // //     .filter(uuid.eq(&customer_id))
    // //     .select(CustomerModel::as_select())
    // //     .first(conn)
    // //     .optional()
    // // {
    // //     Ok(Some(c)) => c,
    // //     Ok(None) => {
    // //         return HttpResponse::BadRequest()
    // //             .status(StatusCode::BAD_REQUEST)
    // //             .json(serde_json::json!({"message": "Customer not found"}))
    // //     }
    // //     Err(_) => {
    // //         return HttpResponse::BadRequest()
    // //             .status(StatusCode::BAD_REQUEST)
    // //             .json(serde_json::json!({"message": "Ops! something went wrong"}))
    // //     }
    // // };
    // //
    // //Create a Khalti payment struct
    // let khalti_payment_payload: KhaltiPayment = KhaltiPayment {
    //     return_url: "http://0.0.0.0:8080/payments/khalti/payment/".into(),
    //     website_url: "http://0.0.0.0:8080/".into(),
    //     amount: 1300.0, // order_json.total_price, //get from the request body
    //     purchase_order_id: "some id".into(), //get from request body
    //     purchase_order_name: "some order name".into(),
    //     customer_info: CustomerInfo {
    //         name: "Sallu bhai".into(),
    //         email: "Sallu@bhai.com".into(),
    //         phone: "9800000001".into(),
    //     },
    //     amount_breakdown: Some(
    //         [
    //             AmountBreakdown {
    //                 label: "some label".into(),
    //                 amount: 1000.0,
    //             },
    //             AmountBreakdown {
    //                 label: "some label".into(),
    //                 amount: 300.0,
    //             },
    //         ]
    //         .to_vec(),
    //     ),
    //     product_details: Some(
    //         [
    //             ProductDetail {
    //                 identity: "some product id1".into(),
    //                 name: "some product name1".into(),
    //                 total_price: 500.0,
    //                 unit_price: 250.0,
    //                 quantity: 2,
    //             },
    //             ProductDetail {
    //                 identity: "some product id2".into(),
    //                 name: "some product name2".into(),
    //                 total_price: 500.0,
    //                 unit_price: 100.0,
    //                 quantity: 5,
    //             },
    //         ]
    //         .to_vec(),
    //     ),
    //     merchant_username: "khalti username".into(),
    //     merchant_extra: String::from(""),
    // };

    // let khalti_url = "https://a.khalti.com/api/v2/epayment/initiate/";

    // let response_result = client
    //     .post(khalti_url)
    //     .header(
    //         AUTHORIZATION,
    //         &format!("key {}", &app_config.khalti_live_secret_key),
    //     )
    //     .header(CONTENT_TYPE, "application/json")
    //     .timeout(Duration::from_secs(10))
    //     .json(&khalti_payment_payload)
    //     .send()
    //     .await;

    // let response: KhaltiResponseCamelCase =
    //     match response_result {
    //         Ok(res) => match res.status() {
    //             reqwest::StatusCode::OK => match res.json::<KhaltiResponse>().await {
    //                 Ok(r) => r.into(),
    //                 Err(er) => {
    //                     eprintln!("{er}");
    //                     return HttpResponse::InternalServerError()
    //                         .status(StatusCode::INTERNAL_SERVER_ERROR)
    //                         .json(serde_json::json!({
    //                             "message": format!("Error parsing response from khalti: {}", er)
    //                         }));
    //                 }
    //             },
    //             _ => match res.json::<serde_json::Value>().await {
    //                 Ok(v) => {
    //                     return HttpResponse::Unauthorized()
    //                         .status(StatusCode::UNAUTHORIZED)
    //                         .json(v)
    //                 }
    //                 Err(e) => return HttpResponse::InternalServerError()
    //                     .status(StatusCode::INTERNAL_SERVER_ERROR)
    //                     .json(serde_json::json!({
    //                         "message": format!("Error parsing error response from khalti: {}", e)
    //                     })),
    //             },
    //         },
    //         Err(e) => {
    //             eprintln!("{e}");
    //             return HttpResponse::InternalServerError()
    //                 .status(StatusCode::INTERNAL_SERVER_ERROR)
    //                 .json(serde_json::json!({
    //                     "message": format!("Error getting response from khalti: {}", e)
    //                 }));
    //         }
    //     };

    // println!("----");
    // println!("response: {response:?}");
    // println!("-----");

    //HttpResponse::Ok().status(StatusCode::OK).json(response)

    HttpResponse::Ok()
        .status(StatusCode::OK)
        .json(serde_json::json!({
          "pidx": "cYqZsuXqBo5nsPHHG5Fibk",
          "paymentUrl": "https://test-pay.khalti.com/?pidx=cYqZsuXqBo5nsPHHG5Fibk",
          "expiresAt": "2024-08-04T10:33:56.423485+05:45",
          "expiresIn": 1800
        }))
}
