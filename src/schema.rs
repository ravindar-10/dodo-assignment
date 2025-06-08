// @generated automatically by Diesel CLI.

diesel::table! {
    balances (user_id) {
        user_id -> Int4,
        balance -> Numeric,
    }
}

diesel::table! {
    otps (id) {
        id -> Int4,
        user_id -> Int4,
        #[max_length = 6]
        otp -> Varchar,
        is_valid -> Nullable<Bool>,
    }
}

diesel::table! {
    transactions (id) {
        id -> Uuid,
        user_id -> Int4,
        amount -> Numeric,
        description -> Text,
        created_at -> Nullable<Timestamptz>,
        #[max_length = 50]
        transaction_type -> Varchar,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 50]
        username -> Nullable<Varchar>,
        password -> Text,
    }
}

diesel::joinable!(balances -> users (user_id));
diesel::joinable!(otps -> users (user_id));
diesel::joinable!(transactions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    balances,
    otps,
    transactions,
    users,
);
