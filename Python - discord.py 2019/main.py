import discord
import asyncio
import sys

class Flightless(discord.Client):
    def __init__(self, token):
        super().__init__()
        self.token = token

    async def on_ready(self):
        print(
            f"Ready as: {self.user.name}",
            f"Running Discord.py v{discord.__version__}",
            f"Serving {len(self.guilds)} guilds with a total of {len(self.users)} users",
            sep='\n')

    async def start(self):
        print("Logging in...")
        try:
            await self.login(self.token, bot=True)
            print("Connecting...")
            await self.connect(reconnect=True)
        except discord.errors.LoginFailure:
            # Invalid token causes LoginFailure
            print("Invalid token provided", file=sys.stderr)
        except discord.errors.HTTPException as e:
            # HTTP error code raised
            print(f"HTTP request operation failed, status code: {e.status}", file=sys.stderr)
        except discord.errors.GatewayNotFound:
            # Unable to reach Discords API, the API being down will probably also mean no one will be online on the client to complain about the bot :^)
            print("Cannot reach Discord gateway, possible Discord API outage", file=sys.stderr)
        except discord.errors.ConnectionClosed:
            # Connection terminated after it was established, probably caused by internet dropping out, reconnect should take care of this
            print("The websocket connection has been terminated", file=sys.stderr)

    async def disconnect(self):
        # Logout
        await self.logout()
        print("Disconnected")

    def run(self):
        # Create the loop
        loop = asyncio.get_event_loop()
        try:
            # Connect to Discord using the token stored as one of the system's environment variables
            loop.run_until_complete(self.start())
        except KeyboardInterrupt:
            # If a keyboard interupt is sent to the console, send the bot into invisible mode and log it out
            loop.run_until_complete(self.disconnect())
        finally:
            # Close the loop
            loop.close()
    
        

    
def main():
    if len(sys.argv) != 2: # File and token ["bot.py", "token"]
        print(f"Usage: python {sys.argv[0]} token", file=sys.stderr)
    else:
        # Run the bot with the token provided from the arguments
        client = Flightless(sys.argv[1])
        client.run()


if __name__ == "__main__":
    main()