// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "accountstatus"))]
    pub struct Accountstatus;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "accounttype"))]
    pub struct Accounttype;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "transactiontype"))]
    pub struct Transactiontype;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Accounttype;
    use super::sql_types::Accountstatus;

    accounts (id) {
        id -> Int8,
        account_type -> Accounttype,
        account_status -> Accountstatus,
        opened_on_date -> Date,
        transaction_count -> Int8,
        balance -> Numeric,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Transactiontype;

    transactions (id) {
        id -> Int8,
        category -> Transactiontype,
        account_id -> Int8,
        receiver_id -> Int8,
        date_time -> Timestamp,
        amount -> Numeric,
        success -> Bool,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    accounts,
    transactions,
);
