# ITank

ITank is a terminal Tank90-style game written in Rust. It uses a 39×20 grid, waves of enemy tanks, destructible brick walls, steel barriers, and water hazards.

## Features

- Player tank with keyboard movement and firing
- Enemy tanks that pursue the player and base
- Five waves with increasing difficulty
- Brick, steel, and water terrain
- Pause, score, lives, and wave tracking
- Terminal rendering via `crossterm`

## Run

```bash
cargo run --release
```

## Controls

| Action | Key |
|---|---|
| Move | `W`, `A`, `S`, `D` or arrow keys |
| Fire | `Space` |
| Pause | `P` |
| Quit | `Q` or `Esc` |

## Build and Test

```bash
cargo build --release
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## Gameplay

Protect the base marked `B`, destroy enemy tanks marked `X`, and survive five waves. Bricks can be destroyed by bullets, steel blocks bullets, and tanks cannot enter water.
