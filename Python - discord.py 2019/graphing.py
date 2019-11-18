import discord
import plotly
import os

class MyClient(discord.Client):
    async def on_ready(self):
        import plotly.graph_objects as go
        from plotly.subplots import make_subplots

        labels = ["Asia", "Europe", "Africa", "Americas", "Oceania"]

        fig = make_subplots(1, 2, specs=[[{'type':'domain'}, {'type':'domain'}]],
                            subplot_titles=['1980', '2007'])
        fig.add_trace(go.Pie(labels=labels, values=[4, 7, 1, 7, 0.5], scalegroup='one',
                            name="World GDP 1980"), 1, 1)
        fig.add_trace(go.Pie(labels=labels, values=[21, 15, 3, 19, 1], scalegroup='one',
                            name="World GDP 2007"), 1, 2)

        fig.update_layout(title_text='World GDP')
        img_bytes = fig.to_image(format="png")
        channel_send = self.get_channel(544760293570379786)
        await channel_send.send(file=discord.File(img_bytes))


client = MyClient()
client.run(os.environ['FLIGHTLESS_TOKEN'])