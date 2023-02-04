use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::{HandlerResult, MyDialogue, OtterCommand, State};
use teloxide::payloads::SendMessageSetters;
use teloxide::prelude::Handler;
use teloxide::requests::Requester;
use teloxide::types::{
    ChatId, ChatMember, CountryCode, InlineKeyboardButton, InlineKeyboardMarkup, InlineQuery,
    InlineQueryResultArticle, InputMessageContent, InputMessageContentText, Message, Recipient,
    UserId,
};
use teloxide::utils::command::BotCommands;
use teloxide::Bot;

pub struct Questions;

impl Questions {
    pub async fn wrong_permissions(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
        match msg.text() {
            Some(text) => {
                dialogue.update(State::Start).await?;
            }
            None => {
                bot.send_message(msg.chat.id, "Send me plain text.").await?;
            }
        }

        dialogue.update(State::Start).await?;
        Ok(())
    }

    pub async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
        // get admins and check that the bot is admin.
        let admins: Vec<ChatMember> = bot.get_chat_administrators(msg.chat.id).await?;
        let me = bot.get_me().await?;
        let bot_name: &str = me.username();

        let bot_is_admin = admins
            .iter()
            .any(|a| a.user.is_bot && a.user.username.as_ref().is_some_and(|n| n == bot_name));
        if !ChatId::is_group(msg.chat.id) {
            bot.send_message(
                msg.chat.id,
                "Please add me to a group chat to function correctly.",
            )
            .await?;
        }
        if !bot_is_admin {
            bot.send_message(msg.chat.id, "I must be admin to function correctly; \n Please reply when youve updated my permissions").await?;
        }

        dialogue.update(State::ReceiveFullName).await?;
        bot.send_message(
            msg.chat.id,
            "Hello! And with whom do I have the pleasure of interfacing with today?",
        )
        .await?;
        //TODO: Check to see if the bot is in a group chat.

        Ok(())
    }

    pub async fn ask_full_name(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
        match msg.text() {
            Some(text) => {
                bot.send_message(msg.chat.id, format!("Hello {}", text))
                    .await?;
                bot.send_message(msg.chat.id, format!("And with whom are you chatting with? (@Felixgate, (the only one that works atm)) ")).await?;

                dialogue
                    .update(State::WhoWith {
                        full_name: String::from(text),
                    })
                    .await?;
            }
            None => {
                bot.send_message(msg.chat.id, "Send me plain text.").await?;
            }
        }
        Ok(())
    }

    pub async fn ask_who_with(
        bot: Bot,
        dialogue: MyDialogue,
        full_name: String,
        msg: Message,
    ) -> HandlerResult {
        match msg.text() {
            Some(text) => {
                let mut hasher = DefaultHasher::new();
                text.hash(&mut hasher);
                //todo secure usernames.
                if Self::valid_usernames().contains(&hasher.finish()) {
                    // invite new user
                    bot.send_message(msg.chat.id, format!("{} is a valid username", text))
                        .await?;
                    let referee = <Recipient as From<String>>::from(text.to_string());

                    // CREAET CHAT MAY BE A PROBLEM.
                    let chat_link = bot.create_chat_invite_link(msg.chat.id).await?;
                    println!("{:?}", chat_link);
                    ////
                    //bot.send_message(msg.chat.id, "I have invited this user to the group, please make sure they join as, for not, your responses will not be saved otherwise.").await?;
                    bot.send_message(msg.chat.id, "where are you based").await?;
                    dialogue
                        .update(State::Location { full_name, referee })
                        .await?;
                } else {
                    bot.send_message(
                        msg.chat.id,
                        format!("{} is not a valid username, please try again", text),
                    )
                    .await?;
                    dialogue.update(State::WhoWith { full_name }).await?;
                }
            }
            None => {
                bot.send_message(msg.chat.id, "Send me plain text.").await?;
            }
        }

        Ok(())
    }

    pub async fn ask_location(bot: Bot, msg: Message, q: InlineQuery) -> HandlerResult {
        match msg.text() {
            Some(text) => {
                let choose_location = InlineQueryResultArticle::new(
                    "0",
                    "Chose debian version",
                    InputMessageContent::Text(InputMessageContentText::new("Countries:")),
                )
                .reply_markup(Self::make_keyboard());
                bot.answer_inline_query(q.id, vec![choose_location.into()])
                    .await?;
            }
            None => {
                bot.send_message(msg.chat.id, "Send me plain text.").await?;
            }
        }

        Ok(())
    }

    pub fn valid_usernames() -> Vec<u64> {
        let mut hasher = DefaultHasher::new();
        "@Felixgate".hash(&mut hasher);
        vec![hasher.finish()]
    }

    /// Creates a keyboard made by buttons in a big column.
    pub fn make_keyboard() -> InlineKeyboardMarkup {
        let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];
        let num_variants =
            std::mem::size_of_val(&CountryCode::AD) / std::mem::size_of::<CountryCode>();

        let _: _ = vec!["GB", "FR", "BG", "GE", "NZ", "AU", "US"]
            .chunks(3)
            .map(|countries| {
                let row = countries
                    .iter()
                    .map(|&version| {
                        InlineKeyboardButton::callback(version.to_owned(), version.to_owned())
                    })
                    .collect();
                keyboard.push(row);
            })
            .collect::<Vec<_>>();

        InlineKeyboardMarkup::new(keyboard)
    }

    pub async fn handle_command(bot: Bot, msg: Message, cmd: OtterCommand) -> HandlerResult {
        match cmd {
            OtterCommand::Help => {
                bot.send_message(msg.chat.id, OtterCommand::descriptions().to_string())
                    .await?
            }
            OtterCommand::Start => {
                bot.send_message(msg.chat.id, OtterCommand::descriptions().to_string())
                    .await?
            }
        };

        Ok(())
    }
}
