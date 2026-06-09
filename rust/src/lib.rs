use godot::prelude::*;

mod types;
mod crafting;
mod interior;
mod simulation;

struct EvofactoryExtension;

#[gdextension]
unsafe impl ExtensionLibrary for EvofactoryExtension {}
