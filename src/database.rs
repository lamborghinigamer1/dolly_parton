use mysql::prelude::*;
use mysql::*;
use std::env;

pub fn _createdb(
    database_name: &str,
    username: &str,
    password: &str,
    hostname: &str,
    port: u16,
) -> Result<(), mysql::Error> {
    let opts = Opts::from_url(&format!(
        "mysql://{}:{}@{}:{}/",
        username, password, hostname, port
    ))?;

    let pool = Pool::new(opts)?;

    let mut conn = pool.get_conn()?;

    conn.query_drop(&format!("CREATE DATABASE IF NOT EXISTS {}", database_name))?;

    conn.query_drop(&format!("USE {}", database_name))?;

    conn.query_drop(
        r#"
        CREATE TABLE IF NOT EXISTS social_credits (
            id int PRIMARY KEY AUTO_INCREMENT,
            user varchar(255) NOT NULL,
            credits int,
            job varchar(255),
            salary int
        )
    "#,
    )?;

    Ok(())
}

fn _establish_connection() -> Result<Pool, mysql::Error> {
    let database_name = "dolly_parton";
    let username = env::var("SQL_USERNAME").expect("Expected a SQL_USERNAME in the environment");
    let password = env::var("SQL_PASSWORD").expect("Expected a SQL_PASSWORD in the environment");
    let hostname = env::var("HOSTNAME").expect("Expected a HOSTNAME in the environment");
    let port = 3306;

    let opts = Opts::from_url(&format!(
        "mysql://{}:{}@{}:{}/{}",
        username, password, hostname, port, database_name
    ))?;

    Ok(Pool::new(opts)?)
}

pub fn _putindb(user: &str, credits: u16) -> Result<(), mysql::Error> {
    let pool = _establish_connection()?;
    let mut conn = pool.get_conn()?;

    let stmt = conn.prep("INSERT INTO social_credits (user, credits) VALUES (?, ?)")?;
    conn.exec_drop(&stmt, (user, credits))?;

    Ok(())
}

pub fn _getuserinfo(user: &str) -> Result<Option<(String, i32)>, mysql::Error> {
    let pool = _establish_connection()?;
    let mut conn = pool.get_conn()?;

    let stmt = conn.prep("SELECT user, credits FROM social_credits WHERE user = ?")?;
    let mut rows = conn.exec_iter(&stmt, (user,))?;

    if let Some(row) = rows.next() {
        let (username, credits) = from_row::<(String, i32)>(row?);
        Ok(Some((username, credits)))
    } else {
        Ok(None)
    }
}

pub fn _add_credits(user: &str, new_credits: i32) -> Result<Option<(String, i32)>, mysql::Error> {
    let pool = _establish_connection()?;
    let mut conn = pool.get_conn()?;

    // Step 1: Get the current credits for the user
    let select_stmt = conn.prep("SELECT user, credits FROM social_credits WHERE user = ?")?;
    let mut select_rows = conn.exec_iter(&select_stmt, (user,))?;

    if let Some(row) = select_rows.next() {
        let (username, current_credits) = from_row::<(String, i32)>(row?);

        // Release the mutable borrow on `conn` by dropping select_rows
        drop(select_rows);

        // Step 2: Calculate the new total credits
        let total_credits = current_credits + new_credits;

        // Step 3: Update the database with the new total credits
        let update_stmt = conn.prep("UPDATE social_credits SET credits = ? WHERE user = ?")?;
        conn.exec_drop(&update_stmt, (total_credits, user))?;

        Ok(Some((username, total_credits)))
    } else {
        // User not found, return None
        Ok(None)
    }
}
