use diesel::{deserialize::Queryable, Selectable};
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
    pub status: String,
    pub quantity: f64,
    pub unit: String,
    pub product_image: String,
    pub product_name: String,
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
    pub status: String,
    pub customer: UserResponse,
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
    pub payment_method: String,
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
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrdersFilterParams {
    pub init_date: String,
    pub final_date: Option<String>,
}
