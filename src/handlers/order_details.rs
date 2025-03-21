use actix_web::{get, http::StatusCode, post, web, HttpResponse, Responder};
use uuid::Uuid;

use diesel::prelude::*;

use crate::{
    contracts::order_item::{NewOrderItem, OrderItems},
    db::connection::{get_conn, SqliteConnectionPool},
    models::{
        order::Order as OrderModel,
        order_item::{NewOrderItem as NewOrderItemModel, OrderItem as OrderItemsModel},
        product::Product as ProductModel,
    },
};

#[get("/{order_uid}")]
pub async fn get(
    order_uid: web::Path<(String,)>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    let order_uid: String = order_uid.into_inner().0;
    let order_uid = match Uuid::parse_str(&order_uid) {
        Ok(oid) => oid,
        Err(_) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Invalid order id"}))
        }
    };

    use crate::schema::order_items::dsl::*;
    use crate::schema::orders::dsl::*;
    use crate::schema::products::dsl::*;
    use crate::schema::{order_items, orders, products};

    //get a pooled connection from db
    let conn = &mut get_conn(&pool);

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
                .json(serde_json::json!({"message": "Order not found"}))
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! somethind went wrong"}))
        }
    };

    //get order items for order

    // let all_order_items: Vec<OrderItems> = match OrderItemsModel::belonging_to(&order).select((
    //    order_items::uuid,
    //     order_items::price,
    //     order_items::quantity,
    //     order_items::product_id,
    //
    // )).load<OrderItems>::(conn).optional() {
    //
    //     };
    //

    match order_items
        .inner_join(products)
        .inner_join(orders)
        .filter(order_items::uuid.eq(&order.get_uuid()))
        .select((
            order_items::uuid,
            products::uuid,
            orders::uuid,
            order_items::quantity,
            order_items::price,
        ))
        .load::<OrderItems>(conn)
    {
        Ok(ods) => HttpResponse::Ok()
            .status(StatusCode::OK)
            .json(serde_json::json!({"order-items": ods})),
        Err(_) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message": "Ops! something went wrong"})),
    }
}

#[get("order-detail/{od_uuid}")]
pub async fn get_order_detail(
    od_uuid: web::Path<(String,)>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    let order_detail_uid: String = od_uuid.into_inner().0;
    let order_detail_uid = match Uuid::parse_str(&order_detail_uid) {
        Ok(oid) => oid,
        Err(_) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Invalid order id"}))
        }
    };

    use crate::schema::order_items;
    use crate::schema::order_items::dsl::*;
    use crate::schema::orders::dsl::*;
    use crate::schema::products::dsl::*;

    //get a pooled connection from db
    let conn = &mut get_conn(&pool);

    match order_items
        .filter(order_items::uuid.eq(&order_detail_uid.to_string()))
        .select(OrderItemsModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(Some(od)) => {
            //get the product and order
            let prod: ProductModel = products
                .find(od.get_product_id())
                .select(ProductModel::as_select())
                .first(conn)
                .unwrap();

            let ord: OrderModel = orders
                .find(od.get_order_id())
                .select(OrderModel::as_select())
                .first(conn)
                .unwrap();

            let order_det: OrderItems = OrderItems {
                uuid: od.get_uuid().to_owned(),
                price: od.get_price(),
                product_id: prod.get_uuid().to_owned(),
                order_id: ord.get_uuid().to_owned(),
                quantity: od.get_quantity(),
            };
            HttpResponse::Ok().status(StatusCode::OK).json(order_det)
        }
        Ok(None) => HttpResponse::NotFound()
            .status(StatusCode::NOT_FOUND)
            .json(serde_json::json!({"message": "Order detail not found"})),
        Err(_) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message": "Ops! something went wrong"})),
    }
}

#[post("/{ord_id}/add")]
pub async fn add_order_detail(
    ord_id: web::Path<(String,)>,
    pool: web::Data<SqliteConnectionPool>,
    ord_det: web::Json<NewOrderItem>,
) -> impl Responder {
    let ord_id: String = ord_id.into_inner().0;
    //check if the ord_id is a valid uuid or not
    let ord_id: Uuid = match Uuid::parse_str(&ord_id) {
        Ok(o) => o,
        Err(_) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Invalid order id"}))
        }
    };

    //check if the order exists or not
    use crate::schema::order_items::dsl::*;
    use crate::schema::orders::dsl::*;
    use crate::schema::products::dsl::*;
    use crate::schema::{orders, products};

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
            return HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "Order detail not found"}))
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}))
        }
    };

    //check if product exists or not
    let prod_bought: ProductModel = match products
        .filter(products::uuid.eq(&ord_det.product_id))
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

    // if product and order both exists then insert the new record into order_items table
    let od: NewOrderItemModel = NewOrderItemModel::new(ord_det.quantity, &prod_bought, &order);

    match diesel::insert_into(order_items).values(&od).execute(conn) {
        Ok(_) => HttpResponse::Ok()
            .status(StatusCode::OK)
            .json(serde_json::json!({"message": "order detail added"})),
        Err(_) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message": "Ops! something went wrong"})),
    }
}
