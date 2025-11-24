//! Discord Sorting Hat Bot
//!
//! A Discord bot that sorts new members into houses through an interactive quiz.
//! When new members join the server, they receive a DM with questions in Portuguese
//! and are assigned to one of four houses based on their answers.

use serenity::{
    async_trait,
    model::{gateway::Ready, guild::Member, id::GuildId, prelude::*},
    prelude::*,
};
use std::{collections::HashMap, env, fs, sync::Arc};
use tokio::sync::RwLock;

mod config;
mod sorting;

use config::{Config, House};
use sorting::SortingSession;

/// Main event handler for the Discord bot.
///
/// Manages sorting sessions and handles Discord events such as member joins,
/// member departures, and incoming messages.
struct Handler {
    /// Shared bot configuration loaded from config.json.
    config: Arc<Config>,
    /// Map of active sorting sessions indexed by user ID.
    sorting_sessions: Arc<RwLock<HashMap<UserId, SortingSession>>>,
    /// The Discord guild (server) ID where the bot operates.
    guild_id: GuildId,
}

#[async_trait]
impl EventHandler for Handler {
    /// Called when the bot successfully connects to Discord.
    ///
    /// # Arguments
    ///
    /// * `_` - The Discord context (unused).
    /// * `ready` - Information about the connected bot user.
    async fn ready(&self, _: Context, ready: Ready) {
        tracing::info!("{} is connected!", ready.user.name);
    }

    /// Called when a member leaves the guild.
    ///
    /// Cleans up any active sorting session for the departing user to prevent
    /// stale sessions from accumulating in memory.
    ///
    /// # Arguments
    ///
    /// * `_ctx` - The Discord context (unused).
    /// * `_guild_id` - The guild ID (unused).
    /// * `user` - The user who left.
    /// * `_member` - Optional member data if cached (unused).
    async fn guild_member_removal(
        &self,
        _ctx: Context,
        _guild_id: GuildId,
        user: User,
        _member: Option<Member>,
    ) {
        // Clean up any active sorting session when a user leaves
        let mut sessions = self.sorting_sessions.write().await;
        if sessions.remove(&user.id).is_some() {
            tracing::info!(
                "Cleaned up sorting session for {} who left the server",
                user.name
            );
        }
    }

    /// Called when a new member joins the guild.
    ///
    /// Initiates the sorting process by sending a welcome message via DM
    /// and starting the quiz with the first question.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The Discord context for API calls.
    /// * `new_member` - The member who just joined.
    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {
        tracing::info!("New member joined: {}", new_member.user.name);

        let user_id = new_member.user.id;

        // Try to create DM channel first, before creating session
        let dm_channel = match new_member.user.create_dm_channel(&ctx.http).await {
            Ok(channel) => channel,
            Err(e) => {
                tracing::error!(
                    "Failed to create DM channel for {}: {:?} - User may have DMs disabled",
                    new_member.user.name,
                    e
                );
                return;
            }
        };

        // Welcome message in Portuguese
        let welcome_msg = format!(
            "Bem-vindo ao servidor, {}! 🎩✨\n\n\
            Antes de acessar o servidor completo, você precisa ser selecionado para sua casa.\n\
            Vou te fazer algumas perguntas e, com base nas suas respostas, você será designado para uma das nossas quatro casas:\n\n\
            🐉 **Draco** - Líderes ousados e criativos\n\
            🐺 **Lupus** - Disciplinados e colaborativos\n\
            🐯 **Tigris** - Ambiciosos e performáticos\n\
            🦅 **Aeternum** - Visionários e livres\n\n\
            Vamos começar!",
            new_member.user.name
        );

        // Only create session if we can send the welcome message
        if let Err(e) = dm_channel.say(&ctx.http, &welcome_msg).await {
            tracing::error!(
                "Failed to send welcome message to {}: {:?} - User may have DMs disabled",
                new_member.user.name,
                e
            );
            return;
        }

        // Now create and store the session (DM was successful)
        let session = SortingSession::new(self.config.clone());
        {
            let mut sessions = self.sorting_sessions.write().await;
            sessions.insert(user_id, session);
        }

        // Send first question
        self.send_question(&ctx, &dm_channel, user_id).await;
    }

    /// Handles incoming messages from users.
    ///
    /// Processes both guild commands (!testsort, !resetsort) and DM responses
    /// to sorting quiz questions.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The Discord context for API calls.
    /// * `msg` - The received message.
    async fn message(&self, ctx: Context, msg: Message) {
        // Ignore bot messages
        if msg.author.bot {
            return;
        }

        // Handle guild messages (commands)
        if msg.guild_id.is_some() {
            match msg.content.as_str() {
                "!testsort" => {
                    // Check if user already has an active session
                    {
                        let sessions = self.sorting_sessions.read().await;
                        if sessions.contains_key(&msg.author.id) {
                            let _ = msg
                                .channel_id
                                .say(
                                    &ctx.http,
                                    "Você já tem uma sessão de seleção ativa! Responda às perguntas na DM ou use `!resetsort` para recomeçar.",
                                )
                                .await;
                            return;
                        }
                    }

                    tracing::info!("Test sorting triggered by: {}", msg.author.name);
                    if let Ok(member) = self.guild_id.member(&ctx.http, msg.author.id).await {
                        self.guild_member_addition(ctx, member).await;
                    }
                }
                "!resetsort" => {
                    // Allow user to reset their sorting session
                    let mut sessions = self.sorting_sessions.write().await;
                    if sessions.remove(&msg.author.id).is_some() {
                        tracing::info!("Sorting session reset by: {}", msg.author.name);
                        let _ = msg
                            .channel_id
                            .say(&ctx.http, "Sua sessão foi resetada. Use `!testsort` para começar novamente.")
                            .await;
                    } else {
                        let _ = msg
                            .channel_id
                            .say(&ctx.http, "Você não tem uma sessão ativa para resetar.")
                            .await;
                    }
                }
                _ => {}
            }
            return;
        }

        let user_id = msg.author.id;

        // Check if user has an active sorting session
        let mut sessions = self.sorting_sessions.write().await;
        if let Some(session) = sessions.get_mut(&user_id) {
            // Check if session has expired
            if session.is_expired() {
                sessions.remove(&user_id);
                tracing::info!("Expired sorting session removed for {}", msg.author.name);
                let _ = msg
                    .channel_id
                    .say(
                        &ctx.http,
                        "Sua sessão de seleção expirou. Use `!testsort` no servidor para começar novamente.",
                    )
                    .await;
                return;
            }

            // Try to parse the answer
            if let Ok(choice) = msg.content.trim().parse::<usize>() {
                if choice > 0 && choice <= 4 {
                    // Record answer
                    session.record_answer(choice - 1);

                    // Check if we have more questions
                    if session.current_question < self.config.questions.len() {
                        // Send next question
                        drop(sessions); // Release lock before async call
                        if let Ok(dm_channel) = msg.author.create_dm_channel(&ctx.http).await {
                            self.send_question(&ctx, &dm_channel, user_id).await;
                        }
                    } else {
                        // Sorting complete, assign house
                        let house = session.determine_house(&self.config);
                        drop(sessions); // Release lock before async calls

                        self.assign_house(&ctx, user_id, &house).await;

                        // Remove session
                        let mut sessions = self.sorting_sessions.write().await;
                        sessions.remove(&user_id);
                    }
                    return;
                }
            }

            // Invalid input - prompt user to enter valid number (in Portuguese)
            if let Err(e) = msg
                .channel_id
                .say(&ctx.http, "Por favor, digite um número entre 1 e 4.")
                .await
            {
                tracing::error!("Failed to send message: {:?}", e);
            }
        }
    }
}

impl Handler {
    /// Sends the current question to a user via DM.
    ///
    /// Retrieves the current question from the user's session and formats it
    /// with numbered options for easy response.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The Discord context for API calls.
    /// * `dm_channel` - The user's DM channel.
    /// * `user_id` - The ID of the user to send the question to.
    async fn send_question(
        &self,
        ctx: &Context,
        dm_channel: &serenity::model::channel::PrivateChannel,
        user_id: UserId,
    ) {
        let sessions = self.sorting_sessions.read().await;
        if let Some(session) = sessions.get(&user_id) {
            let question_idx = session.current_question;
            if question_idx < self.config.questions.len() {
                let question = &self.config.questions[question_idx];

                // Format question with options (questions are in Portuguese)
                let mut msg = format!(
                    "\n**Pergunta {}:**\n{}\n\n",
                    question_idx + 1,
                    question.question
                );
                for (i, option) in question.options.iter().enumerate() {
                    msg.push_str(&format!("{}. {}\n", i + 1, option.text));
                }
                msg.push_str("\nResponda com o número da sua escolha (1-4).");

                if let Err(e) = dm_channel.say(&ctx.http, &msg).await {
                    tracing::error!("Failed to send question: {:?}", e);
                }
            }
        }
    }

    /// Assigns a house role to a user after sorting is complete.
    ///
    /// Sends a congratulatory message to the user via DM and assigns the
    /// corresponding Discord role in the guild.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The Discord context for API calls.
    /// * `user_id` - The ID of the user to assign the house to.
    /// * `house` - The house the user was sorted into.
    async fn assign_house(&self, ctx: &Context, user_id: UserId, house: &House) {
        tracing::info!("Assigning {} to house: {}", user_id, house.name);

        // Send result to user via DM (in Portuguese)
        if let Ok(dm_channel) = user_id.create_dm_channel(&ctx.http).await {
            let result_msg = format!(
                "🎉 **A Seleção está completa!**\n\n\
                Você foi selecionado para a casa... **{}**!\n\n\
                {}\n\n\
                Agora você tem acesso aos canais da sua casa e ao servidor principal. Bem-vindo!",
                house.name, house.description
            );

            if let Err(e) = dm_channel.say(&ctx.http, &result_msg).await {
                tracing::error!("Failed to send result message: {:?}", e);
            }
        }

        // Assign role in guild
        if let Ok(member) = self.guild_id.member(&ctx.http, user_id).await {
            // Find the role by name
            if let Ok(guild) = self.guild_id.to_partial_guild(&ctx.http).await {
                if let Some(role) = guild.roles.values().find(|r| r.name == house.role_name) {
                    if let Err(e) = member.add_role(&ctx.http, role.id).await {
                        tracing::error!("Failed to assign role: {:?}", e);
                    } else {
                        tracing::info!("Successfully assigned {} role to {}", house.name, user_id);
                    }
                } else {
                    tracing::error!("Role '{}' not found in guild", house.role_name);
                }
            }
        }
    }
}

/// Application entry point.
///
/// Initializes logging, loads configuration, sets up the Discord client,
/// and starts the bot.
#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Get Discord token from environment
    let token = env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in environment");

    // Get guild ID from environment and parse it
    let guild_id: u64 = env::var("GUILD_ID")
        .expect("Expected GUILD_ID")
        .parse()
        .expect("GUILD_ID must be a valid u64");

    // Load configuration from config.json
    let config_str = fs::read_to_string("config.json").expect("Failed to read config.json");
    let config: Config = serde_json::from_str(&config_str).expect("Failed to parse config.json");
    let config = Arc::new(config);

    // Set up required gateway intents for the bot
    let intents = GatewayIntents::GUILD_MEMBERS
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES  // Required to receive messages in channels
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create the event handler with configuration
    let handler = Handler {
        config,
        sorting_sessions: Arc::new(RwLock::new(HashMap::new())),
        guild_id: GuildId::new(guild_id),
    };

    // Create and configure the Discord client
    let mut client = Client::builder(&token, intents)
        .event_handler(handler)
        .await
        .expect("Error creating client");

    // Start the bot
    tracing::info!("Starting bot...");
    if let Err(why) = client.start().await {
        tracing::error!("Client error: {:?}", why);
    }
}
