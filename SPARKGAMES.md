# SparkGames' Bevy

The one Bevy fork shared by every SparkGames game. When Bevy itself must change, the change is made here, once, at its cause, rather than worked around in a game.

## Branches

`sparkgames/0.20.1` is Bevy `v0.20.1` plus one commit per fix, each with tests. A game pins a commit of it:

```toml
[patch.crates-io]
bevy = { git = "https://github.com/SparkGamesGS/bevy", rev = "<commit>" }
# ...and every other bevy_* crate the game's dependency tree uses, with the same git and rev.
```

## Rules for every change

1. **It knows no game.** No game name, type, asset or string.
2. **It has no opinion about a game's data.** It fixes or extends Bevy for every game.
3. **Open for extension, closed for modification.** Games extend Bevy from outside, with their own plugins, materials and systems. The fork changes Bevy only where Bevy offers no way to do that, and the change adds that way (a setting or a hook).

Each fix can be offered upstream as a pull request.
