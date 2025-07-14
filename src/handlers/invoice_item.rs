use actix_web::{http::StatusCode, web, HttpResponse, Responder};
use diesel::prelude::*;
use uuid::Uuid;

use crate::{
    contracts::invoice_item::NewInvoiceItem,
    db::connection::{get_conn, SqliteConnectionPool},
    models::{
        invoice::Invoice as InvoiceModel, invoice_item::NewInvoiceItem as NewInvoiceItemModel,
        product::Product as ProductModel,
    },
};

pub fn add(
    inv_id: web::Path<(String,)>,
    inv_item: web::Json<NewInvoiceItem>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    let inv_id: String = inv_id.into_inner().0;

    //check if it is a valid uuid or not
    let inv_id: Uuid = match Uuid::parse_str(&inv_id) {
        Ok(i) => i,
        Err(_) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Invalid invoice id"}))
        }
    };

    //get a connection from db pool
    let conn = &mut get_conn(&pool);

    use crate::schema::invoice_items::dsl::*;
    use crate::schema::invoices::dsl::*;
    use crate::schema::products::dsl::*;
    use crate::schema::{invoices, products};

    let invoice: InvoiceModel = match invoices
        .filter(invoices::uuid.eq(&inv_id.to_string()))
        .select(InvoiceModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(Some(i)) => i,
        Ok(None) => {
            return HttpResponse::NotFound()
                .status(StatusCode::NOT_FOUND)
                .json(serde_json::json!({"message": "Invoice not found"}))
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}))
        }
    };

    //check if product exists or not
    let product: ProductModel = match products
        .filter(products::uuid.eq(&inv_item.product_id))
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

    if product.get_stock() < inv_item.quantity {
        return HttpResponse::BadRequest()
            .status(StatusCode::BAD_REQUEST)
            .json(serde_json::json!({"message": "Product quantity is more than stock"}));
    }

    // Handle discount amount and discount percent
    let mut dis_percent: f64 = 0.0;
    let mut dis_amt: f64 = 0.0;

    if let Some(d_a) = inv_item.discount_amount {
        dis_percent = d_a / product.get_price() * 100.0;
    }

    if let Some(d_p) = inv_item.discount_percent {
        dis_amt = product.get_price() * d_p / 100.0;
    }

    // Check if either discount percent or discount amount has been set
    if dis_percent == 0.0 && dis_amt == 0.0 {
        return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Either discount percent or discount amount has to be set"}));
    }

    let inv_item_model: NewInvoiceItemModel =
        NewInvoiceItemModel::new(&product, &invoice, inv_item.quantity, dis_percent, dis_amt);

    match diesel::insert_into(invoice_items)
        .values(&inv_item_model)
        .execute(conn)
    {
        Ok(_) => {
            match diesel::update(&product)
                .set(stock.eq(stock - inv_item.quantity))
                .execute(conn)
            {
                Ok(_) => HttpResponse::Ok()
                    .status(StatusCode::OK)
                    .json(serde_json::json!({"message": "Invoice added successfully"})),
                Err(_) => HttpResponse::InternalServerError()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .json(serde_json::json!({"message": "Ops! something went wrong while updating product"})),
            }
        }
        Err(_) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(
                serde_json::json!({"message": "Ops! something went wrong while updating product"}),
            ),
    }
}
