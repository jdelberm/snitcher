
use teloxide::{payloads::SendMessageSetters, requests::Requester, types::{ChatId, Recipient}, Bot};
use tokio::{task, time};
use visdom::Vis;
use std::{error::Error, time::Duration};
use crate::{sql::{self, get_known_items}, utils};

mod keys;

const SOURCES: [(&str, fn(body: String) -> Result<Vec<(String, String, String)>, Box<dyn Error + Send + Sync>>); 1] = [
    (
        keys::TARGET_URL,
        |body: String| {
            //Get all items from body and append as many as there are to the vector
            let root_element = Vis::load(body)?;
            let elements = root_element.find(".pro_second_box");
            let mut items_result = Vec::new();
            // Multiple results treatment
            for element in elements {
                let children = element.children();
                
                let name_option = children.find(".s_title_block").find("a").attr("title");            
                let name_value;
                if let Some(name) = name_option {
                    name_value = name.to_string();
                } else {
                    name_value = "".to_string();
                }
                
                let href_option = children.find(".s_title_block").find("a").attr("href");             
                let href_value;
                if let Some(href) = href_option {
                    href_value = href.to_string();
                } else {
                    href_value = "".to_string();
                }     

                let price = children.find(".price").text().replace("\u{a0}", "").replace("€", "");
                log::info!("Found one -> {} :{} ({})", name_value, price, href_value);
    
                items_result.push((name_value, price, href_value));
            }

            Ok(items_result)
        }
    )
];

pub async fn get_items() -> Result<Vec<(String, String, String)>, Box<dyn Error>> {

    let mut all_items=  Vec::new();
    for source in SOURCES {
        let (url, callback) = source;
        log::info!("Iterating source from {}", url);

        let response = reqwest::get(url).await?;
        let body = response.text().await?;

        match callback(body){
            Ok(items) => {
                let mut mutitems = items;
                all_items.append(&mut mutitems)
            },
            Err(err) => log::info!("Error: {}", err),
        }
    }

    if all_items.len() > 0 {
        //Sort by price
        all_items.sort_by(|a,b| a.1.cmp(&b.1));
        Ok(all_items)
    }else {
        Err("No results found".into())
    }

}

async fn notify_subscribers(new_items:Vec<(String, String, String)>){
    let connection = sql::init_ddbb();
    let bot = Bot::from_env();

    let mut msg = String::new();
    msg.push_str("New items found!\n");
    for item in new_items{
        msg.push_str(utils::from_tuple_to_html(item).as_str())
    }
    log::info!("msg -> {}", msg);
    match sql::get_subscriptions(connection).await {
        Ok(subs) => {
           

                let _watcher = task::spawn(async move{ 
                    for chat_id in subs{
                    log::info!("Sending new items to chat_id {}",chat_id);
                    let _ = bot.send_message(
                        Recipient::Id(ChatId(chat_id.as_str().parse::<i64>().unwrap())), 
                        msg.as_str()).parse_mode(teloxide::types::ParseMode::Html).await;
                    };
                });

                
        }
        Err(_e) => log::info!("There isn't subscribers where to send new items")
    }
}

async fn find_changes() {
    log::info!("Looking for changes");
    let mut current_items = Vec::new();
    if let Ok(vals) = get_items().await{
        let mut copy= vals.to_vec();
        current_items.append(&mut copy);
    } 
    current_items.clone().into_iter().for_each(|v|{log::info!("Name: {}, Price: {}",v.0, v.1)});

    let mut saved_items = Vec::new();
    if let Ok(vals) = get_known_items(sql::init_ddbb()).await {
        log::info!("Items read from database");
        let mut copy= vals.to_vec();
        saved_items.append(&mut copy);
    }

    let iter = current_items.iter();
    let new_items: Vec<_> = iter.filter(|i|{ !saved_items.contains(i)}).cloned().collect();
    if new_items.len() > 0 {
        log::info!("New items found -> {}", new_items.len());
        new_items.clone().into_iter().for_each(|v|{log::info!("Name: {}, Price: {}",v.0, v.1)});
        notify_subscribers(new_items.clone()).await;
        
        match sql::store_known_items(sql::init_ddbb(), new_items.clone()).await {
            Ok(_) => {
                log::info!("Saved new items");
            }
            Err(e) => {
                log::error!("Failed write to ddbb -> {}", e);
            }
        };

    }else{
        log::info!("Nothing new");
    }
}

pub async fn start_scraping() {
    log::info!("Starting scrapper");
    let _watcher = task::spawn(async{
        let mut interval = time::interval(Duration::from_millis(1800000));
        
        loop {
            interval.tick().await;
            find_changes().await;
        }
    });

}

/*
pub fn stop_scraping(interval_id: u64) {
    clear_interval(interval_id);
}*/
