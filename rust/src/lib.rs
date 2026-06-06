use godot::prelude::*;

mod simulation;

struct EvofactoryExtension;

#[gdextension]
unsafe impl ExtensionLibrary for EvofactoryExtension {}
