#[macro_use] extern crate lazy_static;
use twilight::{
    gateway::{
        shard::Event, 
        Cluster, 
        ClusterConfig
    },
    http::Client as HttpClient,
    model::channel::embed::*
};
use futures::StreamExt;
use std::{
    env, 
    error::Error
};
use regex::Regex;
use bson::doc;
use mongodb::{
     sync::{
         Client as MongoClient,
         Database
    },
     options::FindOneOptions
};
use chrono::Utc;

lazy_static! {
    // Regex for messages, matches messages that start with f/tag/, f/here/id:tag, f/command etc syntax
    static ref MESSAGE_REGEX: Regex = Regex::new(r"^f/ *(?:(?:(tags?|help|info|timezone|translate) +([^\s=]*) +(.*\S) *= *((?:.*\n*\r*)*))|(?:(here|\d{18}):)?(\S.*))$").unwrap();
    // Regex for creating and editing tags, captures image urls
    static ref IMAGE_URL_REGEX: Regex = Regex::new(r"^([^\2]*)(https?:\/\/(?:[a-z0-9\-]+\.)+[a-z]{2,6}(?:\/[^\/#?]+)+\.(?:(?:jp(?:g|eg)|webp|gif|png)|(?:JP(?:G|EG)|WEBP|GIF|PNG)))([^\2]*)$").unwrap();
    // Database that holds all the tags, tags are stored inside collections that are inside the tags database.
    // The collection name is the ID of the discord server that it is local to. Collection name will be 0 if it is a global tag.
    static ref TAGS_DATABASE: Database = {
        let client = MongoClient::with_uri_str(&env::var("FLIGHTLESS_MONGO_URI").unwrap()).unwrap();
        client.database("tags")
    };
    // Database that holds information on discord users.
    // Collections are admins and blacklisted.
    // TODO: Implement blacklist
    static ref USER_DATABASE: Database = {
        let client = MongoClient::with_uri_str(&env::var("FLIGHTLESS_MONGO_URI").unwrap()).unwrap();
        client.database("users")
    };
    // Discord user ID of bot owner is stored in the Mongo Database. Check README.md for more info.
    static ref OWNER_ID: u64 = {
        let collection = USER_DATABASE.collection("admins");
        let filter = doc! { "rank": "Owner" };
        let find_one_options = FindOneOptions::builder().build();
        let cursor = collection.find_one(filter, find_one_options);
        cursor.unwrap().unwrap().get_i64("id").unwrap() as u64
    };
}

async fn build_tag_embed(tag: &str, key: String) -> Embed {
    // Create the collection using the location as a key
    // Finding the tag in the locations collection
    let collection = TAGS_DATABASE.collection(&key);
    let filter = doc! { "name": tag };
    let find_one_options = FindOneOptions::builder().build();
    let cursor = collection.find_one(filter, find_one_options).unwrap().unwrap();

    // Build embed with Embed struct from Twilight
    Embed {
        author: Some(EmbedAuthor {
            icon_url: Some(String::from("https://raw.githubusercontent.com/weasel-dev/flightless/master/flightless.webp")),
            name: Some(String::from("Flightless")),
            proxy_icon_url: None,
            url: Some(String::from("https://flightless.duncy.nz/")),
        }),
        color: Some(0x007e6049),
        description: match cursor.get_str("content") {
            Ok(content) => {Some(content.to_string())},
            Err(_)      => {None}
        },
        fields: vec![],
        footer: None,
        image: Some(EmbedImage {
            height: None,
            proxy_url: None,
            url: match cursor.get_str("image") {
                Ok(image) => {Some(image.to_string())},
                Err(_)    => {None}
            },
            width: None,
        }),
        kind: String::from("rich"),
        provider: None,
        thumbnail: Some(EmbedThumbnail {
            height: None,
            proxy_url: None,
            url: None,
            width: None,
        }),
        // Formatted timestamp for Discord
        timestamp: Some(Utc::now().format("%F %T").to_string()),
        title: None,
        url: None,
        video: None,
    }
}

async fn handle_event(event: (u64, Event), client: HttpClient) -> Result<(), Box<dyn Error + Send + Sync>> {
    match event {
        (id, Event::Ready(_)) => {
            println!("Connected on shard {}", id);
        }
        (_, Event::MessageCreate(message)) => {
            if message.author.bot == false {
                // TODO: Implement ability to blacklist users and add check for authors blacklist status here.
                if MESSAGE_REGEX.is_match(&message.content) {
                    let captures = MESSAGE_REGEX.captures(&message.content).unwrap();
                    // Check contents of 7th item (6th capture group, index for captures starts at 1 as 0th item is whole string if there is a match)
                    // If Some(tag) it is a normal command/tag triggering, else it is None and a command with inputs (f/tag create dab time = * dabs *) 
                    match captures.get(6).map(|m| m.as_str()) {
                        Some(capture) => {
                            let tag = capture;
                            // 6th item (5th capture group) is the optional location part of the string
                            // If a user specifies a location like here (will be translated to the server's id) or none for global
                            // If a tag name doesn't exist in global then it will check local tags
                            // match Result(String)
                            match match captures.get(5).map(|m| m.as_str()) {
                                Some("here") => {
                                    // Specified that tag is local, return message guild id to be used as collection name
                                    Ok(message.guild_id.unwrap().0.to_string())
                                },
                                Some(capture) => {
                                    // Specified a specific server id, return it as a string
                                    // Check if user can access server with id
                                    if capture.parse::<u64>().unwrap() == message.guild_id.unwrap().0 {
                                        Ok(message.guild_id.unwrap().0.to_string())
                                    } else {
                                        if message.author.id.0 == *OWNER_ID {
                                            Ok(capture.to_string())
                                        } else {
                                            Err("You do not have access to cross-server local tags.")
                                        }
                                    }   
                                },
                                // No location specified, returning 0 to represent global
                                None => {Ok(String::from("0"))}
                            } {
                                // User was allowed to recieve key
                                Ok(key) => {
                                    let embed = build_tag_embed(tag, key).await;
                                    client.create_message(message.channel_id).embed(embed).await?;
                                },
                                // User was not allowed to recieve key, permission error
                                Err(e)  => {
                                    client.create_message(message.channel_id).content(e).await?;
                                }
                            }
                        },
                        // Not a tag, but a command since it still a match to the regex.
                        None => {
                            // Check command (1st capture group).
                            match captures.get(1).map(|m| m.as_str()) {
                                Some("tag") => {
                                    // Tag command, check 2nd capture group to find subcommand
                                    match match captures.get(2).map(|m| m.as_str()) {
                                        Some("create")   => {
                                            // Creating a new local tag
                                            // Check if name exists in global tags

                                            // Check if name exists in local tags

                                            // Check if 3rd capture group is not None

                                            // Run 3rd capture group through image url finding regex

                                            // Insert new tag into database
                                            Ok(())
                                        },
                                        Some("edit")     => {Err(String::from("Not available yet, sorry. :("))},
                                        Some("delete")   => {Err(String::from("Not available yet, sorry. :("))},
                                        Some("promote")  => {Err(String::from("Not available yet, sorry. :("))},
                                        Some("demote")   => {Err(String::from("Not available yet, sorry. :("))},
                                        Some(subcommand) => {Err(format!("`{}` is not a valid option. Perhaps you meant `f/tag create` or `f/tag edit`.\nType `f/help tag` for more info.", subcommand))},
                                        None             => {Err(String::from("You didn't provide an instruction. Maybe you were wanting `f/tag create`?\nType `f/help tag` for more info."))}
                                    } {
                                        // Tag action was completed without errors, do nothing extra
                                        Ok(_)  => {},
                                        // Tag action encountered an error, report it to user
                                        Err(e) => {
                                           client.create_message(message.channel_id).content(e).await?; 
                                        }
                                    }
                                },
                                // Both cases: A command that is not expected or does not exist, do not respond.
                                // Follows accepted Discord bot behaviour guidelines/recommendations.
                                // https://github.com/meew0/discord-bot-best-practices
                                Some(_) => {},
                                None => {}
                            }
                        }
                    };
                }
            }
        }
        _ => {}
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let token = env::var("FLIGHTLESS_TOKEN")?;

    let client = HttpClient::new(&token);

    let cluster_config = ClusterConfig::builder(&token).build();
    let cluster = Cluster::new(cluster_config);
    cluster.up().await?;

    let mut events = cluster.events().await;
    
    while let Some(event) = events.next().await {
        tokio::spawn(handle_event(event, client.clone()));
    }
    Ok(())
}
