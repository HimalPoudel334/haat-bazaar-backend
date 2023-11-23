use actix_web::{delete, get, post, put, web, HttpResponse, Responder};

use crate::{contracts::category::CategoryCreate, models::category::NewCategory};

#[post("/create")]
pub async fn create(category: web::Json<CategoryCreate>) -> impl Responder {
    let category: NewCategory = NewCategory::new(category.name.to_owned());
    HttpResponse::Ok().json(category)
}

#[get("/")]
pub async fn get() -> impl Responder {
    HttpResponse::Ok().finish()
}

#[get("/{catgory_id}")]
pub async fn get_product(category_id: web::Path<(String,)>) -> impl Responder {
    let id: String = category_id.into_inner().0;
    HttpResponse::Ok().finish()
}

#[put("/put/{category_id}")]
pub async fn edit(category_id: web::Path<(String,)>) -> impl Responder {
    let id: String = category_id.into_inner().0;
    HttpResponse::Ok().finish()
}

#[delete("/delete/{category_id}")]
pub async fn delete(category_id: web::Path<(String,)>) -> impl Responder {
    let id: String = category_id.into_inner().0;
    HttpResponse::Ok().finish()
}
