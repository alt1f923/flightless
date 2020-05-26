# Flightless
[logo]: ./flightless.svg
![][logo]
# Discord bot written in Twilight-rs
Dawn has been given renewed life and is now operating under the name 'Twilight'.  
Because of this I will be migrating my bot back to Rust.
### Can I be able to run this bot on my hardware using my own Token?
Yes. To an extent, you will be able to run Flightless on your own Token, even provide your own custom command prefix from the command terminal when you start it.

# Installation
### WIP
# Usage
### Before starting
#### Environment Variables
Flightless uses two environment variables. `FLIGHTLESS_MONGO_URI` and `FLIGHTLESS_TOKEN`.
You need to set these appropriately.  
`FLIGHTLESS_MONGO_URI` being in the format of `mongodb+srv://username:password@...mongodb.net` (See more: https://github.com/mongodb/mongo-rust-driver)  
`FLIGHTLESS_TOKEN` being the discord bot token provided to you at: https://discordapp.com/developers/applications.
#### Database
You must set up an owner user in the `users` mongodb database, under the `admins` collection.  
This must be in the format of: `{ "rank": "Owner", "id": [Your discord user ID] }`
### Starting
You only need to run the binary via typing `./Flightless` into the terminal or by running it otherwise.
# Special thanks
I would like to extend a special thank you to [@dvtkrlbs](https://github.com/dvtkrlbs) and [@Erk-](https://github.com/Erk-) for helping me with this project. :)