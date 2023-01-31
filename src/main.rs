// The version of ngrok ping-pong-bot, which uses a webhook to receive updates
// from Telegram, instead of long polling.
mod questions;

use std::fmt::Display;
use teloxide::types::Recipient;
use teloxide::{prelude::*, update_listeners::webhooks};
use teloxide::{dispatching::dialogue::InMemStorage,};
use teloxide::error_handlers::IgnoringErrorHandler;
use questions::*;

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
    let url = "https://b57e-89-33-43-65.eu.ngrok.io".parse().unwrap();
    let listener = webhooks::axum(bot.clone(), webhooks::Options::new(addr, url))
        .await
        .expect("Couldn't setup webhook");

        //ensure current user 

        Dispatcher::builder(
            bot,
            Update::filter_message()
                .enter_dialogue::<Message, InMemStorage<State>, State>()
                .branch(dptree::case![State::Start].endpoint(questions::start))
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


