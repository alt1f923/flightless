import discord                          # Interacting with the Discord API
import asyncio                          # For creating async loops
import sys                              # For sending error messages to stderr and accessing argv
from datetime import datetime           # For adding UTC timestamps to embeds
import re                               # Regex for parsing text for command and url detection
import shelve                           # For persistency regarding tags
import requests                         # For checking if urls to images exist for tags
from googletrans import Translator      # Free (unofficial) google translate API https://github.com/ssut/py-googletrans
import matplotlib.pyplot as plt
from bisect import bisect
import numpy as np

PREFIX = "f/"

def plot_colour(d):
    return [d.r/255, d.g/255, d.b/255]

class Tag():
    def __init__(self, name, owner=165765321268002816, reply=None, url=None):
        self.reply  = reply             # This is the text that is returned to the user in the embed
        self.name   = name              # Name of the tag, same as key 
        self.owner  = owner             # ID int of user who created the tag
        self.date   = datetime.now()    # Datetime time object of tags creation
        self.image  = url               # Image from message content, will be shown as an embed thumbnail

    def __str__(self):
        return f"Command: {self.name}\nOwner id: {self.owner}\nCreated: {self.date.strftime('%y/%m/%d %H:%M:%S')}"


class Flightless(discord.Client):
    def __init__(self, token):
        super().__init__()
        # The reason token is set here is so I can disconnect the bot and reconnect it without restarting the code or carrying the token around as a global
        self.token               = token
        # Regex following the format of "f/word word word word"
        self.message_parser      = re.compile(r"^{}(\S+) *(\w*) *(\S*) *((.*\n*\r*)*)$".format(PREFIX))
        # Regex following the format of "https://www.website.com/image.png"
        # TODO: []() exclusion
        self.image_url_parser    = re.compile(r"^([^\2]*)(https?:\/\/(?:[a-z0-9\-]+\.)+[a-z]{2,6}(?:\/[^\/#?]+)+\.(?:(?:jp(?:g|eg)|webp|gif|png)|(?:JP(?:G|EG)|WEBP|GIF|PNG)))([^\2]*)$")
        # Aliases for existing commands, both user submitted and not, filled by loading from shelve
        self.aliases             = {}
        # Basic commands, just text replies, user created commands stored in here too, filled by loading from shelve
        self.bc                  = {}
        # Non basic commands, need functions and discord interactions to complete              
        self.nbc                 = {"tags": self.tags_command,
                                    "tag": self.tag_command,
                                    "aliases": self.aliases_command,
                                    "top": self.top_command,
                                    "time": self.time_command,
                                    "translate": self.translate_command,
                                    "vote": self.translate_command,
                                    "help": self.help_command}
        # Dictionary for storing the score data for the top command
        self.guilds_score        = {}
        # Translator from googletrans, used for translate command 
        self.translator          = Translator()
        # Ready, can use bot once this is True
        self.top_ready           = {}

    async def on_ready(self):

        print(
            f"Ready as: {self.user.name}",
            f"Running Discord.py v{discord.__version__}",
            f"Serving {len(self.guilds)} guilds with a total of {len(self.users)} users",
            sep='\n')

        # g = self.get_guild(198337653412855808)
        # await g.create_custom_emoji(name="howiusedtosee", image=open("eye_v1.png", 'rb').read(), roles=[g.get_role(663346825775808521)])
        # await g.create_custom_emoji(name="themostieversaw", image=open("eye_v8.png", 'rb').read(), roles=[g.get_role(663346825775808521)])

        await self.count_messages()

    async def on_message(self, message):
        if not message.author.bot:
            # TODO: Add blacklist check here
            if parsed_message := self.message_parser.match(message.content):
                parsed_message = parsed_message.groups()
                command = parsed_message[0]
                if self.alias_exists(command):
                    command = self.alias(command)
                    if command_tag := self.bc.get(command, False):
                        await self.send_tag(command_tag, message.channel)
                    else:
                        await self.nbc[command](parsed_message, message)
            if not self.top_ready.get(message.guild.id, False):
                if message.guild.id not in self.guilds_score.keys():
                    self.guilds_score[message.guild.id] = [1, {message.author.id: [1, message.author.name, message.author.colour, message.author.roles[-1].id]}]
                else:
                    self.guilds_score[message.guild.id][0] += 1
                    if message.author.id not in self.guilds_score[message.guild.id][1].keys():
                        self.guilds_score[message.guild.id][1][message.author.id] = [1, {message.author.id: [1, message.author.name, message.author.colour, message.author.roles[-1].id]}]
                    else:
                        self.guilds_score[message.guild.id][1][message.author.id][0] += 1

    async def count_messages(self):
        # Counting messages by user by server for the top command
        # TODO: Update it so each guild can use top command once its dataset in particular is ready
        print("Starting count...", end="\r")
        for guild in self.guilds[1:2]:
            users = {}
            total = 0

            for channel in guild.text_channels:
                try:
                    async for message in channel.history(limit=None):
                        if (author := message.author) in guild.members:
                            if not author.bot:
                                if (author_id := author.id) not in users.keys():
                                    users[author_id] = [1, author.name, author.colour, author.roles[-1].id]
                                else:
                                    users[author_id][0] += 1
                                total += 1
                except discord.errors.Forbidden:
                    continue
            self.guilds_score[guild.id] = [total, users]
            self.top_ready[guild.id] = True
            print(f"Top command is  available for {len(self.top_ready)} guilds", end='\r')
        print(f"Top command is available for all guilds. ({len(self.top_ready)})")

    def seperate_url(self, content):
        url = None
        if parsed_message := self.image_url_parser.match(content):
            parsed_message = parsed_message.groups()
            url = parsed_message[1]
            content = parsed_message[0] + parsed_message[2]
        return url, content

    def image_exists(self, url):
        return requests.head(url).status_code == 200 # 200 status code means url exists, consider adding 301, 302, 303, 307, 308

    async def send_tag(self, tag, channel):
        if tag.image:
            if not self.image_exists(tag.image):
                tag.image = None
                self.save_tags()
        await self.send_embed(channel, content=tag.reply, footer=f"{self.get_user(tag.owner)}'s tag", image=tag.image)
        
    async def send_embed(self, channel, content=None, title=None, footer=None, fields=None, image=None):
        # TODO: Add code to limit how much content can be sent to avoid exceeding byte limit
        if not title:
            title = self.user.name.capitalize()
        embed = discord.Embed(colour=discord.Colour(0x985F35), description=content, timestamp=datetime.utcnow())
        embed.set_author(name=title, icon_url=self.user.avatar_url)
        if fields:
            for field in fields: # field = [name, value, inline]
                embed.add_field(name=field[0], value=field[1], inline=field[2])
        if image:
            embed.set_image(url=image)
        if not footer:
            footer = f"{self.user.name.capitalize()} running in {channel.guild.name}"
        embed.set_footer(text=footer)
        await channel.send(embed=embed)

    async def tags_command(self, input, message):
        fields = []
        
        content = ""
        for tag in self.bc.keys():
            content += f"{tag}\n"
        fields.append(["Tags", content, True])
        content = ""
        for command in self.nbc.keys():
            content += f"{command}\n"
        fields.append(["Commands", content, True])
        
        await self.send_embed(message.channel, title=f"{self.user.name.capitalize()}' reserved Commands/Tags", fields=fields) # Hardcoded ' instead of 's since Flightless ends with a 's'

    async def tag_command(self, input, message):
        # TODO: Add request feature and approval system
        if (instruction := input[1].lower()) == "create":
            if self.new_tag(owner=message.author.id, name=(name := input[2].lower()), reply=input[3]): # owner, name, reply
                await self.send_tag(self.bc[name], message.channel)
            else:
                await self.send_embed(message.channel, content="Tag could not be created.\nPlease make sure its name is not already in use.")
        elif instruction == "edit":
            if self.edit_tag((name := input[2].lower()), message.author.id, input[3]): # name, user, new_reply
                await self.send_tag(self.bc[name], message.channel)
            else:
                await self.send_embed(message.channel, content="Tag could not be edited.\nYou can only edit tags that you own or that exist.")
        elif instruction == "delete":
            if self.delete_tag((name := input[2].lower()), message.author.id): # name, user
                await self.send_embed(message.channel, content=f"Tag `{name}` deleted.")
            else:
                await self.send_embed(message.channel, content="Tag could not be deleted.\nYou can only delete tags that you own or that exist.")
        else:
            await self.tags_command(None, message)
        

    async def aliases_command(self, input, message):
        field_one = ""
        field_two = ""
        for alias in self.aliases.keys():
            field_one += f"{alias}\n"
            field_two += f"{self.aliases[alias]}\n"
        fields = [["Alias", field_one, True], ["Command/Tag", field_two, True]]
        await self.send_embed(message.channel, title=f"{self.user.name.capitalize()}' reserved Aliases for Commands/Tags", fields=fields) # Hardcoded ' instead of 's since Flightless ends with a 's'  

    async def top_command(self, input, message):
        if self.top_ready.get(message.guild.id, False):
            users = self.guilds_score[message.guild.id]
            total = users[0]
            users = users[1]

            values = {}
            labels = {}
            colours = {}
            other_users = 0

            for user in users.keys():
                score, name, colour, role = users[user]
                if (percentage := score/total) < 0.0025:
                    other_users += score 
                else: 
                    if role not in values.keys():
                        values[role] = [score]
                        labels[role] = [f"{name} {(percentage * 100):.2f}%"]
                        colours[role] = [plot_colour(colour)]
                    else:
                        values[role].insert((index := bisect(values[role], score)), score)
                        labels[role].insert(index, f"{name} {(percentage * 100):.2f}%")
                        colours[role].insert(index, plot_colour(colour))
            x = []
            y = []
            z = []
            
            for key in values.keys():
                x += values[key]
                label = ""
                if len(labels[key]) > 1:
                    label += f"{message.guild.get_role(key).name}\n\n"
                for item in labels[key]:
                    label += f"{item}\n"
                y.append([label.strip(), len(labels[key])])
                z += colours[key]

            if other_users:
                x.append(other_users)
                y.append([f"Other users {(other_users/total * 100):.2f}%", 1])
                z.append([0, 0, 0])

            ax = plt.subplots(figsize=(10, 9), dpi=180, subplot_kw={"aspect" : "equal"})[1]

            wedges = ax.pie(x, colors=z, wedgeprops={"width" : 0.5, "edgecolor":"0", 'linewidth': 1, 'linestyle': 'solid', 'antialiased': True}, startangle=0, counterclock=False)[0]

            kw = {"arrowprops" : {"arrowstyle" : "-"}, "bbox" : {"boxstyle" : "square,pad=0.6", "fc" : "w", "ec" : "k", "lw" : 0.72}, "zorder" : 0, "va" : "center"}

            labels = y.copy()
            count = 1
            for p in wedges:
                ang = (p.theta2 - p.theta1)/2. + p.theta1
                y = np.sin(np.deg2rad(ang))
                x = np.cos(np.deg2rad(ang))
                horizontalalignment = {-1: "right", 1: "left"}[int(np.sign(x))]
                connectionstyle = "angle,angleA=0,angleB={}".format(ang)
                kw["arrowprops"].update({"connectionstyle": connectionstyle})
                if count == ((labels[0][1]) // 2) + 1:
                    ax.annotate(labels[0][0], xy=(x, y), xytext=(1.35*np.sign(x), 1.4*y), horizontalalignment = horizontalalignment, **kw)
                    count = -(labels[0][1] - count) + 1
                    del labels[0]
                else:
                    count += 1

            ax.set_title(f"{message.guild.name.capitalize()}")
            plt.savefig(f"images/{message.guild.id}.png", bbox_inches="tight")

            await message.channel.send(content=message.author.mention, 
                                        file=discord.File(fp=open(f"images/{message.guild.id}.png", "rb"), filename=f"{message.guild.name}.png"))
        else:
            await self.send_embed(message.channel, content="I am still counting messages for this command sorry. Try again later.")

    async def time_command(self, input, message):
        await self.niy_command("Time", message.channel)

    async def translate_command(self, input, message):
        # TODO: add ability to change destination and set src lang and error handling
        translated = self.translator.translate((text := f"{input[1]} {input[2]} {input[3]}"))
        fields = [["Original", text, False], ["Translated", translated.text, False]]
        await self.send_embed(message.channel, title="Translate", footer=f"Translated from {translated.src} to {translated.dest} using Google Translate", fields=fields)

    async def help_command(self, input, message):
        await self.niy_command("Help", message.channel)

    async def niy_command(self, command, channel): # Not implemented yet command
        await self.send_embed(channel, content=f"{command} is not implemented yet.")

    def load_tags(self):
        with shelve.open("tags") as tags:
            for key in tags.keys():
                self.bc[key] = tags[key]
        tags.close()
        with shelve.open("aliases") as aliases:
            for key in aliases.keys():
                self.aliases[key] = aliases[key]
        aliases.close()

    def save_tags(self):
        with shelve.open("tags") as tags:
            tags.clear()
            for key in self.bc.keys():
                tags[key] = self.bc[key]
        tags.close()
        with shelve.open("aliases") as aliases:
            for key in self.aliases.keys():
                aliases[key] = self.aliases[key]
        aliases.close()

    def new_tag(self, owner, name, reply):
        if name.strip():
            if not self.alias_exists(name):
                url, reply = self.seperate_url(reply)
                if url:
                    if reply:
                        self.bc[name] = Tag(name, owner, reply=reply, url=url) # name, owner, reply, url
                    else:
                        self.bc[name] = Tag(name, owner, url=url) # name, owner, url
                else:
                    self.bc[name] = Tag(name, owner, reply=reply) # name, owner, reply
                self.save_tags()
                return True
            return False

    def edit_tag(self, name, user, new_reply):
        if tag := self.get_tag(name): # Excluding non-basic commands
            if self.is_tag_owner(tag, user):
                url, reply = self.seperate_url(new_reply)
                if url:
                    tag.image = url
                else:
                    tag.image = None
                if reply:
                    tag.reply = reply
                else:
                    tag.reply = None
                self.save_tags()
                return True
        return False

    def delete_tag(self, name, user):
        if tag := self.get_tag(name):
            if self.is_tag_owner(tag, user):
                del self.bc[tag.name]
                return_code = self.delete_aliases(name)
                if return_code:
                    self.save_tags()
                else:
                    self.load_tags()
                return return_code
        return False

    def get_tag(self, name):
        return self.bc.get(self.alias(name), False) # Excluding non-basic commands

    def is_tag_owner(self, tag, user):
        return user in [tag.owner, 165765321268002816]

    def alias_exists(self, alias):
        name = self.alias(alias)
        return self.bc.get(name, False) or self.nbc.get(name, False)

    def new_alias(self, name, alias):
        if not self.alias_exists(alias):
            if self.alias_exists(name):
                self.aliases[alias] = name
                return True
        return False
    
    def delete_aliases(self, name):
        try:
            for alias in self.aliases.keys():
                if self.aliases[alias] == name:
                    del self.aliases[alias]
            return True
        except:
            return False

    def alias(self, name):
        """Dictionary lookup to find a command from its alias, if found it will return the first value of alias (commands name) otherwise will return base command name"""
        return self.aliases.get(name, name)

    async def start(self):
        print("Logging in...", end="\r")
        try:
            await self.login(self.token, bot=True)
            print("Logged in.   ")
            # Load the database of commands now that a connection to Discord has been established and the bot is logged in
            print("Loading tags...", end="\r")
            self.load_tags()
            print(f"Loaded {len(self.bc)} tags:\n{[*self.bc]}")
            print("Connecting...", end="\r")
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
        finally:
            # After the connection has ended, save the tags, this is redunant as any edit or new tag will be saved as part of the process of creation/change however, just a precaution
            print("Saving tags...", end="\r")
            self.save_tags()
            print(f"Saved {len(self.bc)} tags.  ")


    async def disconnect(self):
        # Logout
        await self.logout()
        print("Disconnected.")

    def run(self):
        # Create the loop
        loop = asyncio.get_event_loop()
        try:
            # Connect to Discord using the token stored as one of the system's environment variables
            loop.run_until_complete(self.start())
        except KeyboardInterrupt:
            # If a keyboard interupt is sent to the console log the bot out
            loop.run_until_complete(self.disconnect())
        finally:
            # Close the loop
            loop.close()
