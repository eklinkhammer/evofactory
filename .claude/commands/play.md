Kill any running Godot instance, then build the Rust GDExtension crate and launch the game.

Steps:
1. Kill any existing Godot process (`pkill -f godot`)
2. Build the Rust crate (`cd rust && cargo build`)
3. Launch the game (`godot &`)
