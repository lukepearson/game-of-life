# game-of-life
Conway's game of life using piston

## Prerequisites
* rust/cargo


## Run
```
cargo run
```

## Controls
| Key | Action |
| ----|:------:|
| space | togggle pause |
| r     | reset |
| c     | clear |
| -     | reduce speed |
| +     | increase speed |
| esc   | quit |

https://user-images.githubusercontent.com/2988301/127893810-a79c72cc-95df-474a-8685-38aa21da5951.mov

## Changelog
* Increased performance using ImageBuffer instead of drawing a rectangle for each cell