use actix_web::{delete, get, http::StatusCode, patch, post, web, HttpResponse, Responder};
use diesel::prelude::*;
use uuid::Uuid;

use crate::contracts::cart::{Cart, NewCart, UpdateCartQuantity};
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
            products::price,
            (carts::quantity * products::price),
            carts::sku,
            products::image,
            carts::created_on,
            products::stock,
            products::unit_change,
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
                rate: product.get_price(),
                total: c.get_quantity() * product.get_price(),
                sku: c.get_sku().to_owned(),
                image: product.get_image().to_owned(),
                created_on: c.get_created_on().to_owned(),
                product_name: product.get_name().to_owned(),
                product_stock: product.get_stock(),
                product_unit_change: product.get_unit_change(),
            };
            HttpResponse::Ok()
                .status(StatusCode::OK)
                .json(serde_json::json!({"cart": cart_vm}))
        }
        Err(_) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message": "Ops! something went wrong"})),
    }
}

#[patch("/{cart_id}/update-quantity")]
pub async fn update_quantity(
    cart_id: web::Path<(String,)>,
    pool: web::Data<SqliteConnectionPool>,
    quantity_vm: web::Json<UpdateCartQuantity>,
) -> impl Responder {
    let cart_uuid: String = cart_id.into_inner().0;

    //check if product_quantity is greater than 0
    if quantity_vm.new_quantity <= 0.25 {
        return HttpResponse::BadRequest()
            .status(StatusCode::BAD_REQUEST)
            .json(serde_json::json!({"message": "Quantity must be greater than 0.25 sku"}));
    }

    //check if the customer id is valid uuid or not
    let cart_uuid: Uuid = match Uuid::parse_str(&cart_uuid) {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Invalid cart id"}))
        }
    };

    use crate::schema::carts;
    use crate::schema::carts::dsl::*;
    use crate::schema::products::dsl::*;

    let conn = &mut get_conn(&pool);

    let cart: CartModel = match carts
        .filter(carts::uuid.eq(&cart_uuid.to_string()))
        .select(CartModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(Some(c)) => c,
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

    //get the product for the cart
    let product: ProductModel = match products
        .find(cart.get_product_id())
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

    // validate the quantity against the stock of the product
    if quantity_vm.new_quantity > product.get_stock() {
        return HttpResponse::BadRequest()
            .status(StatusCode::BAD_REQUEST)
            .json(serde_json::json!({"message": "Requested quantity is greater than stock"}));
    }

    match diesel::update(&cart)
        .set(carts::quantity.eq(quantity_vm.new_quantity))
        .get_result::<CartModel>(conn)
    {
        Ok(c) => {
            let cart_vm: Cart = Cart {
                uuid: c.get_uuid().to_owned(),
                product_id: product.get_uuid().to_owned(),
                quantity: c.get_quantity(),
                rate: product.get_price(),
                total: c.get_quantity() * product.get_price(),
                sku: c.get_sku().to_owned(),
                image: product.get_image().to_owned(),
                created_on: c.get_created_on().to_owned(),
                product_name: product.get_name().to_owned(),
                product_stock: product.get_stock(),
                product_unit_change: product.get_unit_change(),
            };
            HttpResponse::Ok().status(StatusCode::OK).json(cart_vm)
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}))
        }
    }
}

#[delete("/{cart_id}")]
pub async fn delete_cart(
    cart_id: web::Path<(String,)>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    let cart_id: String = cart_id.into_inner().0;

    //check if the customer id is valid uuid or not
    let cart_id: Uuid = match Uuid::parse_str(&cart_id) {
        Ok(c) => c,
        Err(_) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Invalid cart id"}))
        }
    };

    use crate::schema::carts;
    use crate::schema::carts::dsl::*;

    let conn = &mut get_conn(&pool);

    let cart: CartModel = match carts
        .filter(carts::uuid.eq(&cart_id.to_string()))
        .select(CartModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(Some(c)) => c,
        Ok(None) => {
            return HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "Cart not found"}))
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}))
        }
    };

    match diesel::delete(&cart).execute(conn).optional() {
        Ok(_) => HttpResponse::NoContent()
            .status(StatusCode::NO_CONTENT)
            .finish(),
        Err(_) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message": "Ops! something went wrong"})),
    }
}

#[delete("/delete-carts/{cust_id}")]
pub async fn delete_customer_cart(
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

    use crate::schema::customers;
    use crate::schema::customers::dsl::*;

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

    match diesel::delete(CartModel::belonging_to(&customer))
        .execute(conn)
        .optional()
    {
        Ok(Some(c)) => HttpResponse::NoContent()
            .status(StatusCode::NO_CONTENT)
            .json(c),
        Ok(None) => HttpResponse::NotFound()
            .status(StatusCode::NOT_FOUND)
            .json(serde_json::json!({"message": "Customer not found"})),

        Err(_) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message": "Ops! something went wrong"})),
    }
}
