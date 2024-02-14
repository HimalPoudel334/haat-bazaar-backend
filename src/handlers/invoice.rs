use actix_web::{http::StatusCode, web, HttpResponse, Responder};
use diesel::prelude::*;

use crate::{
    contracts::invoice::NewInvoice,
    db::connection::{get_conn, SqliteConnectionPool},
    models::{
        customer::Customer as CustomerModel,
        invoice::{Invoice as InvoiceModel, NewInvoice as NewInvoiceModel},
        invoice_item::NewInvoiceItem as NewInvoiceItemModel,
        order::Order as OrderModel,
        payment::Payment as PaymentModel,
        product::Product as ProductModel,
    },
};

pub fn create(
    inv_json: web::Json<NewInvoice>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    use crate::schema::customers::dsl::*;
    use crate::schema::invoice_items::dsl::*;
    use crate::schema::invoices::dsl::*;
    use crate::schema::orders::dsl::*;
    use crate::schema::payments::dsl::*;
    use crate::schema::products::dsl::*;
    use crate::schema::{orders, products};

    //get a database connection for pool
    let conn = &mut get_conn(&pool);

    //first check if order exists or not
    let order: OrderModel = match orders
        .filter(orders::uuid.eq(&inv_json.order_id))
        .select(OrderModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(Some(o)) => o,
        Ok(None) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Order not found"}))
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}))
        }
    };

    //check if payment is done or not
    let payment: PaymentModel = match payments
        .find(order.get_id())
        .select(PaymentModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(Some(p)) => {
            if p.get_uuid().eq(&inv_json.payment_id) {
                p
            } else {
                return HttpResponse::BadRequest()
                    .status(StatusCode::BAD_REQUEST)
                    .json(serde_json::json!({"message": "Invalid payment id provided"}));
            }
        }
        Ok(None) => return HttpResponse::BadRequest()
            .status(StatusCode::BAD_REQUEST)
            .json(
                serde_json::json!({"message": "Payment not done for the order. Please pay first"}),
            ),
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}))
        }
    };

    let customer: CustomerModel = match customers
        .find(order.get_customer_id())
        .select(CustomerModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(Some(c)) => {
            if c.get_uuid().eq(&inv_json.customer_id) {
                c
            } else {
                return HttpResponse::BadRequest()
                    .status(StatusCode::BAD_REQUEST)
                    .json(serde_json::json!({"message": "Invalid customer id provided"}));
            }
        }
        Ok(None) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Customer not found"}))
        }
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}))
        }
    };

    let new_invoice: NewInvoiceModel = NewInvoiceModel::new(
        &inv_json.invoice_date,
        inv_json.sub_total,
        inv_json.vat_percent,
        &order,
        &customer,
        &payment,
    );

    match diesel::insert_into(invoices)
        .values(&new_invoice)
        .get_result::<InvoiceModel>(conn)
    {
        Ok(inv) => {
            for inv_item in &inv_json.invoice_items {
                let prod: ProductModel = match products
                    .filter(products::uuid.eq(&inv_item.product_id))
                    .select(ProductModel::as_select())
                    .first(conn)
                    .optional()
                {
                    Ok(Some(p)) => p,
                    Ok(None) => {
                        return HttpResponse::BadRequest()
                            .status(StatusCode::BAD_REQUEST)
                            .json(serde_json::json!({"message": "Product not found"}))
                    }
                    Err(_) => {
                        return HttpResponse::InternalServerError()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .json(serde_json::json!({"message": "Ops! something went wrong"}))
                    }
                };

                // Handle discount amount and discount percent
                let mut dis_percent: f64 = 0.0;
                let mut dis_amt: f64 = 0.0;

                if let Some(d_a) = inv_item.discount_amount {
                    dis_percent = d_a / prod.get_price() * 100.0;
                } 

                if let Some(d_p) = inv_item.discount_percent {
                    dis_amt = prod.get_price() * d_p / 100.0;
                } 

                // Check if either discount percent or discount amount has been set
                if dis_percent == 0.0 && dis_amt == 0.0 {
                    return HttpResponse::BadRequest()
                            .status(StatusCode::BAD_REQUEST)
                            .json(serde_json::json!({"message": "Either discount percent or discount amount has to be set"}));
                }

                let inv_item_model: NewInvoiceItemModel =
                    NewInvoiceItemModel::new(&prod, &inv, inv_item.quantity, dis_percent, dis_amt);

                diesel::insert_into(invoice_items)
                    .values(&inv_item_model)
                    .execute(conn)
                    .unwrap();
            }
            HttpResponse::Ok().into()
        }
        Err(_) => return HttpResponse::InternalServerError()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .json(serde_json::json!({"message": "Ops! something went wrong while inserting invoice item"}))

    }
}
