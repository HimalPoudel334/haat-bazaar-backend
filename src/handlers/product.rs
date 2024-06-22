use ::uuid::Uuid;
use actix_multipart::form::MultipartForm;
use actix_web::{delete, get, http::StatusCode, patch, post, put, web, HttpResponse, Responder};
use diesel::prelude::*;

use crate::contracts::product_images::ProductImage;
use crate::models::product_image::{NewProductImage, ProductImage as ProductImageModel};
use crate::{
    config::ApplicationConfiguration,
    contracts::{
        category::Category,
        product::{Product, ProductCreate, ProductStockUpdate, UploadForm},
    },
    db::connection::{get_conn, SqliteConnectionPool},
    models::{
        category::Category as CategoryModel,
        product::{NewProduct, Product as ProductModel},
    },
};

#[get("")]
pub async fn get(pool: web::Data<SqliteConnectionPool>) -> impl Responder {
    use crate::schema::categories;
    use crate::schema::categories::dsl::*;
    use crate::schema::products;
    use crate::schema::products::dsl::*;

    //get a pooled connection from db
    let conn = &mut get_conn(&pool);

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
        .load::<Product>(conn);

    match product_vec {
        Ok(p) => HttpResponse::Ok()
            .status(StatusCode::OK)
            .json(serde_json::json!({"products": p})),
        Err(e) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(serde_json::json!({"message": e.to_string()})),
    }
}

#[post("")]
pub async fn create(
    product_json: web::Json<ProductCreate>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    use crate::schema::categories;
    use crate::schema::categories::dsl::*;
    use crate::schema::products::dsl::*;

    //get a pooled connection from db
    let conn = &mut get_conn(&pool);

    //check if the provided category exists or not
    let category: CategoryModel = match categories
        .filter(categories::uuid.eq(&product_json.category_id))
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
        .get_result::<ProductModel>(conn)
    {
        Ok(p) => HttpResponse::Ok()
            .status(StatusCode::OK)
            .json(p.as_response(&category)),
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
                    .json(p.as_response(&category))
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
    product_json: web::Json<Product>,
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
        .filter(categories::uuid.eq(&product_json.category_id))
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

    //zero validation are done for now
    match diesel::update(&product)
        .set((
            products::name.eq(&product_json.name),
            description.eq(&product_json.description),
            image.eq(&product_json.image),
            price.eq(&product_json.price),
            previous_price.eq(product_json.previous_price),
            unit.eq(&product_json.unit),
            unit_change.eq(product_json.unit_change),
            stock.eq(product_json.stock),
            category_id.eq(category.get_id()),
        ))
        .get_result::<ProductModel>(conn)
    {
        Ok(updated_product) => HttpResponse::Ok()
            .status(StatusCode::OK)
            .json(updated_product.as_response(&category)),
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

    let _cat_uuid: Uuid = match Uuid::parse_str(&category_update.uuid.as_str()) {
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
                if urc <= 0 {
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
            Uuid::new_v4().to_string()
        );

        //might throw runtime exeception
        std::fs::copy(img.file.path(), &path).unwrap();
        //f.file.persist(path).unwrap();

        //insert the image name into the db with product id
        let product_image: NewProductImage = NewProductImage::new(&path, &product);
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
        .json(serde_json::json!({"product_images": prod_images}))
}
