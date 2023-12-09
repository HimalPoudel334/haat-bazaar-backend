use actix_web::{delete, get, http::StatusCode, post, put, web, HttpResponse, Responder};
use diesel::prelude::*;
use uuid::Uuid;

use crate::{
    base_types::phone_number::PhoneNumber,
    contracts::customer::{Customer, CustomerCreate},
    db::connection::{get_conn, SqliteConnectionPool},
    models::customer::{Customer as CustomerModel, NewCustomer},
    utils::password_helper::hash_password,
};

#[get("")]
pub async fn get(pool: web::Data<SqliteConnectionPool>) -> impl Responder {
    use crate::schema::customers::dsl::*;
    let customers_vec = customers
        .select((uuid, first_name, last_name, phone_number))
        .load::<Customer>(&mut get_conn(&pool));

    match customers_vec {
        Ok(cat_v) => HttpResponse::Ok().status(StatusCode::OK).json(cat_v),
        Err(e) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message": e.to_string()})),
    }
}

#[post("")]
pub async fn create(
    customer: web::Json<CustomerCreate>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    use crate::schema::customers::dsl::*;
    let phone_num: PhoneNumber = match PhoneNumber::from_str(customer.phone_number.to_owned()) {
        Ok(phone) => phone,
        Err(e) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": e}));
        }
    };

    match customers
        .filter(phone_number.eq(&customer.phone_number))
        .select(CustomerModel::as_select())
        .first(&mut get_conn(&pool))
        .optional()
    {
        Ok(Some(_)) => HttpResponse::Conflict()
            .status(StatusCode::CONFLICT)
            .json(serde_json::json!({"message": "Phone number already used"})),
        Ok(None) => {
            let new_customer = NewCustomer::new(
                customer.first_name.to_owned(),
                customer.last_name.to_owned(),
                phone_num,
                hash_password(&customer.password),
            );

            match diesel::insert_into(customers)
                .values(&new_customer)
                .get_result::<CustomerModel>(&mut get_conn(&pool))
            {
                Ok(c) => {
                    let customer_created: Customer = Customer {
                        first_name: c.get_first_name().to_owned(),
                        last_name: c.get_last_name().to_owned(),
                        uuid: c.get_uuid().to_owned(),
                        phone_number: c.get_phone_number().to_owned(),
                    };
                    HttpResponse::Ok()
                        .status(StatusCode::OK)
                        .json(customer_created)
                }
                Err(e) => HttpResponse::InternalServerError()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .json(serde_json::json!({"message": format!("{}", e)})),
            }
        }
        Err(e) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message": format!("{}", e)})),
    }
}

#[get("/{user_id}")]
pub async fn get_customer(
    user_id: web::Path<(String,)>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    let uid: String = user_id.into_inner().0;
    //check if the user_id is valid uuid or not before trip to db
    let uid: Uuid = match Uuid::parse_str(uid.as_str()) {
        Ok(u) => u,
        Err(_) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Invalid user id"}));
        }
    };

    use crate::schema::customers::dsl::*;

    match customers
        .filter(uuid.eq(uid.to_string()))
        .select(Customer::as_select())
        .first(&mut get_conn(&pool))
        .optional()
    {
        Ok(cust) => match cust {
            Some(c) => HttpResponse::Ok().status(StatusCode::OK).json(c),
            None => HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "Customer not found"})),
        },
        Err(e) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message": format!("Internal server error: {}", e)})),
    }
}

#[get("/phone-number/{phone_num}")]
pub async fn get_customer_from_phone_number(
    phone_num: web::Path<(String,)>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    let phone_num_string: String = phone_num.into_inner().0;
    //check if the user_id is valid uuid or not before trip to db
    let ph_num: PhoneNumber = match PhoneNumber::from_str(phone_num_string) {
        Ok(p) => p,
        Err(e) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": e}));
        }
    };

    use crate::schema::customers::dsl::*;

    match customers
        .filter(phone_number.eq(&ph_num.get_number()))
        .select(Customer::as_select())
        .first(&mut get_conn(&pool))
        .optional()
    {
        Ok(cust) => match cust {
            Some(c) => HttpResponse::Ok().status(StatusCode::OK).json(c),
            None => HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "Customer not found"})),
        },
        Err(e) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message": format!("Internal server error: {}", e)})),
    }
}

#[put("/{user_id}")]
pub async fn edit(
    user_id: web::Path<(String,)>,
    customer_update: web::Json<Customer>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    let uid: String = user_id.into_inner().0;
    //check if the user_id is valid uuid or not before trip to db
    let uid: Uuid = match Uuid::parse_str(uid.as_str()) {
        Ok(u) => u,
        Err(_) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Invalid user id"}));
        }
    };

    //check if the phone number is valid or not
    let _ = match PhoneNumber::from_str(customer_update.phone_number.to_owned()) {
        Ok(_) => (),
        Err(e) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": e}));
        }
    };

    use crate::schema::customers::dsl::*;

    //check if the new phone number is already used or not
    let customer_from_phone = match customers
        .filter(phone_number.eq(customer_update.phone_number.to_string()))
        .select(CustomerModel::as_select())
        .first(&mut get_conn(&pool))
        .optional()
    {
        Ok(cu) => match cu {
            Some(c) => c,
            None => Default::default(), //return the default Customer struct if not found
        },
        Err(e) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": format!("Internal server error: {}", e)}));
        }
    };

    let customer: CustomerModel = match customers
        .filter(uuid.eq(uid.to_string()))
        .select(CustomerModel::as_select())
        .first(&mut get_conn(&pool))
        .optional()
    {
        Ok(cu) => match cu {
            Some(c) => {
                if c == customer_from_phone {
                    c
                } else {
                    return HttpResponse::Conflict()
                        .status(StatusCode::CONFLICT)
                        .json(serde_json::json!({"message": "Phone number already used"}));
                }
            }
            None => {
                return HttpResponse::NotFound()
                    .status(StatusCode::NOT_FOUND)
                    .json(serde_json::json!({"message": "Customer not found"}));
            }
        },
        Err(e) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": format!("Internal server error: {}", e)}));
        }
    };

    match diesel::update(&customer)
        .set((
            first_name.eq(&customer_update.first_name),
            last_name.eq(&customer_update.last_name),
            phone_number.eq(&customer_update.phone_number),
        ))
        .execute(&mut get_conn(&pool))
    {
        Ok(_) => HttpResponse::Ok()
            .status(StatusCode::OK)
            .json(customer_update),
        Err(e) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message": format!("Internal server error: {}", e)})),
    }
}

#[delete("/{user_id}")]
pub async fn delete(user_id: web::Path<(String,)>) -> impl Responder {
    let _id: String = user_id.into_inner().0;
    HttpResponse::BadRequest()
        .status(StatusCode::BAD_REQUEST)
        .json(serde_json::json!({"message": "Cannot delete a resource"}))
}
