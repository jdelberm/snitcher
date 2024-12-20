use sqlite::{Connection, State};
use std::{error::Error, path::Path};
use teloxide::types::ChatId;

const CREATE_TABLES: &str = "
    CREATE TABLE suscription (chat_id TEXT PRIMARY KEY);
    CREATE TABLE known_items (name TEXT, price TEXT, href TEXT PRIMARY KEY);
";

const INSERT_SUSCRIPTION: &str = "INSERT INTO suscription (chat_id) VALUES (?);";
const INSERT_KNOWN_ITEMS: &str = "INSERT INTO known_items (name, price, href) VALUES (?,?,?);";

const QUERY_SUSCRIPTIONS: &str = "SELECT * FROM suscription";
const QUERY_KNOWN_ITEMS: &str = "SELECT * FROM known_items;";

pub fn init_ddbb() -> Connection {
    let already_exists = Path::new("./data.db").exists();
    let connection = sqlite::open("data.db").unwrap();

    if already_exists {
        log::info!("Initializating database");
    } else {
        log::info!("Creating database for the first time");
        connection.execute(CREATE_TABLES).unwrap();
    }

    return connection;
}

pub async fn store_known_items(
    connection: sqlite::Connection,
    items: Vec<(String, String, String)>,
) -> Result<(), Box<dyn Error>> {
    log::info!("Adding known items");
    let mut statement = connection.prepare(INSERT_KNOWN_ITEMS)?;

    items.into_iter().for_each(|item| {
        let _ = statement.bind(1, item.0.as_str());
        let _ = statement.bind(2, item.1.as_str());
        let _ = statement.bind(3, item.2.as_str());
        match statement.next() {
            Ok(_k) => {}
            Err(e) => log::info!("{}", e),
        }
        let _ = statement.reset();
    });

    Ok(())
}

pub async fn get_known_items(
    connection: sqlite::Connection,
) -> Result<Vec<(String, String, String)>, Box<dyn Error>> {
    let mut items: Vec<(String, String, String)> = Vec::new();

    let mut statement = connection.prepare(QUERY_KNOWN_ITEMS)?;
    while let State::Row = statement.next()? {
        items.push((statement.read(0)?, statement.read(1)?, statement.read(2)?))
    }

    Ok(items)
}

pub async fn get_subscriptions(
    connection: sqlite::Connection,
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut subscriptions: Vec<String> = Vec::new();

    let mut statement = connection.prepare(QUERY_SUSCRIPTIONS)?;
    while let State::Row = statement.next()? {
        subscriptions.push(statement.read(0)?);
    }

    Ok(subscriptions)
}

pub async fn add_suscription(
    connection: sqlite::Connection,
    chat_id: ChatId,
) -> Result<(), Box<dyn Error>> {
    log::info!("Adding subscription");
    let mut statement = connection.prepare(INSERT_SUSCRIPTION)?;

    statement.bind(1, chat_id.to_string().as_str())?;
    let _ = statement.next();
    statement.reset()?;
    Ok(())
}
