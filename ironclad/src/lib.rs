extern crate proc_macro;

mod game_value;
mod game_lifecycle;
mod game_entity;

use proc_macro::TokenStream;

/// Generates a bounded newtype with runtime validation.
/// Usage: #[game_value(min = 0, max = 99)]
#[proc_macro_attribute]
pub fn game_value(attr: TokenStream, item: TokenStream) -> TokenStream {
    game_value::expand(attr.into(), item.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Generates typestate transitions for lifecycle enums.
/// Usage: #[game_lifecycle(Seed -> Sprout -> Mature -> Harvested)]
#[proc_macro_attribute]
pub fn game_lifecycle(attr: TokenStream, item: TokenStream) -> TokenStream {
    game_lifecycle::expand(attr.into(), item.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

/// Generates a typed builder with compile-time required field checking.
/// Usage: #[game_entity(requires = [Name, Position, Sprite])]
#[proc_macro_attribute]
pub fn game_entity(attr: TokenStream, item: TokenStream) -> TokenStream {
    game_entity::expand(attr.into(), item.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
