# Bevy Maze Game

A grid-based maze game built with Bevy. 

Functionally deprecated - I wanted to play around with Rust and Bevy and I have acheived that goal, there is some advanced rendering I would like to do that Bevy and its implementation of wgpu don't support out the box. My goal isn't to dive deeply into shaders at this time so I am moving on to try a different engine.

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
