# Transactions With Rust

Expanding on my previous exercise. Hooking a Rust ORM to postgres and implementing transactions in Rust.

- [Transactions With Rust](#transactions-with-rust)
  - [Motivation](#motivation)
  - [Process Documentation](#process-documentation)
    - [Part One: Setting things up](#part-one-setting-things-up)
      - [Dependencies](#dependencies)
      - [Project Initialization Boilerplate](#project-initialization-boilerplate)
      - [Defining the schema](#defining-the-schema)
    - [Part Two: Database Transactions](#part-two-database-transactions)
      - [Deposit](#deposit)
      - [Withdrawal](#withdrawal)
      - [Transfer](#transfer)
      - [Putting it all together](#putting-it-all-together)
  - [Conclusion](#conclusion)

## Motivation

A while back I read a comment in a StackExchange post that lamented the lack of intermeidate+ level guides on how to use the rust Diesel library. The goal of this is to go from 0 to multi-level transactions in a concise if thorough manner. The amount of work required is minimal (this took substantially longer to write about and format than the hour or so to actually develop), but it will serve as a guide for myself and anybody curious enough to read through. Rust is a powerful language with features that position it very well for data engineering tasks. All of the files and code are here in the repo.

## Process Documentation

### Part One: Setting things up

Most of this is all pretty decently documented over on the [diesel.rs](http://diesel.rs/guides/) guides page and [diesel/examples](https://github.com/diesel-rs/diesel/tree/2.1.x/examples) directory in the official repo. Skip to [Defining the schema](#defining-the-schema) to bypass the boilerplate.

#### Dependencies

1. **PostgreSQL** - Pull the latest postgres image from docker and startup a new db to use for this exercise. Start the service.

    ```yaml
    version: "3.8"

    services:
      db:
        environment:
          POSTGRES_PASSWORD: postgres
          POSTGRES_USER: postgres
          POSTGRES_DB: diesel
        image: postgres
        restart: always
        expose:
          - 5432
        ports:
          - 5432:5432
    ```

2. **libpq-dev**: Enter `apt -qq list libpq-dev` into the terminal to check if it has been installed. If it has, the terminal should echo:

    ```bash
    libpq-dev/jammy-updates,jammy-security,now 14.8-0ubuntu0.22.04.1 amd64 [installed]
    ```

    If it is not installed, then do so by entering: `sudo apt-get update && sudo apt-get install libpq-dev`. This package will be the connector between diesel backend and the postgres database.

3. **Diesel CLI**: Next install the Diesel CLI with the cargo package manager by entering: `cargo install diesel_cli --no-default-features --features postgres`.[^1]

#### Project Initialization Boilerplate

1. **Create a library**: Create a new rust library by entering `cargo new --lib orm_lib && cd orm_lib` into the terminal.
2. **Update Dependencies**: Update the *cargo.toml* file to include the **diesel** and **dotenv** crates in the list of dependencies.

    ```toml
    [package]
    name = "orm_lib"
    version = "0.1.0"
    edition = "2021"

    [dependencies]
    bigdecimal = "0.3"
    chrono = "0.4.26"
    diesel = { version = "2.1.0", features = ["postgres", "chrono", "numeric", 
    "extras"] }
    dotenv = "0.15.0"
    ```

3. **Environment Variables**: Create a *.env* file in the rust library directory and include a variable for the `DATABASE_URL`. Very important as it will direct the diesel to the postgres database.
    The variable should be formatted as:

    ```bash
    DATABASE_URL = postgres://$postgres_username[:$postgres_password]@$hostname[:$port]/$postgres_db
    ```

    So, based on the `docker-compose.yml` above:

    ```bash
    DATABASE_URL = postgres://postgres:postgres@localhost:5432/diesel
    ```

4. **Connect Diesel to Postgres**: Enter `diesel setup` into the terminal. A migrations folder has been created. This is where schema migrations are defined and updated as the project evolves.

#### Defining the schema

Diesel CLI provides commands to update and rollback schema changes. It tracks these changes in the migrations/ directory. Instead of defining tables, data types, functions etc. by entering sql directly through a database connection, new schema definitions are created with the `migration generate` cli command. Then pass the SQL schema definitions allong to the Diesel CLI code generator by passing `diesel migration run` to update the *schema.rs* file. We will use this pattern for the tables (I'll only walk through the accounts table in full for the sake of brevity):  

- **accounts**: Enter `diesel migration generate initial_tables` into the terminal. The table schema will be similar to that of the previous exercise, but with a few additional details included to the table definition:
  - Table schema and field descriptions:
    - **id** *(bigint)*: The account id
    - **account_status** *(AccountStatus)*: Tracks the current status of the account. Determines what kinds of activities are currently allowed for the account.
    - **acount_type** *(AccountType)*: Records the type of account. Determines what rules and penalties are applicable.
    - **opened_on_date** *(date)*: Records the date the account was opened.
    - **transaction_count** *(bigint)*: Tracks the monthly transaction count. Updated every time the account participates in a successful transaction. If the value exceeds the number permitted, the account will be limited as if it had a negative balance. The transaction count resets every month.
    - **balance** *(numeric)*: Tracks the overall balance of the account. Although the balance may be negative, if it is less than zero at the onset of a debit or outgoing transfer transaction, the transaction will get rolled back. If the balance ends below zero at the end of the transaction, a penalty is applied and the account_status is updated to `frozen`.
  - PostgreSQL Table and Type definitions
  
    *./migrations/...create_accounts/up.sql*

    ```sql
    CREATE TYPE AccountType AS ENUM(
        'personal',
        'business',
        'data_engineer'
    );

    CREATE TYPE AccountStatus AS ENUM(
        'open',
        'frozen',
        'closed'
    )

    CREATE TABLE accounts (
        id                bigint          NOT NULL,
        account_type      AccountType     NOT NULL,
        account_status    AccountStatus   NOT NULL,
        opened_on_date    date            NOT NULL,
        transaction_count bigint          NOT NULL    DEFAULT 0::integer,
        balance           numeric         NOT NULL    DEFAULT 0::numeric,
        CONSTRAINT        account_id_pk   PRIMARY KEY ( id ),
        CONSTRAINT        opened_on_check CHECK       ( opened_on_date BETWEEN '2019-12-31'::date AND 'infinity'::date )
    );
    ```

    *./migrations/...create_accounts/down.sql*

    ```sql
    DROP TABLE accounts;
    DROP TYPE  AccountType;
    ```

  - Rust types as deserialized representations of the custom postgres enum types. The macros prepended to the enum definitions provide trait implementations for the data types' serde, representation, and mappings to types in postgres.

    ```rust
    #[derive(Debug, AsExpression, FromSqlRow)]
    #[diesel(sql_type = sql_types::Accountstatus)]
    pub enum AccountStatus {
        Open,
        Closed,
        Frozen
    }

    #[derive(Debug, AsExpression, FromSqlRow)]
    #[diesel(sql_type = sql_types::Accounttype)]
    pub enum AccountType {
        Personal,
        Business,
        DataEngineer
    }
    ```

  - Diesel provides macros to generate some of the serde implementations for our new data types. However for enums additional implementations `ToSql` and `FromSql` need more explicit definition. Below are those two implementations for the *AccountStatus* Rust enum type. *AccountType* and *TransactionType* will have a similar implementation.

    ```rust
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
    ```

  - By treating an entire row as a struct, records and their fields become exposed to Diesel's query writer. The Identifiable trait allows rows to be treated and updated on their own and assumes a 1:1 correspondence between some instance of the struct and the corresponding data in the target table. Likewise, create rust struct representations of accounts records that we may want to insert or update. For these, we can leave out the fields with default or calculated values, as well as the fields we want to make sure do not get updated i.e. the field's primary key(s). As references to the values being entered into or updated the table are borrowed, we must define the variables' scopes in the struct.

    ```rust
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

    ```

  - Wrapping an `INSERT INTO` query in a rust function can look like this:

    ```rust
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
    ```

- **transactions**:
  - **id** *(bigint)*:
  - **category** *(TransactionType)*: The type of transaction is determined by the transaction itself. If the account_id is the same as the receiver_id, then the transaction is determined by the amount (positive for a deposit/credit, negative for a withdrawal/debit). When the acount_id is different from the receiver_id, then the transaction is considered a transfer. Note: transfers may only go in one direction towards the receiver acount.
  - **date_time** *(timestamp)*: The date and time of the transaction.
  - **amount** *(numeric)*: A non-zero amount of money to be transfered, debited, or credited.
  - **success** *(boolean)*: A number of scenarios will cause a transaction to fail. In those cases, the attempted transaction will still be saved to the transactions table, however this field flags which succeeeded and which failed.

    ```sql
    CREATE TYPE TransactionType AS ENUM(
        'debit',
        'credit',
        'transfer'
    );

    CREATE TABLE transactions (
        id           bigint            NOT NULL,
        category     TransactionType   NOT NULL,
        account_id   bigint            NOT NULL,
        receiver_id  bigint            NOT NULL,
        date_time    timestamp         NOT NULL,
        amount       numeric           NOT NULL,
        success      boolean           NOT NULL    DEFAULT false,
        CONSTRAINT   transaction_id_pk PRIMARY KEY ( id, category, account_id ),
        CONSTRAINT   account_id_fk     FOREIGN KEY ( account_id )  REFERENCES accounts ( id ),
        CONSTRAINT   receiver_id_fk    FOREIGN KEY ( receiver_id ) REFERENCES accounts ( id ),
        CONSTRAINT   amount_check      CHECK       ( amount <> 0::numeric )
    );
    ```

    And the rust implementation of the `INSERT INTO` sql function:

    ```rust
    pub fn insert_transaction( conn: &mut PgConnection
                             , send_id: &i64
                             , rece_id: &i64
                             , tran_date_time: &NaiveDateTime
                             , tran_amount: &BigDecimal
                             , tran_type: &TransactionType
                             , tran_success: &bool ) -> diesel::result::QueryResult<()> {
        use self::schema::transactions::dsl::*;

        diesel::insert_into(transactions).values(
            &NewTransaction { category: tran_type
                            , account_id: send_id
                            , receiver_id: rece_id
                            , date_time: tran_date_time
                            , amount: tran_amount
                            , success: tran_success
            })
            .execute(conn)?;
        
        diesel::result::QueryResult::Ok(())
    }
    ```

### Part Two: Database Transactions

#### Deposit

It is now possible to begin wrapping SQL transactions inside rust functions. First the deposit transaction. This transaction will do two things: add the deposit amount to the target account and then updating the account's status based on the account balance. A match statement is employed to perform a state check on the AccountStatus field of the account, only performing the operations if the account is not `closed`. [^2]

```rust
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
```

#### Withdrawal

The withdrawal transaction is similar to the deposit, only more strict: an account may only withdraw if it is `Open`. A similar additional step is taken to set the AccountStatus flag based on the ending balance.

```rust
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

            },  _ => Err(diesel::result::Error::RollbackTransaction)
        }
    })
}
```

These two transactions can be combined and orchestrated by a third funtion to reduce repetitive code. This function will determine and return the type of transaction as well as executing the steps defined for either type.

```rust
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
```

#### Transfer

A transfer is defined by two accounts and an ammount of money. All of the withdrawal rules apply to the account initiating the transfer while all of the deposit rules apply to the recipient account. Thus those functions may be reused here again. Reminder: the RollBackTransaction returned from either sub-transaction will roll back any other previous transactions or subtransactions within the scope of the connection.transaction() call.

```rust
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
```

#### Putting it all together

Now the various transaction types have been defined, define a final `perform_transaction` function to wrap around all of the transaction logic required to properly update the database based on some arbitrary input describing some transaction attempting to be performed.

```rust
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
```

## Conclusion

This exercise is intended to show how Rust is particularly adept at handling control flow and transaction state with the Result<T, E> return type. This allows for more complex, multi-staged transactions to be executed or rolled up with relative ease. While handling possibly missing values requires a bit of extra work, the total amount of code required for an ORM like the one here is trivial, and expanding or modifying the rules and transaction logic is made easy with the explicit scopes and error handling required by the language.

[^1]: This will only install the postgres backend features of the Diesel CLI tool and ignore those for SQLite and MySQL backends.
[^2]: The `Err(diesel::result::Error::RollbackTransaction)` error will trigger the ORM to roll back any other associated transactions not yet finalized. Note that the `diesel::result::QueryResult<()>` returned by a function can by used by a parent scope to determine rollback by default. If there are non-default behaviors desired like `READ ONLY` or `READ COMMITED` then diesel.build_transaction should be used with the appropriate methods appended.