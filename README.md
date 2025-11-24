# discord-bot-sorting-hat

A Discord bot written in Rust that sorts new members into houses through an interactive quiz. When new members join the server, they receive a direct message with questions in Portuguese and, based on their answers, are assigned to one of four houses with the corresponding role.

## Features

- Automatically detects when new members join the server
- Sends a private quiz via DM to new members (questions in Portuguese)
- Sorts members into one of four houses based on their answers
- Automatically assigns the appropriate house role
- Persistent storage: remembers sorted users even after bot restarts
- Prevents users from being sorted multiple times
- Fully configurable: houses, roles, questions, and trait mapping

## Prerequisites

- Rust (1.70 or later)
- A Discord bot token
- A Discord server where you have admin permissions

## Discord Bot Setup

1. Go to the [Discord Developer Portal](https://discord.com/developers/applications)
2. Click "New Application" and give it a name
3. Go to the "Bot" section and click "Add Bot"
4. Under "Privileged Gateway Intents", enable:
   - Server Members Intent
   - Message Content Intent
5. Copy the bot token (you'll need this later)
6. Go to "OAuth2" > "URL Generator"
7. Select the following scopes:
   - `bot`
8. Select the following bot permissions:
   - Manage Roles
   - Send Messages
   - Read Message History
9. Copy the generated URL and open it in your browser to invite the bot to your server

## Server Setup

1. Create four roles in your Discord server matching the house names in `config.json`:
   - Draco (Red #D7263D and Gold #F5C400)
   - Lupus (Gray #4F4F4F and Black #000000)
   - Tigris (Royal Blue #1A56DB and Gold #E3AC00)
   - Aeternum (Purple #6B2FB7 and Silver #CFCFCF)

2. Make sure the bot's role is positioned ABOVE these house roles in the server settings
   (Server Settings > Roles > Drag the bot role above house roles)

3. Set up channel permissions so that:
   - New members can only see a welcome/sorting channel by default
   - Each house role grants access to its respective house channels
   - All house roles grant access to the main server channels

4. Get your Server (Guild) ID:
   - Enable Developer Mode in Discord (User Settings > Advanced > Developer Mode)
   - Right-click your server icon and select "Copy ID"

### About the Houses

**Draco**
- **Values**: Courage, creativity, initiative, intensity
- **Motto**: "Onde criamos, queimamos limites." (Where we create, we burn limits.)
- **Description**: House of bold leaders, creative and driven by action.

**Lupus**
- **Values**: Loyalty, discipline, community, focus
- **Motto**: "Sozinho vai rapido. Em equipe vai longe." (Alone you go fast. Together you go far.)
- **Description**: House of the disciplined, collaborative, and united like a pack.

**Tigris**
- **Values**: Ambition, excellence, focus, high performance
- **Motto**: "Nada menos que o melhor." (Nothing less than the best.)
- **Description**: House of the ambitious, high-performers focused on excellence.

**Aeternum**
- **Values**: Freedom, innovation, vision, autonomy
- **Motto**: "Enxerga alem do codigo." (See beyond the code.)
- **Description**: House of visionaries, creative and free.

## Installation

1. Clone this repository:
```bash
git clone <your-repo-url>
cd discord-bot-sorting-hat
```

2. Create a `.env` file from the example:
```bash
cp .env.example .env
```

3. Edit `.env` and fill in your values:
```env
DISCORD_TOKEN=your_bot_token_here
GUILD_ID=your_guild_id
```

4. (Optional) Customize the houses and questions in `config.json`

5. Build and run the bot:
```bash
cargo build --release
cargo run --release
```

## Configuration

### Houses (`config.json`)

The `houses` array defines the available houses. Each house has:
- `name`: Display name of the house
- `role_name`: The exact name of the role in Discord (must match exactly)
- `description`: Description shown when a user is sorted (in Portuguese)
- `traits`: Array of traits associated with this house

### Questions (`config.json`)

The `questions` array defines the sorting questions (in Portuguese). Each question has:
- `question`: The question text (in Portuguese)
- `options`: Array of 4 answer options

Each option has:
- `text`: The answer text shown to the user (in Portuguese)
- `traits`: Object mapping trait names to point values

The sorting algorithm calculates which house a user should join based on the total points accumulated for each trait. The current configuration includes 5 questions designed to identify personality traits aligned with each house's values.

### Data Persistence (`sorted_users.json`)

The bot automatically creates and maintains a `sorted_users.json` file to persist information about users who have been sorted. This ensures that:

- Users are only sorted once, even if they leave and rejoin the server
- Sorted user data persists across bot restarts
- Users attempting to sort again receive a message indicating their existing house

The file is created automatically on first sort and updated each time a user completes the sorting process. You don't need to manually create or edit this file.

**Example format:**
```json
{
  "123456789012345678": {
    "house_name": "Draco",
    "sorted_at": "2025-01-15T10:30:00Z"
  }
}
```

**Note:** If you want to allow a user to be sorted again, you can manually remove their entry from this file (make sure the bot is stopped when editing).

## How It Works

1. When a new member joins the server, the bot checks if they've been sorted before
2. If they have been sorted before, they receive a welcome back message with their house
3. If they haven't been sorted, the bot sends them a DM in Portuguese with quiz questions
4. The user responds by entering a number (1-4) for each question
5. After all questions are answered, the bot:
   - Calculates trait scores based on the user's answers
   - Determines which house best matches those traits (Draco, Lupus, Tigris, or Aeternum)
   - Assigns the corresponding role to the user
   - Saves the result to `sorted_users.json` for persistence
   - Notifies them of their house assignment in Portuguese

## Commands

- `!testsort` - Start a test sorting session (useful for testing the bot). Note: This will not work if you have already been sorted.
- `!resetsort` - Reset your current sorting session if you want to start over (only cancels an active quiz, doesn't allow re-sorting)

## Troubleshooting

### Bot doesn't respond to new members
- Make sure "Server Members Intent" is enabled in the Discord Developer Portal
- Verify the bot has permissions to send DMs
- Check the logs for any error messages

### Role assignment fails
- Ensure the bot's role is positioned ABOVE the house roles
- Verify the role names in `config.json` match exactly (case-sensitive)
- Make sure the bot has "Manage Roles" permission

### Bot crashes on startup
- Check that `config.json` is valid JSON
- Verify all required environment variables are set in `.env`
- Ensure the IDs in `.env` are valid numbers

### User wants to be sorted again
- The bot intentionally prevents users from being sorted multiple times
- To allow a user to be sorted again:
  1. Stop the bot
  2. Open `sorted_users.json`
  3. Remove the user's entry (the key will be their Discord user ID)
  4. Save the file
  5. Restart the bot

## Development

Run tests:
```bash
cargo test
```

Run with logging:
```bash
RUST_LOG=info cargo run
```

## License

MIT

## Contributing

Feel free to open issues or submit pull requests!
