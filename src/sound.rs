use std::error::Error as StdError;
use std::str::FromStr;
use std::time::Duration;
use std::{env, fmt};

use dotenv::dotenv;
use serenity::async_trait;
use serenity::builder::{CreateActionRow, CreateButton, CreateSelectMenu, CreateSelectMenuOption};
use serenity::client::{Context, EventHandler};
use serenity::futures::StreamExt;
use serenity::model::channel::Message;
use serenity::model::interactions::message_component::ButtonStyle;
use serenity::model::interactions::InteractionResponseType;
use serenity::prelude::*;

use crate::animal::{ParseComponentError, Animal};

#[derive(Debug)]
pub enum Sound {
    Meow,
    Woof,
    Neigh,
    Honk,
}

impl fmt::Display for Sound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Meow => write!(f, "meow"),
            Self::Woof => write!(f, "woof"),
            Self::Neigh => write!(f, "neigh"),
            Self::Honk => write!(f, "hoooooooonk"),
        }
    }
}

impl Sound {
    fn emoji(&self) -> char {
        match self {
            Self::Meow => Animal::Cat.emoji(),
            Self::Woof => Animal::Dog.emoji(),
            Self::Neigh => Animal::Horse.emoji(),
            Self::Honk => Animal::Alpaca.emoji(),
        }
    }

    fn button(&self) -> CreateButton {
        let mut b = CreateButton::default();
        b.custom_id(self.to_string().to_ascii_lowercase());
        b.emoji(self.emoji());
        b.label(self);
        b.style(ButtonStyle::Primary);
        b
    }

    pub fn action_row() -> CreateActionRow {
        let mut ar = CreateActionRow::default();
        // We can add up to 5 buttons per action row
        ar.add_button(Sound::Meow.button());
        ar.add_button(Sound::Woof.button());
        ar.add_button(Sound::Neigh.button());
        ar.add_button(Sound::Honk.button());
        ar
    }
}

impl FromStr for Sound {
    type Err = ParseComponentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "meow" => Ok(Sound::Meow),
            "woof" => Ok(Sound::Woof),
            "neigh" => Ok(Sound::Neigh),
            "hoooooooonk" => Ok(Sound::Honk),
            _ => Err(ParseComponentError(s.to_string())),
        }
    }
}
