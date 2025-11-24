# Discord Sorting Bot

Um bot de Discord escrito em Rust que seleciona novos membros para casas através de um quiz interativo. Quando novos membros entram no servidor, eles recebem uma mensagem direta com perguntas em português e, com base nas respostas, são designados para uma das quatro casas e recebem o cargo correspondente.

## Features

- Detecta automaticamente quando novos membros entram no servidor
- Envia um quiz privado via DM para novos membros (em português)
- Seleciona membros para uma das quatro casas baseado nas respostas
- Atribui automaticamente o cargo apropriado da casa
- Completamente configurável: casas, cargos, perguntas e mapeamento de traços

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
   - 🐉 **Draco** (Vermelho #D7263D e Dourado #F5C400)
   - 🐺 **Lupus** (Cinza #4F4F4F e Preto #000000)
   - 🐯 **Tigris** (Azul Royal #1A56DB e Dourado #E3AC00)
   - 🦅 **Aeternum** (Roxo #6B2FB7 e Prata #CFCFCF)

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

🐉 **Casa Draco**
- **Valores**: Coragem, criatividade, iniciativa, intensidade
- **Frase**: "Onde criamos, queimamos limites."
- **Descrição**: Casa dos líderes ousados, criativos e movidos a ação.

🐺 **Casa Lupus**
- **Valores**: Lealdade, disciplina, comunidade, foco
- **Frase**: "Sozinho você vai rápido. Em equipe você vai longe."
- **Descrição**: Casa dos disciplinados, colaborativos e unidos como uma alcateia.

🐯 **Casa Tigris**
- **Valores**: Ambição, excelência, foco, alta performance
- **Frase**: "Nada menos que o melhor."
- **Descrição**: Casa dos ambiciosos, performáticos e focados em excelência.

🦅 **Casa Aeternum**
- **Valores**: Liberdade, inovação, visão, autonomia
- **Frase**: "Enxerga além do código."
- **Descrição**: Casa dos visionários, criativos e livres.

## Installation

1. Clone this repository:
```bash
git clone <your-repo-url>
cd discord-bot
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
- `description`: Description shown when a user is sorted
- `traits`: Array of traits associated with this house

### Questions (`config.json`)

The `questions` array defines the sorting questions (in Portuguese). Each question has:
- `question`: The question text (in Portuguese)
- `options`: Array of 4 answer options

Each option has:
- `text`: The answer text shown to the user (in Portuguese)
- `traits`: Object mapping trait names to point values

The sorting algorithm calculates which house a user should join based on the total points accumulated for each trait. The current configuration includes 5 questions designed to identify personality traits aligned with each house's values.

## How It Works

1. When a new member joins the server, the bot sends them a DM in Portuguese
2. The bot asks a series of questions (defined in `config.json`)
3. The user responds by entering a number (1-4)
4. After all questions are answered, the bot:
   - Calculates trait scores based on the user's answers
   - Determines which house best matches those traits (Draco, Lupus, Tigris, or Aeternum)
   - Assigns the corresponding role to the user
   - Notifies them of their house assignment in Portuguese

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
