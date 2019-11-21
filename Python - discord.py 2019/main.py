from bot import Flightless, sys
    
def main():
    if len(sys.argv) != 2: # File and token ["bot.py", "token"]
        print(f"Usage: python {sys.argv[0]} token", file=sys.stderr)
    else:
        # Run the bot with the token provided from the arguments
        client = Flightless(sys.argv[1])
        client.run()

if __name__ == "__main__":
    main()