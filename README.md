# Flightless
[logo]: ./flightless.svg
![][logo]
# Discord bot written in Twilight-rs
Discord bot runs with the crate [Twilight](https://github.com/twilight-rs/), formerly known as Dusk and before that Dawn? I honestly can't remember anymore.
### Can I be able to run this bot on my hardware using my own Token?
Yes. To an extent, this was my original intention with the bot, but it is less of a priority for me now. However you will be able to run Flightless on your own Token. Provided you set up a MongoDB database with the instructions down below.

# Installation
In your terminal run the following. (You will need rust and git installed)
`git clone https://github.com/duncy/flightless`  
`cd flightless`  
`cargo build` 
You should be able to then use the binary.
# Usage
### Before starting
#### Environment Variables
Flightless uses two environment variables. `FLIGHTLESS_MONGO_URI` and `FLIGHTLESS_TOKEN`.
You need to set these appropriately.  
`FLIGHTLESS_MONGO_URI` being in the format of `mongodb+srv://username:password@...mongodb.net` (See more: https://github.com/mongodb/mongo-rust-driver)  
`FLIGHTLESS_TOKEN` being the discord bot token provided to you at: https://discordapp.com/developers/applications.
#### Database
You must set up an owner user in the `users` MongoDB database, under the `admins` collection.  
This must be in the format of: `{ "rank": "Owner", "id": [Your discord user ID] }`
### Starting
You only need to run the binary via typing `./Flightless` into the terminal. I personally use a shell script that sets the environment variables before running the binary.
# Special thanks
I would like to extend a special thank you to [@dvtkrlbs](https://github.com/dvtkrlbs) and [@Erk-](https://github.com/Erk-) for helping me with this project, They are epic. :)