use actix_web::{delete, get, http::StatusCode, post, put, web, HttpResponse, Responder};
use diesel::prelude::*;

use crate::{
    contracts::category::{Category, CategoryCreate},
    db::connection::{get_conn, SqliteConnectionPool},
    models::category::NewCategory,
};

#[post("")]
pub async fn create(
    category: web::Json<CategoryCreate>,
    db_conn_pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    use crate::schema::categories::dsl::*;
    //check if category already exists
    match categories
        .filter(name.like(format!("%{}%", category.name.to_owned())))
        .select(name)
        .first::<String>(&mut get_conn(&db_conn_pool))
        .optional()
    {
        Ok(cat) => match cat {
            Some(c) => HttpResponse::Conflict().json(
                serde_json::json!({"status": "fail","message": format!("Category with name {} aready exists", c)}),
            ),
            None => {
                let category: NewCategory = NewCategory::new(category.name.to_owned());
                match diesel::insert_into(categories)
                    .values(&category)
                    .execute(&mut get_conn(&db_conn_pool)) {
                        Ok(_) => HttpResponse::Ok().json(category),
                        Err(_) => HttpResponse::InternalServerError().finish()
                    }
            }
        },
        Err(e) => 
             HttpResponse::InternalServerError()
                .body(format!("Ops! something went wrong: {}", e))
        
    }
}

#[get("")]
pub async fn get(pool: web::Data<SqliteConnectionPool>) -> impl Responder {
    use crate::schema::categories::dsl::*;
    let categories_vec = categories
        .select((uuid, name))
        .load::<Category>(&mut get_conn(&pool));

    match categories_vec {
        Ok(cat_v) => HttpResponse::Ok().json(serde_json::json!({"categories": cat_v})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"message": e.to_string()}))
        
    }
}

#[get("/{catgory_id}")]
pub async fn get_category(
    category_id: web::Path<(String,)>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    use crate::schema::categories::dsl::*;
    let uid: String = category_id.into_inner().0; //uid = uuid
    match categories
        .filter(uuid.eq(uid))
        .select((uuid, name))
        .first::<Category>(&mut get_conn(&pool))
        .optional()
    {
        Ok(cat) => match cat {
            Some(c) => HttpResponse::Ok().status(StatusCode::OK).json(c),
            None => HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message":"Category not found"})),
        },
        Err(e) => HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("Ops! something went wrong: {}", e))
        
    }
}

#[put("/{category_id}")]
pub async fn edit(
    category_id: web::Path<(String,)>,
    category_update: web::Json<CategoryCreate>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    use crate::schema::categories::dsl::*;
    let uid: String = category_id.into_inner().0;
    match diesel::update(categories)
        .filter(uuid.eq(&uid))
        .set(name.eq(category_update.name.to_owned()))
        .execute(&mut get_conn(&pool)) 
        //all this match can be shortened if used get_result() but how?
    {
        Ok(urc) if urc > 0 => {
            //urc = updated row count
            let category = Category {
                uuid: uid,
                name: category_update.name.to_owned(),
            };
            HttpResponse::Ok()
                .status(StatusCode::OK)
                .json(category)
        }
        Ok(_) => 
            //match branch if the value is <= 0
            HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .finish(),
        
        Err(e) => 
             HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": format!("Ops! something went wrong: {}", e)}))
        
    }
}

#[delete("/{category_id}")]
pub async fn delete(
    category_id: web::Path<(String,)>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    use crate::schema::categories::dsl::*;
    let uid: String = category_id.into_inner().0;
    match diesel::delete(categories.filter(uuid.eq(uid)))
        .execute(&mut get_conn(&pool))
    {
        //drc = deleted_row_count
        //execute() function returns the number of row affected
        Ok(drc) if drc > 0 => HttpResponse::Ok().status(StatusCode::OK).finish(),

        //if the drc <= 0 then no row is affected meaning deletetion not successfull. 
        //Why? because the resource is not found with that uuid
        Ok(_) => HttpResponse::NotFound().status(StatusCode::NOT_FOUND).finish(),
        
        Err(e) => 
            HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": format!("Ops! something went wrong: {}", e)}))
        
    }
}
