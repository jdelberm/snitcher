use teloxide::{prelude::*, types::Recipient, utils::command::BotCommands};
mod scraper;
mod sql;
mod utils;

#[tokio::main]
async fn main() {
    env_logger::init();
    log::info!("Initializating...");

    scraper::start_scraping().await;

    let bot = Bot::from_env(); 
    let _ = bot.send_message(
        Recipient::Id(ChatId(657372341)), 
        "I am ready to snitch!").parse_mode(teloxide::types::ParseMode::Html).await;

    //Command proccessor
    Command::repl(bot, answer).await;
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    #[command(description = "Displays this text.")]
    Help,
    #[command(description = "Read items")]
    Tellme,
    #[command(description = "Suscribe to auto updates")]
    Suscribe,
}

async fn answer(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    let connection = sql::init_ddbb();

    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?
        }
        Command::Suscribe => {
            match sql::add_suscription(connection, msg.chat.id).await {
                Ok(_k) => {
                    log::info!("User {} suscribed successfully", msg.chat.id);
                }
                Err(e) => {
                    log::info!("error -> {}", e)
                }
            };
            bot.send_message(msg.chat.id, "Suscribed successfully")
                .await?
        }
        Command::Tellme => {
            let mut result = String::new();
            let mut items = Vec::new();

            match scraper::get_items().await {
                Ok(vals) => {
                    let mut copy= vals.to_vec();
                    items.append(&mut copy);
                }
                Err(_e) => {
                    log::info!("No items were found")
                }
            };

            items.clone().into_iter().for_each(|item| {
                result.push_str(utils::from_tuple_to_html(item).as_str());
            });

            bot.send_message(msg.chat.id, result)
                .parse_mode(teloxide::types::ParseMode::Html)
                .await?
        }
    };

    Ok(())
}
