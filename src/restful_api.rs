use crate::database::Database;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;

/// Starts the web server and binds it to the specified address and port.
///
/// This function initializes the HTTP server and sets up the route for handling
/// transaction queries. It binds the server to the address `127.0.0.1` and port `8080`.
///
/// # Returns
///
/// A `std::io::Result<()>` indicating the success or failure of starting the server.
#[actix_web::main]
pub async fn web_server() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(transactions))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

/// Represents query parameters for filtering transactions.
#[derive(Deserialize)]
struct Info {
    start_date: Option<String>,
    end_date: Option<String>,
    signature: Option<String>,
    sender: Option<String>,
    receiver: Option<String>,
}

/// Handles HTTP GET requests to retrieve filtered transactions.
///
/// This function queries the database for transactions that match the specified
/// query parameters. The supported query parameters are `start_date`, `end_date`,
/// `signature`, `sender`, and `receiver`.
///
/// # Arguments
///
/// * `info` - The query parameters for filtering the transactions.
///
/// # Returns
///
/// A JSON response containing the filtered transactions.
#[get("/transactions")]
async fn transactions(info: web::Query<Info>) -> impl Responder {
    let mut database = Database::new_connection().unwrap();
    let mut query = "SELECT * FROM transactions".to_string();
    let mut flag = false;
    if let Some(start_date) = &info.start_date {
        start_date_query(&mut flag, &mut query, start_date)
    }
    if let Some(end_date) = &info.end_date {
        end_date_query(&mut flag, &mut query, end_date)
    }
    if let Some(signature) = &info.signature {
        signature_query(&mut flag, &mut query, signature)
    }
    if let Some(sender) = &info.sender {
        sender_query(&mut flag, &mut query, sender)
    }
    if let Some(recevier) = &info.receiver {
        receiver_query(&mut flag, &mut query, recevier)
    }
    let data = database.query(&query);
    HttpResponse::Ok().json(data)
}

/// Adds a sender filter to the query string.
///
/// # Arguments
///
/// * `flag` - A mutable reference to a boolean flag indicating whether this is the first filter.
/// * `query` - A mutable reference to the query string.
/// * `sender` - The sender to filter by.
fn sender_query(flag: &mut bool, query: &mut String, sender: &str) {
    if !(*flag) {
        query.push_str(" WHERE");
        *flag = true;
    } else {
        query.push_str(" AND");
        *flag = true;
    }
    query.push_str(" sender=\"");
    query.push_str(sender);
    query.push('"');
}

/// Adds a receiver filter to the query string.
///
/// # Arguments
///
/// * `flag` - A mutable reference to a boolean flag indicating whether this is the first filter.
/// * `query` - A mutable reference to the query string.
/// * `receiver` - The receiver to filter by.
fn receiver_query(flag: &mut bool, query: &mut String, receiver: &str) {
    if !(*flag) {
        query.push_str(" WHERE");
        *flag = true;
    } else {
        query.push_str(" AND");
        *flag = true;
    }
    query.push_str(" receiver=\"");
    query.push_str(receiver);
    query.push('"');
}

/// Adds a signature filter to the query string.
///
/// # Arguments
///
/// * `flag` - A mutable reference to a boolean flag indicating whether this is the first filter.
/// * `query` - A mutable reference to the query string.
/// * `signature` - The signature to filter by.
fn signature_query(flag: &mut bool, query: &mut String, signature: &str) {
    if !(*flag) {
        query.push_str(" WHERE");
        *flag = true;
    } else {
        query.push_str(" AND");
        *flag = true;
    }
    query.push_str(" signature=\"");
    query.push_str(signature);
    query.push('"');
}

/// Adds a start date filter to the query string.
///
/// # Arguments
///
/// * `flag` - A mutable reference to a boolean flag indicating whether this is the first filter.
/// * `query` - A mutable reference to the query string.
/// * `start_date` - The start date to filter by.
fn start_date_query(flag: &mut bool, query: &mut String, start_date: &str) {
    if !(*flag) {
        query.push_str(" WHERE");
        *flag = true;
    } else {
        query.push_str(" AND");
        *flag = true;
    }
    query.push_str(" timestamp>=");
    query.push_str(start_date);
}

/// Adds an end date filter to the query string.
///
/// # Arguments
///
/// * `flag` - A mutable reference to a boolean flag indicating whether this is the first filter.
/// * `query` - A mutable reference to the query string.
/// * `end_date` - The end date to filter by.
fn end_date_query(flag: &mut bool, query: &mut String, end_date: &str) {
    if !(*flag) {
        query.push_str(" WHERE");
        *flag = true;
    } else {
        query.push_str(" AND");
        *flag = true;
    }
    query.push_str(" timestamp<=");
    query.push_str(end_date);
}
