use crate::{schema::{accounts,  transactions}, 
            models::enums::{AccountType, TransactionType}};
use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use diesel::prelude::*;

#[derive(Insertable)]
#[diesel(table_name = accounts)]
pub struct NewAccount<'a> {
    pub account_type: &'a AccountType,
    pub opened_on_date: &'a NaiveDate
}

#[derive(AsChangeset)]
#[diesel(table_name = accounts)]
pub struct UpdateAccount<'a> {
    pub transaction_count: &'a i64,
    pub balance: &'a BigDecimal,
}


#[derive(Insertable)]
#[diesel(table_name = transactions)]
pub struct NewTransaction<'a> {
    pub category: &'a TransactionType,
    pub account_id: &'a i64,
    pub receiver_id: &'a i64,
    pub date_time: &'a chrono::NaiveDateTime,
    pub amount: &'a BigDecimal,
    pub success: &'a bool
}

