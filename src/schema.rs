// @generated automatically by Diesel CLI.

diesel::table! {
    carts (id) {
        id -> Integer,
        uuid -> Text,
        product_id -> Integer,
        customer_id -> Integer,
        quantity -> Double,
        created_on -> Text,
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
    customers (id) {
        id -> Integer,
        uuid -> Text,
        first_name -> Text,
        last_name -> Text,
        phone_number -> Text,
        password -> Text,
    }
}

diesel::table! {
    order_details (id) {
        id -> Integer,
        uuid -> Text,
        product_id -> Integer,
        order_id -> Integer,
        quantity -> Double,
        price -> Double,
    }
}

diesel::table! {
    orders (id) {
        id -> Integer,
        uuid -> Text,
        created_on -> Text,
        fulfilled_on -> Text,
        delivery_location -> Text,
        delivery_status -> Text,
        total_price -> Double,
        customer_id -> Integer,
    }
}

diesel::table! {
    payments (id) {
        id -> Integer,
        uuid -> Text,
        pay_date -> Text,
        amount -> Double,
        payment_method -> Text,
        customer_id -> Integer,
        order_id -> Integer,
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
    }
}

diesel::joinable!(carts -> customers (customer_id));
diesel::joinable!(carts -> products (product_id));
diesel::joinable!(order_details -> orders (order_id));
diesel::joinable!(order_details -> products (product_id));
diesel::joinable!(orders -> customers (customer_id));
diesel::joinable!(payments -> customers (customer_id));
diesel::joinable!(payments -> orders (order_id));
diesel::joinable!(product_images -> products (product_id));
diesel::joinable!(products -> categories (category_id));
diesel::joinable!(shipments -> orders (order_id));

diesel::allow_tables_to_appear_in_same_query!(
    carts,
    categories,
    customers,
    order_details,
    orders,
    payments,
    product_images,
    products,
    shipments,
);
