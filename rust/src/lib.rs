use godot::prelude::*;

mod types;
mod crafting;
mod interior;
mod rules;
mod tech;
mod sync;
mod autonomy;
mod simulation;

struct EvofactoryExtension;

#[gdextension]
unsafe impl ExtensionLibrary for EvofactoryExtension {}
