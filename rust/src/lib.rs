use godot::prelude::*;

mod types;
mod crafting;
mod interior;
mod rules;
mod tech;
mod simulation;

struct EvofactoryExtension;

#[gdextension]
unsafe impl ExtensionLibrary for EvofactoryExtension {}
