pub mod schema;
pub mod models;

use bigdecimal::BigDecimal;
use chrono::{ NaiveDate
            , NaiveDateTime };
use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use models::{ enums::{ AccountStatus
                     , AccountType
                     , TransactionType }
            , new_records::{ NewAccount
                           , NewTransaction }
            , records::Account };
use std::env;


pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in ./orm_lib/.env");

    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}


pub fn insert_account( conn: &mut PgConnection
                     , type_of_account: &AccountType
                     , current_date: &NaiveDate) -> Account {
    use self::schema::accounts::dsl::*;

    let new_account = NewAccount{
            account_type: type_of_account
            , opened_on_date: current_date
    };

    diesel::insert_into(accounts)
        .values(&new_account)
        .returning(Account::as_returning())
        .get_result(conn)
        .expect("Error saving new account")
}

pub fn deposit_transaction( conn: &mut PgConnection
                          , account: &Account
                          , amount: &BigDecimal ) -> diesel::result::QueryResult<()> {
    use self::schema::accounts::dsl::*;
    
    conn.transaction::<_, diesel::result::Error, _>(|conn| {

        match account.account_status {
            AccountStatus::Closed => Err(diesel::result::Error::RollbackTransaction),
            _ => { 
                diesel::update(&account)
                    .set((
                            balance.eq(balance + amount)
                        ,   transaction_count.eq(transaction_count + 1)
                    ))
                    .execute(conn)?;
                
                if account.balance >= BigDecimal::from(0) {
                    diesel::update(&account)
                        .set(account_status.eq(AccountStatus::Open))
                        .execute(conn)?;
                }

                diesel::result::QueryResult::Ok(())
            },
        }
    })
}

pub fn withdrawal_transaction( conn: &mut PgConnection
                             , account: &Account
                             , amount: &BigDecimal) -> diesel::result::QueryResult<()> {
    use self::schema::accounts::dsl::*;
    
    conn.transaction::<_, diesel::result::Error, _>(|conn| {

        match account.account_status {
            AccountStatus::Open => {
                diesel::update(&account)
                    .set((
                            balance.eq(balance - amount)
                        ,   transaction_count.eq(transaction_count + 1)
                        ))
                    .execute(conn)?;
                
                if account.balance < BigDecimal::from(0) {
                    diesel::update(&account)
                        .set(account_status.eq(AccountStatus::Frozen))
                        .execute(conn)?;
                }

                diesel::result::QueryResult::Ok(()) 
            },  _ => Err(diesel::result::Error::RollbackTransaction)
        }
    })
}

pub fn deposit_or_withdrawal_transaction( conn: &mut PgConnection
                                        , account: &Account
                                        , amount: &BigDecimal) -> (diesel::result::QueryResult<()>, TransactionType) {
    match amount > &BigDecimal::from(0) {
        true => (   deposit_transaction(conn, account, &amount.abs())
                ,   TransactionType::Deposit ),
        false => (  withdrawal_transaction(conn, account, &amount.abs())
                 ,  TransactionType::Withdrawal ),
    }
}

pub fn transfer_transaction( conn: &mut PgConnection
                           , sender_account: &Account
                           , receiver_account: &Account
                           , amount: &BigDecimal) -> (diesel::result::QueryResult<()>, TransactionType) {

    (   conn.transaction::<_, diesel::result::Error, _>(|conn| {
            
            deposit_transaction(conn, receiver_account, amount)?;
            withdrawal_transaction(conn, sender_account, amount)?;

            diesel::result::QueryResult::Ok(())
        })
    ,   TransactionType::Transfer 
    )
}

pub fn insert_transaction( conn: &mut PgConnection
                         , send_id: &i64
                         , rece_id: &i64
                         , tran_date_time: &NaiveDateTime
                         , tran_amount: &BigDecimal
                         , tran_type: &TransactionType
                         , tran_success: &bool ) -> diesel::result::QueryResult<()> {
    use self::schema::transactions::dsl::*;

    diesel::insert_into(transactions).values(
        &NewTransaction { 
              category: tran_type
            , account_id: send_id
            , receiver_id: rece_id
            , date_time: tran_date_time
            , amount: tran_amount
            , success: tran_success
        })
        .execute(conn)?;
    
    diesel::result::QueryResult::Ok(())
}

pub fn perform_transaction( conn: &mut PgConnection
                          , sender_id: &i64
                          , receiver_id: &Option<i64>
                          , transaction_amount: &BigDecimal
                          , transaction_date_time: &NaiveDateTime ) -> diesel::result::QueryResult<()> {
    use self::schema::accounts::dsl::*;
    

    // Get the sender account, or panic if it doesn't exist
    let sender_account: Account = match accounts.filter(id.eq(sender_id))
            .select(Account::as_select())
            .first::<Account>(conn)
            .optional()
            .expect("Error loading sender account") {
        Some(account) => account,
        None => panic!("Sender account not found"),
    };
    
    // Get the receiver account, or use the sender account if it doesn't exist or receiver_id is None
    let receiver_account = match receiver_id {
        Some(receiver_id) => match
            accounts.filter(id.eq(receiver_id))
                .select(Account::as_select())
                .first::<Account>(conn)
                .optional()
                .expect("Error loading receiver account") {
            Some(account) => account,
            None => sender_account.clone(),
                },
        None => sender_account.clone(),
    };

    conn.transaction::<_, diesel::result::Error, _>(|conn| {
        let (transaction, type_of_transaction) = match receiver_account.id == sender_account.id {
                true  => deposit_or_withdrawal_transaction(conn, &sender_account, transaction_amount),
                false => transfer_transaction(conn, &sender_account, &receiver_account, &transaction_amount.abs()),
            };

        insert_transaction( conn
                          , &sender_account.id.to_owned()
                          , &match receiver_id {
                              Some(receiver_id) => receiver_id.to_owned(),
                              None => sender_account.id, }
                          , transaction_date_time
                          , transaction_amount
                          , &type_of_transaction
                          , &match transaction {
                                Ok(_) => true,
                                Err(_) => false,
                            } )?;
        
        diesel::result::QueryResult::Ok(())

    })

}