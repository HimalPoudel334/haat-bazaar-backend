use diesel::sql_types::{Double, Integer, Text};
use diesel::{deserialize::Queryable, prelude::QueryableByName, sqlite::Sqlite, Selectable};
use serde::{Deserialize, Serialize};

use super::order_item::NewOrderItem;

#[derive(Serialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::orders)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    #[serde(rename = "id")]
    pub uuid: String,
    pub created_on: String,
    pub fulfilled_on: String,
    pub delivery_charge: f64,
    pub delivery_location: String,
    pub delivery_status: String,
    pub total_price: f64,
    pub user_id: String,
}

#[derive(Serialize, Queryable)]
#[serde(rename_all = "camelCase")]
pub struct AllOrderResponse {
    #[serde(rename = "id")]
    pub uuid: String,
    pub created_on: String,
    pub fulfilled_on: String,
    pub delivery_charge: f64,
    pub delivery_location: String,
    pub delivery_status: String,
    pub total_price: f64,
    pub discount: f64,
    pub amount: f64,
    pub status: String,
    pub quantity: f64,
    pub unit: String,
    pub product_image: String,
    pub product_name: String,
}

#[derive(Debug, QueryableByName)]
#[diesel(check_for_backend(Sqlite))]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AllOrderResponse1 {
    #[diesel(sql_type = Text)]
    #[serde(rename = "id")]
    pub uuid: String,

    #[diesel(sql_type = Text)]
    pub created_on: String,

    #[diesel(sql_type = Text)]
    pub fulfilled_on: String,

    #[diesel(sql_type = Double)]
    pub delivery_charge: f64,

    #[diesel(sql_type = Text)]
    pub delivery_location: String,

    #[diesel(sql_type = Text)]
    pub delivery_status: String,

    #[diesel(sql_type = Double)]
    pub total_price: f64,

    #[diesel(sql_type = Double)]
    pub discount: f64,

    #[diesel(sql_type = Double)]
    pub amount: f64,

    #[diesel(sql_type = Text)]
    pub status: String,

    #[diesel(sql_type = Integer)]
    pub quantity: i32,

    #[diesel(sql_type = Text)]
    pub unit: String,

    #[diesel(sql_type = Text)]
    pub image: String,

    #[diesel(sql_type = Text)]
    pub name: String,
}

/* Order response */
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderResponse {
    #[serde(rename = "id")]
    pub uuid: String,
    pub created_on: String,
    pub fulfilled_on: String,
    pub delivery_charge: f64,
    pub delivery_location: String,
    pub delivery_status: String,
    pub total_price: f64,
    pub discount: f64,
    pub amount: f64,
    pub status: String,
    pub user: UserResponse,
    pub order_items: Vec<OrderItemResponse>,
    pub payment: Option<PaymentResponse>,
    pub shipment: Option<ShipmentResponse>,
    pub invoice_id: Option<String>,
}

#[derive(Serialize, Queryable)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    #[serde(rename = "id")]
    pub uuid: String,
    pub first_name: String,
    pub last_name: String,
    pub phone_number: String,
    pub email: String,
    pub user_type: String,
}

#[derive(Serialize, Queryable)]
pub struct OrderItemResponse {
    #[serde(rename = "id")]
    pub uuid: String,
    pub product: ProductResponse,
    pub quantity: f64,
    pub price: f64,
    pub discount: f64,
    pub amount: f64,
}

#[derive(Serialize, Queryable)]
pub struct ProductResponse {
    #[serde(rename = "id")]
    pub uuid: String,
    pub name: String,
    pub description: String,
    pub image: String,
    pub price: f64,
    pub unit: String,
    pub category: CategoryResponse,
}

#[derive(Serialize, Queryable)]
pub struct CategoryResponse {
    #[serde(rename = "id")]
    pub uuid: String,
    pub name: String,
}

#[derive(Serialize, Queryable)]
#[serde(rename_all = "camelCase")]
pub struct PaymentResponse {
    #[serde(rename = "id")]
    pub uuid: String,
    pub payment_method: String,
    pub transaction_id: String,
    pub amount: f64,
    pub status: String,
}

#[derive(Serialize, Queryable)]
#[serde(rename_all = "camelCase")]
pub struct ShipmentResponse {
    pub uuid: String,
    pub status: String,
    pub ship_date: String,
}

/* End */

#[derive(Serialize, Queryable)]
#[serde(rename_all = "camelCase")]
pub struct UserOrderResponse {
    #[serde(rename = "id")]
    pub uuid: String,
    pub created_on: String,
    pub fulfilled_on: String,
    pub delivery_charge: f64,
    pub delivery_location: String,
    pub delivery_status: String,
    pub total_price: f64,
    pub discount: f64,
    pub amount: f64,
    pub status: String,
    pub order_items: Vec<OrderItemResponse>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderCreate {
    pub created_on: String,
    pub fulfilled_on: Option<String>,
    pub delivery_charge: f64,
    pub delivery_location: String,
    pub delivery_status: String,
    pub total_price: f64,
    pub user_id: String,
    pub order_items: Vec<NewOrderItem>,
    pub payment: PaymentDetails,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentDetails {
    pub payment_method: String,
    pub status: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderEdit {
    pub fulfilled_on: String,
    pub delivery_status: String,
    pub delivery_location: String,
    pub total_price: f64,
    pub user_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderDeliveryStatus {
    pub delivery_status: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderStatus {
    pub status: String,
}

#[derive(Deserialize)]
pub struct CartCheckout {
    pub cart_ids: Vec<String>,
    pub user_id: String,
    pub payment_method: String,
    pub payment_status: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DateFilterParams {
    pub init_date: String,
    pub final_date: Option<String>,
}
