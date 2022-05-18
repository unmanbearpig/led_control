mod config;
mod action_spec;
mod actions;
mod web;

use std::env;

#[macro_use]
extern crate rust_embed;

fn main() -> Result<(), String> {
    let (action, config) = config::from_args(env::args())?;
    if let Some(action) = &action {
        action.run(&config.mux)?;
    }

    Ok(())
}
