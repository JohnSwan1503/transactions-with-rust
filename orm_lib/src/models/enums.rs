
use crate::schema::sql_types;
use diesel::deserialize::{self, FromSql, FromSqlRow};
use diesel::expression::AsExpression;
use diesel::pg::{Pg, PgValue};
use diesel::serialize::{self, IsNull, Output, ToSql};
use std::io::Write;


#[derive(AsExpression, Clone, Copy, Debug, FromSqlRow, PartialEq)]
#[diesel(sql_type=sql_types::Accountstatus)]
#[diesel(table_name = accounts)]
#[diesel(check_for_backend(Pg))]
pub enum AccountStatus {
    Open,
    Closed,
    Frozen
}

#[derive(AsExpression, Clone, Copy, Debug, FromSqlRow, PartialEq)]
#[diesel(sql_type = sql_types::Accounttype)]
pub enum AccountType {
    Personal,
    Business,
    DataEngineer
}

#[derive(Debug, AsExpression, FromSqlRow, PartialEq)]
#[diesel(sql_type = sql_types::Transactiontype)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Transfer
}

// ToSQL implementations for enums
impl ToSql<sql_types::Accountstatus, Pg> for AccountStatus {
    fn to_sql<'b>(&'b self, out: &mut Output<'b,'_ , Pg>) -> serialize::Result {
        match *self {
            AccountStatus::Open => out.write_all(b"open")?,
            AccountStatus::Closed => out.write_all(b"closed")?,
            AccountStatus::Frozen => out.write_all(b"frozen")?,
        }
        Ok(IsNull::No)
    }
}

impl ToSql<sql_types::Accounttype, Pg> for AccountType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b,'_ , Pg>) -> serialize::Result {
        match *self {
            AccountType::Personal => out.write_all(b"personal")?,
            AccountType::Business => out.write_all(b"business")?,
            AccountType::DataEngineer => out.write_all(b"data_engineer")?,
        }
        Ok(IsNull::No)
    }
}

impl ToSql<sql_types::Transactiontype, Pg> for TransactionType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b,'_ , Pg>) -> serialize::Result {
        match *self {
            TransactionType::Deposit => out.write_all(b"deposit")?,
            TransactionType::Withdrawal => out.write_all(b"withdrawal")?,
            TransactionType::Transfer => out.write_all(b"transfer")?,
        }
        Ok(IsNull::No)
    }
}

// FromSQL implementations for enums
impl FromSql<sql_types::Accountstatus, Pg> for AccountStatus {
    fn from_sql<'b>(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"open" => Ok(AccountStatus::Open),
            b"closed" => Ok(AccountStatus::Closed),
            b"frozen" => Ok(AccountStatus::Frozen),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl FromSql<sql_types::Accounttype, Pg> for AccountType {
    fn from_sql<'b>(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"personal" => Ok(AccountType::Personal),
            b"business" => Ok(AccountType::Business),
            b"data_engineer" => Ok(AccountType::DataEngineer),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl FromSql<sql_types::Transactiontype, Pg> for TransactionType {
    fn from_sql<'b>(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"deposit" => Ok(TransactionType::Deposit),
            b"withdrawal" => Ok(TransactionType::Withdrawal),
            b"transfer" => Ok(TransactionType::Transfer),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}
