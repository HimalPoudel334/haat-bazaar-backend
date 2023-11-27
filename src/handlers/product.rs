use actix_web::{delete, get, post, put, HttpResponse, Responder};

#[post("/")]
pub async fn create() -> impl Responder {
    HttpResponse::Ok().finish()
}

#[get("/")]
pub async fn get() -> impl Responder {
    HttpResponse::Ok().finish()
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
