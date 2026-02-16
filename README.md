# Bevy Maze Game

A grid-based maze game built with Bevy.

## Running the Game

```bash
cargo run
```

The game reads configuration from `config.toml` at startup. Adjust movement speed, rotation speed, input delay, and camera settings there as needed.

## Controls

### Movement & Rotation
| Key | Action |
|-----|--------|
| **W** | Move forward |
| **S** | Move backward |
| **A** | Move left |
| **D** | Move right |
| **Q** | Rotate left (90°) |
| **E** | Rotate right (90°) |

Movement is grid-based and relative to your facing direction. Hold a key to repeat movement every `input_repeat_delay` seconds (configurable in `config.toml`). The movement and rotation keys are configurable under the `[controls]` section in `config.toml`.

### Camera
| Key | Action |
|-----|--------|
| **TAB** | Hold to enable mouse look |
| **M** | Toggle camera look mode (relative ↔ absolute) |

**Look Modes:**
- **Relative**: Mouse movement controls camera look (default)
- **Absolute**: Cursor position controls where you look

Camera keys are configurable under the `[controls]` section in `config.toml`.
