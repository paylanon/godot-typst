<p align="center">
<img src="https://github.com/paylhorse/godot-typst/assets/74363924/61433620-8126-46a4-8deb-39c7eac1c5f1" alt="logo" width="200"/>
</p>
<p align="center">
<b>Render Typst expressions in <a href="https://github.com/godotengine/godot">Godot 4</a></b>
</p>
<p align="center">
<b>Requires <a href="https://github.com/godot-rust/gdext">godot-rust</a></b>
</p>

## ABOUT
WORK IN PROGRESS

Inspired by [GodoTeX](https://github.com/file-acomplaint/GodoTeX)

## INSTALLATION
(1) Add this crate as a dependency to your godot-rust project. 

**Cargo.toml:**

```toml
[dependencies]
godot-typst = { git = "https://github.com/paylhorse/godot-typst" }
```
(2) Import the Typst class to automatically add it to Godot. Ignore warning.

**lib.rs:**

```rs
use godot_typst::Typst;
```

Locked and loaded!
