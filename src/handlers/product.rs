use std::{env, path::Path};

use crate::{
    config::ApplicationConfiguration,
    contracts::{
        category::Category,
        product::{CategoryFilterParams, Product, ProductCreate, ProductStockUpdate, UploadForm},
        product_image::ProductImage,
        product_rating::{NewProductRating, ProductRating},
    },
    db::connection::{get_conn, SqliteConnectionPool},
    models::{
        category::Category as CategoryModel,
        product::{NewProduct, Product as ProductModel},
        product_image::{
            NewProductImage as NewProductImageModel, ProductImage as ProductImageModel,
        },
        product_rating::{
            NewProductRating as NewProductRatingModel, ProductRating as ProductRatingModel,
        },
    },
};
use ::uuid::Uuid;
use actix_multipart::form::MultipartForm;
use actix_web::{delete, get, http::StatusCode, patch, post, put, web, HttpResponse, Responder};
use diesel::prelude::*;

#[get("")]
pub async fn get(
    filters: web::Query<CategoryFilterParams>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    use crate::schema::categories;
    use crate::schema::categories::dsl::*;
    use crate::schema::products;
    use crate::schema::products::dsl::*;

    let conn = &mut get_conn(&pool);

    let mut query = products.inner_join(categories).into_boxed();

    if let Some(cid) = &filters.category_id {
        query = query.filter(categories::uuid.eq(cid));
    }

    let result = query
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
            (categories::uuid, categories::name),
        ))
        .load::<Product>(conn);

    match result {
        Ok(p) => HttpResponse::Ok().json(serde_json::json!({ "products": p })),
        Err(e) => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "message": e.to_string() })),
    }
}

#[post("")]
pub async fn create(
    MultipartForm(form): MultipartForm<ProductCreate>,
    app_config: web::Data<ApplicationConfiguration>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    use crate::schema::categories;
    use crate::schema::categories::dsl::*;
    use crate::schema::products::dsl::*;

    //get a pooled connection from db
    let conn = &mut get_conn(&pool);

    //check if the provided category exists or not
    let category: CategoryModel = match categories
        .filter(categories::uuid.eq(&form.category_id.0))
        .select(CategoryModel::as_select())
        .first::<CategoryModel>(conn)
        .optional()
    {
        Ok(Some(c)) => c,
        Ok(None) => {
            return HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "Category could not be found"}));
        }

        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}));
        }
    };

    let image_path = if let Some(thumbnail) = form.image {
        let full_path = Path::new(&env::current_dir().expect("Failed to get current directory"))
            .join(&app_config.product_thumbnail_path);

        if let Err(e) = std::fs::create_dir_all(full_path) {
            eprintln!("Failed to create directories: {:?}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "message": "Could not create image directory"
            }));
        }

        let path = format!(
            "{}product_{}_thumbnail.png",
            app_config.product_thumbnail_path,
            Uuid::new_v4().to_string().replace("-", "")
        );
        // Persist the file
        thumbnail.file.persist(&path).unwrap();
        Some(path)
    } else {
        None
    };

    // Now create the product, passing image_path (unwrap or default as needed)
    let product: NewProduct = NewProduct::new(
        form.name.0.to_owned(),
        form.description.0.to_owned(),
        image_path.unwrap_or_default(), // <- Here is your image path
        form.price.0,
        form.previous_price.0,
        form.unit.0.to_owned(),
        form.unit_change.0,
        form.stock.0,
        &category,
    );

    //insert the product to db
    match diesel::insert_into(products)
        .values(&product)
        .get_result::<ProductModel>(conn)
    {
        Ok(p) => HttpResponse::Ok()
            .status(StatusCode::OK)
            .json(serde_json::json!({"product": p.as_response(&category)})),
        Err(_) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message": "Ops! Something went wrong"})),
    }
}

#[get("/{product_id}")]
pub async fn get_product(
    product_id: web::Path<(String,)>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    let prod_uuid: String = product_id.into_inner().0;

    // I wonder if I should first validate the product_id
    use crate::schema::categories::dsl::*;
    use crate::schema::products;
    use crate::schema::products::dsl::*;

    //get a pooled connection from db
    let conn = &mut get_conn(&pool);

    match products
        .filter(products::uuid.eq(&prod_uuid))
        .select(ProductModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(prod) => match prod {
            Some(p) => {
                let category: CategoryModel =
                    categories.find(p.get_category_id()).first(conn).unwrap();
                HttpResponse::Ok()
                    .status(StatusCode::OK)
                    .json(serde_json::json!({"product": p.as_response(&category)}))
            }
            None => HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "Product not found. lol"})),
        },
        Err(_) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message": "Ops! something went wrong"})),
    }
}

#[put("/{product_id}")]
pub async fn edit(
    product_id: web::Path<(String,)>,
    pool: web::Data<SqliteConnectionPool>,
    MultipartForm(form): MultipartForm<ProductCreate>,
) -> impl Responder {
    let prod_uuid: String = product_id.into_inner().0;

    use crate::schema::categories;
    use crate::schema::categories::dsl::*;
    use crate::schema::products;
    use crate::schema::products::dsl::*;

    //get a pooled connection from db
    let conn = &mut get_conn(&pool);

    let product: ProductModel = match products
        .filter(products::uuid.eq(&prod_uuid))
        .select(ProductModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(prod) => match prod {
            Some(p) => p,
            None => {
                return HttpResponse::NotFound()
                    .status(StatusCode::NOT_FOUND)
                    .json(serde_json::json!({"message": "Product not found"}))
            }
        },
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong!"}))
        }
    };

    //validate if the category exists or not
    let category: CategoryModel = match categories
        .filter(categories::uuid.eq(&form.category_id.0))
        .select(CategoryModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(cat) => match cat {
            Some(c) => c,
            None => {
                return HttpResponse::BadRequest()
                    .status(StatusCode::BAD_REQUEST)
                    .json(serde_json::json!({"message": "Category not found"}))
            }
        },
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong!"}))
        }
    };

    //get the file from product.image and replace it with the incomming image. use same filename
    if let Some(new_thumbnail) = form.image {
        let cwd = &env::current_dir().expect("Failed to get current working directory");

        let relative_path = product.get_image().trim_start_matches('/');

        let full_path = Path::new(cwd).join(relative_path);
        println!("Attempting to save to: {:?}", full_path);
        if let Err(err) = new_thumbnail.file.persist(&full_path) {
            eprintln!("Failed to persist image: {:?}", err);
            return HttpResponse::InternalServerError().json(serde_json::json!({
                "message": "Could not save image"
            }));
        }
    }

    //zero validation are done for now
    match diesel::update(&product)
        .set((
            products::name.eq(&form.name.0),
            description.eq(&form.description.0),
            price.eq(&form.price.0),
            previous_price.eq(form.previous_price.0),
            unit.eq(&form.unit.0),
            unit_change.eq(form.unit_change.0),
            stock.eq(form.stock.0),
            category_id.eq(category.get_id()),
        ))
        .get_result::<ProductModel>(conn)
    {
        Ok(updated_product) => HttpResponse::Ok()
            .status(StatusCode::OK)
            .json(serde_json::json!({"product": updated_product.as_response(&category)})),
        Err(_) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message":"Ops! something went wrong!"})),
    }
}

#[patch("/{product_id}/category/update")]
pub async fn update_product_category(
    product_id: web::Path<(String,)>,
    category_update: web::Json<Category>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    let prod_uuid: String = product_id.into_inner().0;

    use crate::schema::categories::dsl::*;
    use crate::schema::products::dsl::*;
    use crate::schema::{categories, products};

    //get a pooled connection from db
    let conn = &mut get_conn(&pool);

    //check if the product_id is valid uuid or not before trip to db
    let prod_uuid: Uuid = match Uuid::parse_str(prod_uuid.as_str()) {
        Ok(u) => u,
        Err(_) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Invalid product id"}));
        }
    };

    let _cat_uuid: Uuid = match Uuid::parse_str(category_update.uuid.as_str()) {
        Ok(cu) => cu,
        Err(_) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Invalid category id"}));
        }
    };

    //first I have to get the category for the category uuid
    let category: CategoryModel = match categories
        .filter(categories::uuid.eq(&category_update.uuid))
        .select(CategoryModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(Some(c)) => c,
        Ok(None) => {
            return HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "Category not found"}));
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong."}));
        }
    };

    match diesel::update(products)
        .filter(products::uuid.eq(&prod_uuid.to_string()))
        .set(category_id.eq(&category.get_id()))
        .execute(conn)
    {
        Ok(urc) if urc > 0 => HttpResponse::Ok().status(StatusCode::OK).finish(),
        Ok(_) => HttpResponse::NotFound()
            .status(StatusCode::NOT_FOUND)
            .finish(),
        Err(_) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message": "Ops! something went wrong."})),
    }
}

#[patch("{product_id}/stock/update")]
pub async fn update_product_stock(
    product_id: web::Path<(String,)>,
    new_stock: web::Query<ProductStockUpdate>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    let prod_uuid: String = product_id.into_inner().0;

    //check if the product_id is valid uuid or not before trip to db
    let prod_uuid: Uuid = match Uuid::parse_str(prod_uuid.as_str()) {
        Ok(u) => u,
        Err(_) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Invalid product id"}));
        }
    };

    use crate::schema::products::dsl::*;

    //get a pooled connection from db
    let conn = &mut get_conn(&pool);

    //update the product's stock
    match diesel::update(products)
        .filter(uuid.eq(&prod_uuid.to_string()))
        .set(stock.eq(new_stock.stock))
        .execute(conn)
    {
        Ok(urc) if urc > 0 => HttpResponse::Ok().status(StatusCode::OK).finish(),
        Ok(_) => HttpResponse::NotFound()
            .status(StatusCode::NOT_FOUND)
            .json(serde_json::json!({"message": "Product not found"})),
        Err(_) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message": "Ops! something went wrong."})),
    }
}

#[delete("/{product_id}")]
pub async fn delete(_product_id: web::Path<(String,)>) -> impl Responder {
    HttpResponse::Ok().finish()
}

#[post("/{prod_id}/images")]
pub async fn upload_product_images(
    prod_id: web::Path<(String,)>,
    pool: web::Data<SqliteConnectionPool>,
    app_config: web::Data<ApplicationConfiguration>,
    MultipartForm(form): MultipartForm<UploadForm>,
) -> impl Responder {
    let prod_uuid: String = prod_id.into_inner().0;

    //check if the product_id is valid uuid or not before trip to db
    let prod_uuid: Uuid = match Uuid::parse_str(prod_uuid.as_str()) {
        Ok(u) => u,
        Err(_) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Invalid product id"}));
        }
    };

    use crate::schema::products;
    use crate::schema::products::dsl::*;

    //get a pooled connection from db
    let conn = &mut get_conn(&pool);

    //get the product for the uuid
    let product: ProductModel = match products
        .filter(products::uuid.eq(&prod_uuid.to_string()))
        .select(ProductModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(Some(p)) => p,
        Ok(None) => {
            return HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "Product not found"}));
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}));
        }
    };

    //handle the thumbnail image
    if let Some(thumbnail_image) = form.image {
        let path = format!(
            "{}product_{}_thumbnail.png",
            app_config.product_thumbnail_path,
            product.get_uuid(),
        );
        println!("{}", path);
        println!("Thumbnail image path is: {:?}", thumbnail_image.file.path());
        //might throw runtime exeception
        std::fs::copy(thumbnail_image.file.path(), &path).unwrap();
        std::fs::remove_file(thumbnail_image.file.path()).unwrap();
        //thumbnail_image.file.persist(path).unwrap();

        //save the file path in db
        match diesel
        ::update(&product)
            .set(image.eq(&path))
            .execute(conn)
        {
            Ok(urc) => {
                if urc == 0 {
                    return HttpResponse::InternalServerError().status(StatusCode::INTERNAL_SERVER_ERROR).json(serde_json::json!({"message": "ops! something went wrong while updating product thumbnail"}));
                }
            },
            Err(_) => return HttpResponse::InternalServerError().status(StatusCode::INTERNAL_SERVER_ERROR).json(serde_json::json!({"message": "ops! something went wrong while updating product thumbnail"}))
        };
    }
    //handle multiple images
    use crate::schema::product_images::dsl::*;

    println!("Multiple images code");
    println!("Server got {} images", form.images.len());
    for img in form.images {
        let path = format!(
            "{}image_{}_extra.png",
            app_config.product_extraimages_path,
            Uuid::new_v4()
        );

        //might throw runtime exeception
        std::fs::copy(img.file.path(), &path).unwrap();
        //f.file.persist(path).unwrap();

        //insert the image name into the db with product id
        let product_image = NewProductImageModel::new(&path, &product);
        match diesel::insert_into(product_images)
            .values(&product_image)
            .execute(conn)
        {
            Ok(_) => {}
            Err(_) => return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(
                    serde_json::json!({"message": "Ops! something went wrong while saving image"}),
                ),
        };
    }

    HttpResponse::Ok().json(serde_json::json!({"message": "Upload successful"}))
}

#[get("/{prod_id}/images")]
pub async fn get_product_images_list(
    prod_id: web::Path<(String,)>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    let prod_id: String = prod_id.into_inner().0;

    //check if the product_id is valid uuid or not before trip to db
    let prod_uuid: Uuid = match Uuid::parse_str(&prod_id) {
        Ok(u) => u,
        Err(_) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Invalid product id"}));
        }
    };

    use crate::schema::products::dsl::*;
    use crate::schema::{product_images, products};

    //get a pooled connection from db
    let conn = &mut get_conn(&pool);

    //get the product for the uuid
    let product: ProductModel = match products
        .filter(products::uuid.eq(&prod_uuid.to_string()))
        .select(ProductModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(Some(p)) => p,
        Ok(None) => {
            return HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "Product not found"}));
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}));
        }
    };

    let prod_images: Vec<ProductImage> = match ProductImageModel::belonging_to(&product)
        .select((product_images::uuid, product_images::image_name))
        .load::<ProductImage>(conn)
        .optional()
    {
        Ok(Some(pi)) => pi,
        Ok(None) => {
            return HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "Product image not found"}));
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}));
        }
    };

    HttpResponse::Ok()
        .status(StatusCode::OK)
        .json(serde_json::json!({"productImages": prod_images}))
}

#[post("/{prod_id}/rate")]
pub async fn rate_product(
    prod_id: web::Path<String>,
    rating_json: web::Json<NewProductRating>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    let prod_id: String = prod_id.into_inner();
    //check if the product_id is valid uuid or not before trip to db
    match Uuid::parse_str(&prod_id) {
        Ok(_) => (),
        Err(_) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Invalid product id"}));
        }
    };
    use crate::schema::product_ratings::dsl::*;
    use crate::schema::products::dsl::*;
    use crate::schema::{products, users};
    let conn = &mut get_conn(&pool);
    if rating_json.rating < 1.0 || rating_json.rating > 5.0 {
        return HttpResponse::BadRequest()
            .status(StatusCode::BAD_REQUEST)
            .json(serde_json::json!({"message": "Rating must be between 1 and 5"}));
    }
    //check if the product exists
    let product = match products
        .filter(products::uuid.eq(&prod_id))
        .select(ProductModel::as_select())
        .first::<ProductModel>(conn)
        .optional()
    {
        Ok(Some(p)) => p,
        Ok(None) => {
            return HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "Product not found"}));
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}));
        }
    };
    //check if the user exists
    let user = match users::table
        .filter(users::uuid.eq(&rating_json.user_id))
        .select(crate::models::user::User::as_select())
        .first::<crate::models::user::User>(conn)
        .optional()
    {
        Ok(Some(u)) => u,
        Ok(None) => {
            return HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "User not found"}));
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}));
        }
    };

    let new_rating = NewProductRatingModel::new(
        &product,
        &user,
        rating_json.rating,
        rating_json.review.clone(),
    );

    match diesel::insert_into(product_ratings)
        .values(&new_rating)
        .on_conflict((product_id, user_id)) // Composite key conflict
        .do_update()
        .set((
            rating.eq(rating_json.rating),
            review.eq(&rating_json.review),
            updated_at.eq(diesel::dsl::now),
        ))
        .execute(conn)
    {
        Ok(_) => {
            HttpResponse::Ok().json(serde_json::json!({"message": "Rating saved successfully"}))
        }
        Err(_) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"message": "Ops! something went wrong while saving rating"})),
    }
}

#[get("/{prod_id}/ratings")]
pub async fn get_product_ratings(
    prod_id: web::Path<String>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    let prod_id: String = prod_id.into_inner();

    //check if the product_id is valid uuid or not before trip to db
    let prod_uuid: Uuid = match Uuid::parse_str(&prod_id) {
        Ok(u) => u,
        Err(_) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Invalid product id"}));
        }
    };

    use crate::schema::products::dsl::*;
    use crate::schema::{product_ratings, products, users};

    //get a pooled connection from db
    let conn = &mut get_conn(&pool);

    //get the product for the uuid
    let product: ProductModel = match products
        .filter(products::uuid.eq(&prod_uuid.to_string()))
        .select(ProductModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(Some(p)) => p,
        Ok(None) => {
            return HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "Product not found"}));
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}));
        }
    };

    match ProductRatingModel::belonging_to(&product)
        .inner_join(users::table)
        .select((
            product_ratings::uuid,
            users::first_name,
            users::last_name,
            product_ratings::rating,
            product_ratings::review,
            product_ratings::created_at,
            product_ratings::updated_at,
        ))
        .load::<ProductRating>(conn)
        .optional()
    {
        Ok(r) => HttpResponse::Ok()
            .status(StatusCode::OK)
            .json(serde_json::json!({"ratings": r})),
        Err(_) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message": "Ops! something went wrong"})),
    }
}

#[get("/{prod_id}/rating/{user_id}")]
pub async fn get_user_product_rating(
    path: web::Path<(String, String)>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    let (prod_id, u_id) = path.into_inner();

    // Validate UUIDs
    if Uuid::parse_str(&prod_id).is_err() {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"message": "Invalid product id"}));
    }
    if Uuid::parse_str(&u_id).is_err() {
        return HttpResponse::BadRequest().json(serde_json::json!({"message": "Invalid user id"}));
    }

    use crate::schema::product_ratings::dsl::*;
    use crate::schema::{product_ratings, products, users};

    let conn = &mut get_conn(&pool);

    // Get user's rating for the product
    let user_rating = match product_ratings
        .inner_join(products::table.on(products::id.eq(product_id)))
        .inner_join(users::table.on(users::id.eq(user_id)))
        .filter(products::uuid.eq(&prod_id))
        .filter(users::uuid.eq(&u_id))
        .select((
            product_ratings::uuid,
            users::first_name,
            users::last_name,
            product_ratings::rating,
            product_ratings::review,
            product_ratings::created_at,
            product_ratings::updated_at,
        ))
        .first::<ProductRating>(conn)
        .optional()
    {
        Ok(Some(r)) => r,
        Ok(None) => {
            return HttpResponse::NotFound()
                .json(serde_json::json!({"message": "Rating not found"}));
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"message": "Ops! something went wrong"}));
        }
    };

    HttpResponse::Ok().json(serde_json::json!({"rating": user_rating}))
}
