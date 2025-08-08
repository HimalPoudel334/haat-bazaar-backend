// @generated automatically by Diesel CLI.

diesel::table! {
    admin_devices (id) {
        id -> Integer,
        uuid -> Text,
        user_id -> Integer,
        fcm_token -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    carts (id) {
        id -> Integer,
        uuid -> Text,
        product_id -> Integer,
        user_id -> Integer,
        quantity -> Double,
        sku -> Text,
        created_on -> Text,
        discount -> Double,
    }
}

diesel::table! {
    categories (id) {
        id -> Integer,
        uuid -> Text,
        name -> Text,
    }
}

diesel::table! {
    invoice_items (id) {
        id -> Integer,
        uuid -> Text,
        quantity -> Double,
        unit_price -> Double,
        discount_percent -> Double,
        discount_amount -> Double,
        total -> Double,
        product_id -> Integer,
        invoice_id -> Integer,
    }
}

diesel::table! {
    invoices (id) {
        id -> Integer,
        uuid -> Text,
        invoice_number -> Integer,
        invoice_date -> Text,
        sub_total -> Double,
        vat_percent -> Double,
        vat_amount -> Double,
        net_amount -> Double,
        order_id -> Integer,
        user_id -> Integer,
        payment_id -> Integer,
    }
}

diesel::table! {
    order_items (id) {
        id -> Integer,
        uuid -> Text,
        product_id -> Integer,
        order_id -> Integer,
        quantity -> Double,
        price -> Double,
        discount -> Double,
        amount -> Double,
    }
}

diesel::table! {
    orders (id) {
        id -> Integer,
        uuid -> Text,
        created_on -> Text,
        fulfilled_on -> Text,
        delivery_charge -> Double,
        delivery_location -> Text,
        delivery_status -> Text,
        total_price -> Double,
        user_id -> Integer,
        quantity -> Double,
        status -> Text,
        discount -> Double,
        amount -> Double,
    }
}

diesel::table! {
    password_reset_otps (id) {
        id -> Integer,
        user_id -> Integer,
        otp_code -> Text,
        expires_at -> Text,
        is_used -> Bool,
        attempts -> Integer,
        created_at -> Text,
    }
}

diesel::table! {
    payments (id) {
        id -> Integer,
        uuid -> Text,
        pay_date -> Text,
        amount -> Double,
        payment_method -> Text,
        user_id -> Integer,
        order_id -> Integer,
        transaction_id -> Text,
        tendered -> Double,
        change -> Double,
        discount -> Double,
        status -> Text,
        service_charge -> Double,
        refunded -> Bool,
    }
}

diesel::table! {
    product_images (id) {
        id -> Integer,
        uuid -> Text,
        image_name -> Text,
        product_id -> Integer,
    }
}

diesel::table! {
    products (id) {
        id -> Integer,
        uuid -> Text,
        name -> Text,
        description -> Text,
        image -> Text,
        price -> Double,
        previous_price -> Double,
        unit -> Text,
        unit_change -> Double,
        stock -> Double,
        category_id -> Integer,
    }
}

diesel::table! {
    refresh_tokens (id) {
        id -> Integer,
        uuid -> Text,
        token -> Text,
        user_id -> Integer,
        expires_on -> Text,
    }
}

diesel::table! {
    shipments (id) {
        id -> Integer,
        uuid -> Text,
        ship_date -> Text,
        address -> Text,
        city -> Text,
        state -> Text,
        country -> Text,
        zip_code -> Text,
        order_id -> Integer,
        status -> Text,
        assigned_to -> Nullable<Integer>,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        uuid -> Text,
        first_name -> Text,
        last_name -> Text,
        phone_number -> Text,
        email -> Text,
        password -> Text,
        user_type -> Text,
        location -> Nullable<Text>,
        nearest_landmark -> Nullable<Text>,
    }
}

diesel::joinable!(admin_devices -> users (user_id));
diesel::joinable!(carts -> products (product_id));
diesel::joinable!(carts -> users (user_id));
diesel::joinable!(invoice_items -> invoices (invoice_id));
diesel::joinable!(invoice_items -> products (product_id));
diesel::joinable!(invoices -> orders (order_id));
diesel::joinable!(invoices -> payments (payment_id));
diesel::joinable!(invoices -> users (user_id));
diesel::joinable!(order_items -> orders (order_id));
diesel::joinable!(order_items -> products (product_id));
diesel::joinable!(orders -> users (user_id));
diesel::joinable!(password_reset_otps -> users (user_id));
diesel::joinable!(payments -> orders (order_id));
diesel::joinable!(payments -> users (user_id));
diesel::joinable!(product_images -> products (product_id));
diesel::joinable!(products -> categories (category_id));
diesel::joinable!(refresh_tokens -> users (user_id));
diesel::joinable!(shipments -> orders (order_id));
diesel::joinable!(shipments -> users (assigned_to));

diesel::allow_tables_to_appear_in_same_query!(
    admin_devices,
    carts,
    categories,
    invoice_items,
    invoices,
    order_items,
    orders,
    password_reset_otps,
    payments,
    product_images,
    products,
    refresh_tokens,
    shipments,
    users,
);
