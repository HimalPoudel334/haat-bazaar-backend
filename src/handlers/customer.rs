use actix_web::{delete, get, post, put, web, HttpResponse, Responder};

#[post("/")]
pub async fn create() -> impl Responder {
    HttpResponse::Ok()
}

#[get("/")]
pub async fn get() -> impl Responder {
    HttpResponse::Ok().finish()
}

#[get("/{user_id}")]
pub async fn get_customer(user_id: web::Path<(String,)>) -> impl Responder {
    let id: String = user_id.into_inner().0;
    HttpResponse::Ok().body(format!("Hello {}", id))
}

#[put("/{user_id}")]
pub async fn edit(user_id: web::Path<(String,)>) -> impl Responder {
    let id: String = user_id.into_inner().0;
    HttpResponse::Ok().body(format!("Hello {}", id))
}

#[delete("/{user_id}")]
pub async fn delete(user_id: web::Path<(String,)>) -> impl Responder {
    let id: String = user_id.into_inner().0;
    HttpResponse::Ok().body(format!("Hello {}", id))
}
