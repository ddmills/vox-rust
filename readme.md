### experiments with bevy and rust

https://github.com/tbillington/bevy_best_practices
https://bevyengine.org/learn/quick-start/getting-started/setup/

```toml
# Enable a small amount of optimization in debug mode
[profile.dev]
opt-level = 1

# Enable high optimizations for dependencies (incl. Bevy), but not for our code:
[profile.dev.package."*"]
opt-level = 3
```
