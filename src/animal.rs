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

#[derive(Debug)]
pub enum Animal {
    Cat,
    Dog,
    Horse,
    Alpaca,
}

#[derive(Debug)]
pub struct ParseComponentError(pub String);

impl fmt::Display for ParseComponentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to parse {} as component", self.0)
    }
}

impl StdError for ParseComponentError {}

impl fmt::Display for Animal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cat => write!(f, "Cat"),
            Self::Dog => write!(f, "Dog"),
            Self::Horse => write!(f, "Horse"),
            Self::Alpaca => write!(f, "Alpaca"),
        }
    }
}

impl FromStr for Animal {
    type Err = ParseComponentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cat" => Ok(Animal::Cat),
            "dog" => Ok(Animal::Dog),
            "horse" => Ok(Animal::Horse),
            "alpaca" => Ok(Animal::Alpaca),
            _ => Err(ParseComponentError(s.to_string())),
        }
    }
}

impl Animal {
    pub fn emoji(&self) -> char {
        match self {
            Self::Cat => '🐈',
            Self::Dog => '🐕',
            Self::Horse => '🐎',
            Self::Alpaca => '🦙',
        }
    }

    fn menu_option(&self) -> CreateSelectMenuOption {
        let mut opt = CreateSelectMenuOption::default();
        // This is what will be shown to the user
        opt.label(format!("{} {}", self.emoji(), self));
        // This is used to identify the selected value
        opt.value(self.to_string().to_ascii_lowercase());
        opt
    }

    fn select_menu() -> CreateSelectMenu {
        let mut menu = CreateSelectMenu::default();
        menu.custom_id("animal_select");
        menu.placeholder("No animal selected");
        menu.options(|f| {
            f.add_option(Self::Cat.menu_option())
                .add_option(Self::Dog.menu_option())
                .add_option(Self::Horse.menu_option())
                .add_option(Self::Alpaca.menu_option())
        });
        menu
    }

    pub fn action_row() -> CreateActionRow {
        let mut ar = CreateActionRow::default();
        // A select menu must be the only thing in an action row!
        ar.add_select_menu(Self::select_menu());
        ar
    }
}
