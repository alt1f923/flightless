#[macro_use] extern crate lazy_static;
use twilight_gateway::{
    Event, 
    Cluster, 
};
use twilight_model::channel::embed::*;
use twilight_model::gateway::Intents;
use twilight_http::Client as HttpClient;
use futures::StreamExt;
use std::{
    env, 
    error::Error
};
use regex::Regex;
use mongodb::{
    bson::{
        doc,
        Document
    },
    Client as MongoClient,
    Database,
    options::FindOneOptions,
};
use chrono::Utc;

lazy_static! {
    // Regex for messages, matches messages that start with f/tag/, f/here/id:tag, f/command etc syntax
    static ref MESSAGE_REGEX: Regex = Regex::new(r"^f/ *(?:(?:(tags?|help|info|timezone|translate) +([^\s=]*) +(.*\S) *= *((?:.*\n*\r*)*))|(?:(here|\d{18}):)?(\S.*))$").unwrap();
    // Regex for creating and editing tags, captures image urls
    static ref IMAGE_URL_REGEX: Regex = Regex::new(r"^(.*)(https?://(?:[a-z0-9\-]+\.)+[a-z]{2,6}(?:/[^/#?]+)+.(?:(?:jp(?:g|eg)|webp|gif|png)|(?:JP(?:G|EG)|WEBP|GIF|PNG)))(.*)$").unwrap();
}

async fn get_database_client() -> Result<MongoClient, Box<dyn Error + Send + Sync>> {
    let client = MongoClient::with_uri_str(&env::var("FLIGHTLESS_MONGO_URI").unwrap()).await?;
    Ok(client)
}

async fn get_tags_database() -> Result<Database, Box<dyn Error + Send + Sync>> {
    let client = get_database_client().await?;
    let db = client.database("tags");
    Ok(db)
}

async fn get_owner_id() -> Result<u64, Box<dyn Error + Send + Sync>> {
    let client = get_database_client().await?;
    let db = client.database("users");

    let collection = db.collection("admins");

    let filter = doc! { "rank": "Owner" };
    let find_one_options = FindOneOptions::builder().build();
    let result = collection.find_one(filter, find_one_options).await?;

    Ok(match result {
        Some(document) => {document.get_i64("id").unwrap() as u64}
        // Unable to find owner id, owner id might not be set, set owner id to 0
        None => {0}
    })
}

fn get_filter_and_options(tag: &str) -> (Document, FindOneOptions) {
    // Finding the tag in the keys collection
    let filter = doc! { "name": tag };
    let find_one_options = FindOneOptions::builder().build();

    // Return tuple containing filter and options
    (filter, find_one_options)
}

async fn get_tag(tag: &str, key: &str) -> Result<Option<Document>, Box<dyn Error + Send + Sync>> {
    // Get tags database
    let tags_database = get_tags_database().await?;
    // Create the collection using the location as a key
    let collection = tags_database.collection(&key);
    // Get filter and options from function
    let (filter, find_one_options) = get_filter_and_options(tag);
    // Get result from database
    let result = collection.find_one(filter, None).await?;
    // Return result
    Ok(result)
}

async fn create_tag(tag:&str, key:String, image:Option<&str>, text:Option<String>) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Get tags database
    let tags_database = get_tags_database().await?;
    // Create the collection using the location as a key
    let collection = tags_database.collection(&key);
    // Create doc with the tag name
    let mut doc = doc! { "name": tag };
    // Insert text content and image url into document
    if let Some(content) = text {
        doc.insert("content", content);
    }
    if let Some(image_url) = image {
        doc.insert("image", image_url);
    }
    // Insert the document (tag) into the collection (local to server id).
    collection.insert_one(doc, None).await?;
    // Return OK result
    Ok(())
}

// TODO: delete_tag, edit_tag, promote_tag, demote_tag etc functions
// Implement also permission checkers and way of tracking owners of tags

async fn edit_tag(tag:&str, key:String, image:Option<&str>, text:Option<String>) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Get tags database
    let tags_database = get_tags_database().await?;
    // Create the collection using the location as a key
    let collection = tags_database.collection(&key);
    // Create query to be used to find the tag via its name
    let query = doc! { "name": tag };
    // Create empty document
    let mut doc  = Document::new();
    // Insert items into document
    if let Some(content) = text {
        doc.insert("content", content);
    }
    if let Some(image_url) = image {
        doc.insert("image", image_url);
    }
    // Update tag
    collection.update_one(query, doc, None).await?;
    Ok(())
}

async fn delete_tag(tag: &str, key: String) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Get tags database
    let tags_database = get_tags_database().await?;
    // Create the collection using the location as a key
    let collection = tags_database.collection(&key);
    // Create query to be used to find the tag via its name
    let query = doc! { "name": tag };
    // Delete tag
    collection.delete_one(query, None).await?;
    Ok(())
}

async fn check_if_tag_exists(tag: &str, local_scope: &str) -> Result<u8, Box<dyn Error + Send + Sync>> {
    // Check if the tag exists in either scope
    match get_tag(tag, "0").await? {
        Some(_) => {
            // The tag already exists in the global scope
            Ok(0)
        }
        None => {
            // Check if name exists in local tags
            match get_tag(tag, local_scope).await? {
                Some(_) => {
                    // The tag already exists in local scope
                    Ok(1)
                }
                None => {
                    // The tag does not exist
                    Ok(2)
                }
            }
        }
    }
}



async fn build_tag_embed(cursor: Document) -> Embed {
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
            if !message.author.bot {
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
                                        let owner_id = get_owner_id().await?;
                                        if message.author.id.0 == owner_id {
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
                                    // Get Database cursor for tag at key location
                                    match get_tag(tag, &key).await? {
                                        Some(cursor) => {
                                            // Build embed using database content via the cursor
                                            let embed = build_tag_embed(cursor).await;
                                            // Send the embed 
                                            client.create_message(message.channel_id).embed(embed)?.await?;
                                            }
                                        None => {
                                            // Tag was not found in previous scope, if it checked global, then check local
                                            if key == "0" {
                                                if let Some(cursor) = get_tag(tag, &message.guild_id.unwrap().0.to_string()).await? {
                                                    // Build embed using database content via the cursor
                                                    let embed = build_tag_embed(cursor).await;
                                                    // Send the embed 
                                                    client.create_message(message.channel_id).embed(embed)?.await?;
                                                }
                                            }
                                         }
                                    }
                                },
                                // User was not allowed to recieve key, permission error
                                Err(e)  => {
                                    client.create_message(message.channel_id).content(e)?.await?;
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
                                        Some("create") => {
                                            // Creating a new local tag
                                            // No need to check whether capture group 4 or 5 are some, they are guaranteed to exist because of the regex match
                                            let tag = captures.get(3).map(|m| m.as_str()).unwrap();
                                            let local_scope = &message.guild_id.unwrap().0.to_string();

                                            // Check if tag exists in any scope
                                            match check_if_tag_exists(tag, local_scope).await? {
                                                0 | 1 => {
                                                    // Tag exists in global or local scope, inform the user and do not create tag
                                                    Err(String::from("A tag by this name already exists, try again with a different name."))
                                                },
                                                _ => {
                                                    // Run 3rd capture group through image url finding regex
                                                    match captures.get(4).map(|m| m.as_str()) {
                                                        Some(content) => {let (content, image) = if IMAGE_URL_REGEX.is_match(&content) {
                                                            let captures = IMAGE_URL_REGEX.captures(content).unwrap();
                                                            // Use captures from regex match to define image url as option (None if not found)
                                                            let image = captures.get(2).map(|m| m.as_str());
                                                            // Use captures from regex match to create string
                                                            let mut text = String::new();
                                                            if let Some(first_half) = captures.get(1).map(|m| m.as_str()) {
                                                                text.push_str(first_half);
                                                            }
                                                            if let Some(last_half) = captures.get(3).map(|m| m.as_str()) {
                                                                text.push_str(last_half);
                                                            }
                                                            // If no text was added from captures then define text as None
                                                            let mut content = None;
                                                            if !text.is_empty(){
                                                                content = Some(text);
                                                            }
                                                            (content, image)
                                                            } else {
                                                                (Some(String::from(content)), None)
                                                            };
                                                            // Insert new tag into database
                                                            let key = message.guild_id.unwrap().0.to_string();
                                                            create_tag(tag, key, image, content).await?;
                                                            Ok(())
                                                        },
                                                        None => {
                                                            // Tag content was not defined - THIS DOES NOT WORK
                                                            Err(String::from("Tag content was not defined."))
                                                        }
                                                    }
                                                }  
                                            } 
                                        },
                                        Some("edit")     => {
                                            // Creating a new local tag
                                            // No need to check whether capture group 4 or 5 are some, they are guaranteed to exist because of the regex match
                                            let tag = captures.get(3).map(|m| m.as_str()).unwrap();
                                            let local_scope = message.guild_id.unwrap().0.to_string();

                                            let tag_exists = check_if_tag_exists(tag, &local_scope).await?;
                                            
                                            // Check if name exists in global tags
                                            match tag_exists {
                                                0 | 1 => {
                                                    // Tag was found in global scope

                                                    // TODO: Add checks for permission to edit this tag - Stuff like is this user the owner of the tag or a 
                                                    // bot admin?

                                                    // Run 3rd capture group through image url finding regex
                                                    if let Some(content) = captures.get(4).map(|m| m.as_str()) {
                                                        let (content, image) = if IMAGE_URL_REGEX.is_match(&content) {
                                                            let captures = IMAGE_URL_REGEX.captures(content).unwrap();
                                                            // Use captures from regex match to define image url as option (None if not found)
                                                            let image = captures.get(2).map(|m| m.as_str());
                                                            // Use captures from regex match to create string
                                                            let mut text = String::new();
                                                            if let Some(first_half) = captures.get(1).map(|m| m.as_str()) {
                                                                text.push_str(first_half);
                                                            }
                                                            if let Some(last_half) = captures.get(3).map(|m| m.as_str()) {
                                                                text.push_str(last_half);
                                                            }
                                                            // If no text was added from captures then define text as None
                                                            let mut content = None;
                                                            if !text.is_empty(){
                                                                content = Some(text);
                                                            }
                                                            (content, image)
                                                        } else {
                                                            (Some(String::from(content)), None)
                                                        };

                                                        // Insert new tag into database
                                                        let key;
                                                        if tag_exists == 1 {
                                                            key = message.guild_id.unwrap().0.to_string();
                                                        } else {
                                                            key = local_scope;
                                                        }
                                                        
                                                        edit_tag(tag, key, image, content).await?;
                                                    }
                                                    Ok(())
                                                },
                                                _ => {
                                                    // Tag does not exist, can not edit
                                                    Err(String::from("No tag by that name was found."))
                                                }
                                            }
                                        }
                                        Some("delete")   => {
                                            let tag = captures.get(3).map(|m| m.as_str()).unwrap();
                                            let local_scope = message.guild_id.unwrap().0.to_string();

                                            match check_if_tag_exists(tag, &local_scope).await? {
                                                0 => {
                                                    // Tag was found in global scope
                                                    delete_tag(tag, String::from("0")).await?;
                                                },
                                                1 => {
                                                    // Tag was found in local scope
                                                    delete_tag(tag, local_scope).await?;
                                                },
                                                _ => {
                                                    // Tag was not found, do nothing
                                                }
                                            }
                                            Ok(())
                                        },
                                        Some("promote")  => {Err(String::from("Not available yet, sorry. :("))},
                                        Some("demote")   => {Err(String::from("Not available yet, sorry. :("))},
                                        Some(subcommand) => {Err(format!("`{}` is not a valid option. Perhaps you meant `f/tag create` or `f/tag edit`.\nType `f/help tag` for more info.", subcommand))},
                                        None             => {Err(String::from("You didn't provide an instruction. Maybe you were wanting `f/tag create`?\nType `f/help tag` for more info."))}
                                    } {
                                        // Tag action was completed without errors, do nothing extra
                                        Ok(_)  => {},
                                        // Tag action encountered an error, report it to user
                                        Err(e) => {
                                           client.create_message(message.channel_id).content(e)?.await?; 
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

    let intents = Intents::GUILD_MESSAGES;
    let cluster = Cluster::builder(&token).intents(intents).build().await?;
    cluster.up().await;

    let mut events = cluster.events();
    
    while let Some(event) = events.next().await {
        tokio::spawn(handle_event(event, client.clone()));
    }
    Ok(())
}
