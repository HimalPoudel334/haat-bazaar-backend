use actix_web::{get, http::StatusCode, post, web, HttpResponse, Responder};
use diesel::prelude::*;
use uuid::Uuid;

use crate::{
    contracts::{
        invoice::{Invoice, InvoiceOnly, NewInvoice},
        invoice_item::InvoiceItem,
    },
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

#[post("")]
pub async fn create(
    inv_json: web::Json<NewInvoice>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    use crate::schema::customers::dsl::*;
    use crate::schema::invoice_items::dsl::*;
    use crate::schema::invoices::dsl::*;
    use crate::schema::orders::dsl::*;
    use crate::schema::payments::dsl::*;
    use crate::schema::products::dsl::*;
    use crate::schema::{orders, payments, products};

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

    if order.get_total_price() != inv_json.sub_total {
        return HttpResponse::BadRequest()
            .status(StatusCode::BAD_REQUEST)
            .json(serde_json::json!({"message": "Invoice total and order total do not match"}));
    }

    //check if payment is done or not
    let payment: PaymentModel = match payments
        .filter(payments::order_id.eq(order.get_id()))
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
            if inv_json.invoice_items.is_empty() {
                return HttpResponse::BadRequest()
                    .status(StatusCode::BAD_REQUEST)
                    .json(serde_json::json!({"message": "Error! no invoice items provided"}));
            } else {
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

                    if prod.get_stock() < inv_item.quantity {
                        return HttpResponse::BadRequest()
                            .status(StatusCode::BAD_REQUEST)
                            .json(serde_json::json!({"message": "Product quantity is more than stock"}));
                    }

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

                    let inv_item_model: NewInvoiceItemModel = NewInvoiceItemModel::new(
                        &prod,
                        &inv,
                        inv_item.quantity,
                        dis_percent,
                        dis_amt,
                    );

                    match diesel::insert_into(invoice_items)
                        .values(&inv_item_model)
                        .execute(conn)
                        {
                            Ok(_) => {},
                            Err(_) => return HttpResponse::InternalServerError()
                                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                                        .json(serde_json::json!({"message": "Ops! something went wrong while inserting invoice item"}))
                        }
                }
                let redirect_url: String = format!("invoices/get/{}", inv.uuid().to_owned());
                HttpResponse::SeeOther()
                    .append_header((actix_web::http::header::LOCATION, redirect_url))
                    .finish()
            }
        }
        Err(_) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(
                serde_json::json!({"message": "Ops! something went wrong while inserting invoice"}),
            ),
    }
}

#[get("/{inv_id}")]
pub async fn get(
    inv_id: web::Path<(String,)>,
    pool: web::Data<SqliteConnectionPool>,
) -> impl Responder {
    let inv_id: String = inv_id.into_inner().0;

    let inv_uuid: Uuid = match Uuid::parse_str(&inv_id) {
        Ok(uid) => uid,
        Err(_) => {
            return HttpResponse::BadRequest()
                .status(StatusCode::BAD_REQUEST)
                .json(serde_json::json!({"message": "Invalid invoice id"}))
        }
    };

    let conn = &mut get_conn(&pool);

    use crate::schema::customers::dsl::*;
    use crate::schema::invoice_items::dsl::*;
    use crate::schema::invoices::dsl::*;
    use crate::schema::orders::dsl::*;
    use crate::schema::payments::dsl::*;
    use crate::schema::products::dsl::*;
    use crate::schema::{invoice_items, invoices, products};

    let invoice: InvoiceModel = match invoices
        .filter(invoices::uuid.eq(&inv_uuid.to_string()))
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
        Err(_) => return HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(
                serde_json::json!({"message": "Ops! something went wrong while searching invoice"}),
            ),
    };

    // I think we can simply unwrap these entities.
    let order: OrderModel = match orders
        .find(invoice.order_id())
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

    let payment: PaymentModel = match payments
        .find(invoice.order_id())
        .select(PaymentModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(Some(p)) => p,
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
        .find(invoice.customer_id())
        .select(CustomerModel::as_select())
        .first(conn)
        .optional()
    {
        Ok(Some(c)) => c,
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

    let inv_items: Vec<InvoiceItem> = match invoice_items
        .inner_join(invoices)
        .inner_join(products)
        .select((
            invoice_items::uuid,
            products::uuid,
            products::name,
            invoices::uuid,
            invoice_items::quantity,
            invoice_items::unit_price,
            invoice_items::discount_percent,
            invoice_items::discount_amount,
            invoice_items::total,
        ))
        .load::<InvoiceItem>(conn)
    {
        Ok(v) => v,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .json(serde_json::json!({"message": "Ops! something went wrong"}))
        }
    };

    let inv_vm: Invoice = Invoice {
        uuid: invoice.uuid().to_owned(),
        invoice_number: invoice.invoice_number(),
        invoice_date: invoice.invoice_date().to_owned(),
        sub_total: invoice.sub_total(),
        vat_percent: invoice.vat_percent(),
        vat_amount: invoice.vat_amount(),
        net_amount: invoice.net_amount(),
        order_id: order.get_uuid().to_owned(),
        customer_id: customer.get_uuid().to_owned(),
        customer_name: customer.get_full_name().to_owned(),
        payment_id: payment.get_uuid().to_owned(),
        invoice_items: inv_items,
    };

    HttpResponse::Ok().status(StatusCode::OK).json(inv_vm)
}

#[get("")]
pub async fn get_all(pool: web::Data<SqliteConnectionPool>) -> impl Responder {
    let conn = &mut get_conn(&pool);
    use crate::schema::customers::dsl::*;
    use crate::schema::invoices::dsl::*;
    use crate::schema::orders::dsl::*;
    use crate::schema::payments::dsl::*;
    use crate::schema::{customers, invoices, orders, payments};

    match invoices
        .inner_join(customers)
        .inner_join(orders)
        .inner_join(payments)
        .select((
            invoices::uuid,
            invoice_number,
            invoice_date,
            customers::first_name
                .concat(" ")
                .concat(customers::last_name),
            sub_total,
            vat_percent,
            vat_amount,
            net_amount,
            orders::uuid,
            customers::uuid,
            payments::uuid,
        ))
        .load::<InvoiceOnly>(conn)
    {
        Ok(i) => HttpResponse::Ok()
            .status(StatusCode::OK)
            .json(serde_json::json!({"invoices": i})),
        Err(_) => HttpResponse::InternalServerError()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .json(
                serde_json::json!({"message": "Ops! something went wrong while fetching invoices"}),
            ),
    }
}
