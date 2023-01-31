// The version of ngrok ping-pong-bot, which uses a webhook to receive updates
// from Telegram, instead of long polling.

use std::collections::hash_map::DefaultHasher;
use std::fmt::Display;
use std::hash::{Hash, Hasher};

use teloxide::{prelude::*, update_listeners::webhooks};
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};
use teloxide::error_handlers::IgnoringErrorHandler;
use teloxide::types::{UserId, Recipient};

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone)]
pub enum PersonType {
    BusinessDev,
    Founder, 
    Builder,
    Investor,
    Marketing,
    Other
}

impl Display for PersonType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            PersonType::BusinessDev => "Business Development",
            PersonType::Founder => "Founder",
            PersonType::Builder => "Builder",
            PersonType::Investor => "Investor",
            PersonType::Marketing => "Marketing",
            PersonType::Other => "Other",
        };
        write!(f, "{}", s)
    }
}

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    ReceiveFullName,
    WhoWith {full_name: String},
    Location {full_name: String, referee: Recipient},
    TypeOfPersonList {full_name: String, referee: Recipient, location: String},
    WhatAreYouBuilding {full_name: String, referee: Recipient, location: String, person: PersonType},
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting ngrok ping-pong bot...");

    let bot = Bot::from_env();

    let addr = ([127, 0, 0, 1], 8443).into();
    let url = "https://bacc-80-28-254-88.eu.ngrok.io".parse().unwrap();
    let listener = webhooks::axum(bot.clone(), webhooks::Options::new(addr, url))
        .await
        .expect("Couldn't setup webhook");

        Dispatcher::builder(
            bot,
            Update::filter_message()
                .enter_dialogue::<Message, InMemStorage<State>, State>()
                .branch(dptree::case![State::Start].endpoint(start))
                .branch(dptree::case![State::ReceiveFullName].endpoint(ask_full_name))
                .branch(dptree::case![State::WhoWith { full_name }].endpoint(ask_who_with))
                .branch(dptree::case![State::Location { full_name, referee }].endpoint(ask_location))
                //.branch(dptree::case![State::TypeOfPersonList { full_name, referee, location }].endpoint(ask_type))
                //.branch(dptree::case![State::WhatAreYouBuilding { full_name, referee, location, person } ].endpoint(ask_what))
        )
        .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch_with_listener(listener, IgnoringErrorHandler::new())
        .await;
}


async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Hello! And with whom do I have the pleasure of interfacing with today?").await?;
    dialogue.update(State::ReceiveFullName).await?;
    Ok(())
}

async fn ask_full_name(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            bot.send_message(msg.chat.id, format!("Hello {}", text)).await?;
            bot.send_message(msg.chat.id, format!("And with whom are you chatting with? (@alexanderfive, (the only one that works atm)) ")).await?;

            dialogue.update(State::WhoWith { full_name: String::from(text) }).await?;

        }
        None => {
            bot.send_message(msg.chat.id, "Send me plain text.").await?;
        }
    }
    Ok(())
}

async fn ask_who_with(bot: Bot, dialogue: MyDialogue, full_name: String, msg: Message) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            let mut hasher = DefaultHasher::new();
            text.hash(&mut hasher);
        loop {
            //todo secure usernames.
            if valid_usernames().contains(&hasher.finish()) {
                // invite new user
                bot.send_message(msg.chat.id, format!("{} is a valid username", text)).await?;
                let referee = <Recipient as From<String>>::from(text.to_string());
                bot.create_chat_invite_link(referee.clone()).await?;
                dialogue.update(State::Location {full_name, referee}).await?;
                bot.send_message(msg.chat.id, "I have invited this user to the group, please make sure they join as, for not, your responses will not be saved otherwise.").await?;
                bot.send_message(msg.chat.id, "where are you based").await?;

                break
            } else {
                bot.send_message(msg.chat.id, format!("{} is not a valid username, please try again", text)).await?;
            }
        }

        }
        None => {
            bot.send_message(msg.chat.id, "Send me plain text.").await?;
        }
    }

    Ok(())
}

async fn ask_location(bot: Bot, dialogue: MyDialogue, full_name: String, msg: Message) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            let mut hasher = DefaultHasher::new();
            text.hash(&mut hasher);
        }
        None => {
            bot.send_message(msg.chat.id, "Send me plain text.").await?;
        }
    }

    Ok(())
}

fn valid_usernames() -> Vec<u64> {
    let mut hasher = DefaultHasher::new();
    "@alexanderfive".hash(&mut hasher);
    vec![hasher.finish()]
}
