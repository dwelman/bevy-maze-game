# Bevy Maze Game

A grid-based maze game built with Bevy.

## Running the Game

```bash
cargo run
```

The game reads configuration from `config.toml` at startup. Adjust movement speed, rotation speed, and input delay there as needed.

## Controls

| Key | Action |
|-----|--------|
| **W** | Move forward |
| **S** | Move backward |
| **A** | Move left |
| **D** | Move right |
| **Q** | Rotate left (90°) |
| **E** | Rotate right (90°) |

Movement is grid-based and relative to your facing direction. Hold a key to repeat movement every `input_repeat_delay` seconds (configurable in `config.toml`).
