use actix_web::{get, post};
use actix_web::{http::StatusCode, web, HttpResponse, Responder};
use diesel::prelude::*;
use diesel::SelectableHelper;
use uuid::Uuid;

use crate::contracts::cart::{Cart, NewCart};
use crate::models::cart::{Cart as CartModel, NewCartItem};
use crate::{
    db::connection::{get_conn, SqliteConnectionPool},
    models::{customer::Customer as CustomerModel, product::Product as ProductModel},
};

#[get("/{cust_id}")]
pub async fn get(
    cust_id: web::Path<(String,)>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    let cust_id: String = cust_id.into_inner().0;

    //check if the customer id is valid uuid or not
    let cust_id: Uuid = match Uuid::parse_str(&cust_id) {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Invalid customer id"}))
        }
    };

    //check if customer exists or not
    //maybe not needed

    use crate::schema::carts::dsl::*;
    use crate::schema::customers::dsl::*;
    use crate::schema::products::dsl::*;
    use crate::schema::{carts, customers, products};

    let customer: CustomerModel = match customers
        .filter(customers::uuid.eq(&cust_id.to_string()))
        .select(CustomerModel::as_select())
        .first(&mut get_conn(&pool))
        .optional()
    {
        Ok(Some(c)) => c,
        Ok(None) => {
            return HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "Customer not found"}))
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}))
        }
    };

    let carts_vec: Vec<Cart> = match carts
        .inner_join(customers)
        .inner_join(products)
        .filter(carts::customer_id.eq(&customer.get_id()))
        .select((
            carts::uuid,
            products::uuid,
            products::name,
            carts::quantity,
            carts::created_on,
        ))
        .load::<Cart>(&mut get_conn(&pool))
        .optional()
    {
        Ok(Some(c)) => c,
        Ok(None) => {
            return HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "Customer not found"}))
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}))
        }
    };

    HttpResponse::Ok().json(serde_json::json!({"carts": carts_vec}))
}

#[post("/{cust_id}")]
pub async fn create(
    cust_id: web::Path<(String,)>,
    cart_item: web::Json<NewCart>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    let cust_id: String = cust_id.into_inner().0;

    //check if product_quantity is greater than 0
    if cart_item.quantity <= 0.25 {
        return HttpResponse::BadRequest()
            .status(StatusCode::BAD_REQUEST)
            .json(serde_json::json!({"message": "Quantity must be greater than 0.25 sku"}));
    }

    //check if the customer id is valid uuid or not
    let cust_id: Uuid = match Uuid::parse_str(&cust_id) {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Invalid customer id"}))
        }
    };

    //check if customer exists or not
    //maybe not needed

    use crate::schema::carts::dsl::*;
    use crate::schema::customers::dsl::*;
    use crate::schema::products::dsl::*;
    use crate::schema::{customers, products};

    let conn = &mut get_conn(&pool);

    let customer: CustomerModel = match customers
        .filter(customers::uuid.eq(&cust_id.to_string()))
        .select(CustomerModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(Some(c)) => c,
        Ok(None) => {
            return HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "Customer not found"}))
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}))
        }
    };

    let product: ProductModel = match products
        .filter(products::uuid.eq(&cart_item.product_id))
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

    let cart: NewCartItem = NewCartItem::new(
        &product,
        &customer,
        cart_item.quantity,
        cart_item.created_on.to_owned(),
    );

    match diesel::insert_into(carts)
        .values(&cart)
        .get_result::<CartModel>(conn)
    {
        Ok(c) => {
            let cart_vm: Cart = Cart {
                uuid: c.get_uuid().to_owned(),
                product_id: product.get_uuid().to_owned(),
                quantity: c.get_quantity(),
                created_on: c.get_created_on().to_owned(),
                product_name: product.get_name().to_owned(),
            };
            HttpResponse::Ok().status(StatusCode::OK).json(cart_vm)
        }
        Err(_) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message": "Ops! something went wrong"})),
    }
}
