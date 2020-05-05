#[macro_use] extern crate lazy_static;
use twilight::{
    gateway::{shard::Event, Cluster, ClusterConfig},
    http::Client as HttpClient
};
use futures::StreamExt;
use std::{env, error::Error};
use regex::Regex;
use bson::doc;
use mongodb::{
     Client as MongoClient,
     Database
};
use std::sync::{Arc, Mutex};

lazy_static! {
    static ref MESSAGE_REGEX: Regex = Regex::new(r"^f/ *(?:(?:([^\s=]+) +([^\s=]*) +(.*\S) *= *((?:.*\n*\r*)*))|(?:(here|\d{18}):)?(\S.*))?$").unwrap();
}

// fn get_tag_content(tag: &str, id: , db: Database) -> String {
//     
// }

async fn get_database(uri: &String) -> Result<Database, Box<dyn Error + Send + Sync>> {
    let client = MongoClient::with_uri_str(uri)?;
    Ok(client.database("tags"))
}

async fn handle_event(event: (u64, Event), client: HttpClient, db: Arc<Mutex<Database>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    match event {
        (id, Event::Ready(_)) => {
            println!("Connected on shard {}", id);
        }
        (_, Event::MessageCreate(message)) => {
            if message.author.bot == false {
                // TODO: Implement ability to blacklist users and add check for authors blacklist status here.
                
                if MESSAGE_REGEX.is_match(&message.content) {
                    // Check contents of 9th item
                    // If Some(tag) it is a normal command/tag triggering, else it is None and a command with inputs (f/tag create dab time = * dabs *) 
                    let captures = MESSAGE_REGEX.captures(&message.content).unwrap();
                    println!("{:?}", captures);
                    match captures.get(8).map(|m| m.as_str()) {
                        Some(x) => {let tag = x;
                                    let collection = db.lock().unwrap().collection(&message.guild_id.unwrap().to_string());
                                },
                        None    => {}
                    };

                    client.create_message(message.channel_id).content("dab").await?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}

async fn run_bot(token: &String, database_uri: &String) -> Result<(), Box<dyn Error + Send + Sync>> {
    let db = Arc::new(Mutex::new(get_database(database_uri).await?));

    let client = HttpClient::new(token);

    let cluster_config = ClusterConfig::builder(token).build();
    let cluster = Cluster::new(cluster_config);
    cluster.up().await?;

    let mut events = cluster.events().await;
    
    while let Some(event) = events.next().await {
        tokio::spawn(handle_event(event, client.clone(), Arc::clone(&db)));
    }
    Ok(())
}

fn argv_failure(argv0: &String) {
    println!("Usage: {} <token> <mongodb uri>", argv0);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Vector to contain argv from terminal, correct usage of the bot will yield
    // <"./executable", "token", "mongodb uri">
    let mut argv = Vec::new();

    for (position, argument) in env::args().enumerate() {
        match position {
            3 => {
                argv_failure(&argv[0]);
                break;
            }
            _ => {
                argv.push(argument);
            }
        }
    }
    if argv.len() != 3 {
        argv_failure(&argv[0]);
    } else {
        run_bot(&argv[1], &argv[2]).await?;
    }
    Ok(())
}