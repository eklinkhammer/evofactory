# EvoFactory

A real-time cell biology simulation game built with **Godot 4** and **Rust** (GDExtension). You control a living cell navigating an infinite procedurally generated world, absorbing resources, building organelles, and evolving through a tech tree.

## Gameplay

You play as a single cell trying to survive and grow. The game alternates between two views:

- **World view** — navigate your cell through a dimetric-projected infinite world to find glucose, amino acids, and nucleotides
- **Interior view** (TAB) — manage organelles inside your cell: place zymases, motors, and mRNA strands

### Core systems

- **Resource absorption** — glucose, amino acids, and nucleotides are absorbed on contact and enter the cell interior as particles
- **Crafting** — zymases convert glucose into ATP; mRNA strands consume amino acids to produce new organelles (motors, zymases, membrane growth)
- **Gene regulation** (G) — set conditional rules that automatically suppress or enable mRNA production based on organelle counts, density, or surface density
- **Tech tree** (T) — unlock capabilities like chemoreceptors and a programmable nucleus by meeting biological milestones
- **Autonomous movement** (A) — toggle AI-driven chemotaxis (gradient-following) or Brownian random walk; requires the Chemoreceptor tech for sensor-based navigation

### Controls

| Key | Action |
|-----|--------|
| Arrow keys / WASD | Move cell |
| TAB | Toggle interior view |
| G | Gene regulation panel |
| T | Tech tree panel |
| A | Autonomy panel |
| R | Restart (when dead) |

## Building

Requires Godot 4.6+ and Rust nightly.

```bash
cd rust
cargo build
```

Then open the project in Godot and run.

## Architecture

- `rust/src/simulation.rs` — core simulation state and tick loop
- `rust/src/autonomy.rs` — chemotaxis and random walk logic
- `rust/src/crafting.rs` — organelle crafting and mRNA processing
- `rust/src/interior.rs` — particle physics, absorption, diffusion
- `rust/src/rules.rs` — gene regulation rule evaluation
- `rust/src/tech.rs` — tech tree progression
- `scripts/` — Godot GDScript for rendering, input, and UI panels

## Testing

```bash
cd rust
cargo test
```
