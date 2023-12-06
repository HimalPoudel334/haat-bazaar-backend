use actix_web::{delete, get, http::StatusCode, post, put, web, HttpResponse, Responder};
use diesel::prelude::*;

use crate::{
    contracts::product::{Product, ProductCreate},
    db::connection::{get_conn, SqliteConnectionPool},
    models::{
        category::Category as CategoryModel,
        product::{NewProduct, Product as ProductModel},
    },
};

#[get("/")]
pub async fn get(pool: web::Data<SqliteConnectionPool>) -> impl Responder {
    use crate::schema::categories;
    use crate::schema::categories::dsl::*;
    use crate::schema::products;
    use crate::schema::products::dsl::*;

    let product_vec = products
        .inner_join(categories)
        .select((
            products::uuid,
            products::name,
            description,
            image,
            price,
            previous_price,
            unit,
            unit_change,
            stock,
            categories::uuid,
            categories::name,
        ))
        .load::<Product>(&mut get_conn(&pool));

    match product_vec {
        Ok(p) => HttpResponse::Ok().status(StatusCode::OK).json(p),
        Err(e) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message": e.to_string()})),
    }
}

#[post("/")]
pub async fn create(
    product_json: web::Json<ProductCreate>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    use crate::schema::categories;
    use crate::schema::categories::dsl::*;
    use crate::schema::products::dsl::*;

    //check if the provided category exists or not
    let category: CategoryModel = match categories
        .filter(categories::uuid.eq(&product_json.category_id))
        .select(CategoryModel::as_select())
        .first::<CategoryModel>(&mut get_conn(&pool))
        .optional()
    {
        Ok(Some(c)) => c,
        Ok(None) => {
            return HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "Category could not be found"}))
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}))
        }
    };

    let product: NewProduct = NewProduct::new(
        product_json.name.to_owned(),
        product_json.description.to_owned(),
        product_json.image.to_owned(),
        product_json.price,
        product_json.previous_price,
        product_json.unit.to_owned(),
        product_json.unit_change,
        product_json.stock,
        &category,
    );

    //insert the product to db
    match diesel::insert_into(products)
        .values(&product)
        .get_result::<ProductModel>(&mut get_conn(&pool))
    {
        Ok(p) => {
            let product_response: Product = Product {
                uuid: p.get_uuid().to_owned(),
                name: product_json.name.to_owned(),
                description: product_json.description.to_owned(),
                image: product_json.image.to_owned(),
                price: product_json.price,
                previous_price: product_json.previous_price,
                unit: product_json.unit.to_owned(),
                unit_change: product_json.unit_change,
                stock: product_json.stock,
                category_id: category.get_uuid().to_owned(),
                category_name: category.get_name().to_owned(),
            };

            HttpResponse::Ok()
                .status(StatusCode::OK)
                .json(product_response)
        }
        Err(_) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message": "Ops! Something went wrong"})),
    }
}

#[get("/{product_id}")]
pub async fn get_product(product_id: String) -> impl Responder {
    HttpResponse::Ok().finish()
}

#[put("/{product_id}")]
pub async fn edit(product_id: String) -> impl Responder {
    HttpResponse::Ok().finish()
}

#[delete("/{product_id}")]
pub async fn delete(product_id: String) -> impl Responder {
    HttpResponse::Ok().finish()
}
