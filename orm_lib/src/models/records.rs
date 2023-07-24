use crate::{schema::{accounts,  transactions}, 
            models::enums::{AccountStatus, AccountType, TransactionType}};
use bigdecimal::BigDecimal;
use diesel::pg::Pg;
use diesel::prelude::*;
use chrono::{NaiveDate, NaiveDateTime};


#[derive(AsChangeset, Clone, Debug, Identifiable, Insertable, PartialEq, Queryable, Selectable)]
#[diesel(table_name = accounts)]
#[diesel(check_for_backend(Pg))]
pub struct Account {
    pub id: i64,
    pub account_status: AccountStatus,
    pub account_type: AccountType,
    pub opened_on_date: NaiveDate,
    pub transaction_count: i64,
    pub balance: BigDecimal
}

#[derive(AsChangeset, Queryable, Selectable)]
#[diesel(table_name = transactions)]
#[diesel(check_for_backend(Pg))]
pub struct Transaction {
    pub id: i64,
    pub category: TransactionType,
    pub account_id: i64,
    pub receiver_id: i64,
    pub date_time: NaiveDateTime,
    pub amount: BigDecimal,
    pub success: bool
}