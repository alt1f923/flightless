import discord
import asyncio
import sys

class Flightless(discord.Client):
    async def on_ready(self):
        print(
            f"Ready as: {self.user.name}",
            f"Running Discord.py v{discord.__version__}",
            f"Serving {len(self.guilds)} guilds with a total of {len(self.users)} users",
            sep='\n'
        )
        
    
client = Flightless()


async def connect(token):
    print("Logging in...")
    try:
        await client.login(token, bot=True)
        print("Connecting...")
        await client.connect(reconnect=True)
    except discord.errors.LoginFailure:
        print("Invalid token provided", file=sys.stderr)
        await client.close()
    except discord.errors.HTTPException as e:
        print(f"HTTP request operation failed, status code: {e.status}", file=sys.stderr)
    except discord.errors.GatewayNotFound:
        print("Cannot reach Discord gateway, possible Discord API outage", file=sys.stderr)
    except discord.errors.ConnectionClosed:
        print("The websocket connection has been terminated", file=sys.stderr)

async def disconnect():
    # Logout
    await client.logout()
    print("Disconnected")

def run(token):
    # Create the loop
    loop = asyncio.get_event_loop()
    try:
    # Connect to Discord using the token stored as one of the system's environment variables
        loop.run_until_complete(connect(token))
    except KeyboardInterrupt:
    # If a keyboard interupt is sent to the console, send the bot into invisible mode and log it out
        loop.run_until_complete(disconnect())
    finally:
        # Close the loop
        loop.close()

        

    
def main():
    if len(sys.argv) != 2: # File and token ["bot.py", "token"]
        print(f"Usage: python {sys.argv[0]} token", file=sys.stderr)
    else:
        # Run the bot with the token provided from the arguments
        run(sys.argv[1])


if __name__ == "__main__":
    main()