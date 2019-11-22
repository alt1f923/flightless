import discord
import asyncio
import sys
from datetime import datetime
from parse import compile, parse
import shelve

class CommandTag():
    def __init__(self, reply, name, owner=165765321268002816):
        self.reply = reply
        self.name = name
        self.owner = owner
        self.date = datetime.now()

    def __str__(self):
        return f"Command: {self.name}\nOwner id: {self.owner}\nCreated: {self.date.strftime('%y/%m/%d %H:%M:%S')}"

class Flightless(discord.Client):
    def __init__(self, token):
        super().__init__()
        # The reason token is set here is so I can disconnect the bot and reconnect it without restarting the code or carrying the token around as a global
        self.token   = token
        self.parser  = compile("f/{} {}")
        self.aliases = {} # Aliases for existing commands, both user submitted and not
        self.bc      = {} # Basic commands, just text replies, user created commands stored in here too
        self.nbc     = {"tags": self.tags_command, # Non basic commands, need functions and discord interactions to complete
                        "aliases": self.aliases_command,
                        "top": self.top_command,
                        "time": self.time_command,
                        "translate": self.translate_command,
                        "game": self.game_server_command}

    async def on_ready(self):
        print(
            f"Ready as: {self.user.name}",
            f"Running Discord.py v{discord.__version__}",
            f"Serving {len(self.guilds)} guilds with a total of {len(self.users)} users",
            sep='\n')

    async def on_message(self, message):
        if message.guild.id == 198337653412855808: # So that no server but mine will get interactions with my bot while testing --THIS IS TEMPORARY--
            if not message.author.bot:
                # TODO: Add blacklist check here
                # TODO: Go through parse source code to find a tidy solution for problem below
                # TODO: Just use regex
                try:
                    command, input = self.parser.parse(message.content + "  ") # Shhh don't tell anyone, it's 1am and I've been looking at the one page for parse to find a better solution :)
                    if self.alias_exists(command):
                        command = self.alias(command)
                        if command_tag := self.bc.get(command, False):
                            await self.send_tag(command_tag, message.channel)
                        else:
                            await self.nbc[command](input, message)
                except TypeError: # Message doesn't fit parser format
                    pass

    async def send_tag(self, tag, channel):
        await self.send_embed(channel, content=tag.reply, footer=f"{self.get_user(tag.owner)}'s tag")
        
    async def send_embed(self, channel, content=None, title=None, footer=None, fields=None):
        # TODO: Add code to limit how much content can be sent to avoid exceeding byte limit
        if not title:
            title = self.user.name.capitalize()
        embed = discord.Embed(colour=discord.Colour(0x985F35), description=content, timestamp=datetime.utcnow())
        embed.set_author(name=title, icon_url=self.user.avatar_url)
        if fields:
            for field in fields: # field = [name, value, inline]
                embed.add_field(name=field[0], value=field[1], inline=field[2])
        if not footer:
            footer = f"{self.user.name.capitalize()} running in {channel.guild.name}"
        embed.set_footer(text=footer)
        await channel.send(embed=embed)

    async def tags_command(self, input, message):
        fields = []
        
        content = ""
        for command in self.bc.keys():
            content += f"{command}\n"
        fields.append(["Tags", content, True])
        content = ""
        for command in self.nbc.keys():
            content += f"{command}\n"
        fields.append(["Commands", content, True])
        
        await self.send_embed(message.channel, title=f"{self.user.name.capitalize()}' reserved Commands/Tags", fields=fields) # Hardcoded ' instead of 's since Flightless ends with a 's'

    async def aliases_command(self, input, message):
        field_one = ""
        field_two = ""
        for alias in self.aliases.keys():
            field_one += f"{alias}\n"
            field_two += f"{self.aliases[alias]}\n"
        fields = [["Alias", field_one, True], ["Command/Tag", field_two, True]]
        await self.send_embed(message.channel, title=f"{self.user.name.capitalize()}' reserved Aliases for Commands/Tags", fields=fields) # Hardcoded ' instead of 's since Flightless ends with a 's'
        

    async def top_command(self, input, message):
        await self.niy_command("Top", message.channel)

    async def time_command(self, input, message):
        await self.niy_command("Time", message.channel)

    async def translate_command(self, input, message):
        await self.niy_command("Translate", message.channel)

    async def game_server_command(self, input, message):
        await self.niy_command("Game server", message.channel)

    async def niy_command(self, command, channel): # Not implemented yet command
        await self.send_embed(channel, content=f"{command} is not implemented yet.")

    def load_commands(self):
        with shelve.open("commands") as commands:
            for key in commands.keys():
                self.bc[key] = commands[key]
        commands.close()
        with shelve.open("aliases") as aliases:
            for key in aliases.keys():
                self.aliases[key] = aliases[key]
        aliases.close()

    def save_commands(self):
        with shelve.open("commands") as commands:
            for key in self.bc.keys():
                commands[key] = self.bc[key]
        commands.close()
        with shelve.open("aliases") as aliases:
            for key in self.aliases.keys():
                aliases[key] = self.aliases[key]
        aliases.close()

    def new_command(self, owner, name, reply):
        if not self.alias_exists(name):
            self.bc[name] = CommandTag(owner, name, reply)
            self.save_commands()
            return True
        return False

    def edit_command(self, name, user, new_reply):
        if command := self.bc.get(self.alias(name), False):
            if user in [command.owner, 165765321268002816]:
                command.reply = new_reply
                self.save_commands()
                return True
        return False

    def alias_exists(self, alias):
        name = self.alias(alias)
        return self.bc.get(name, False) or self.nbc.get(name, False)

    def new_alias(self, name, alias):
        if not self.alias_exists(alias):
            if self.alias_exists(name):
                self.aliases[alias] = name
                return True
        return False

    def alias(self, name):
        """Dictionary lookup to find a command from it's alias, if found it will return the first value of alias (commands name) otherwise will return base command name"""
        return self.aliases.get(name, name)

    async def start(self):
        print("Logging in...")
        try:
            await self.login(self.token, bot=True)
            # Load the database of commands now that a connection to Discord has been established and the bot is logged in
            print("Loading commands...")
            self.load_commands()
            print(f"Loaded {len(self.bc)} commands:\n{[*self.bc]}")
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
        else:
            # After the connection has ended, save the commands
            print("Saving commands...")
            self.save_commands()
            print("Saved")


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