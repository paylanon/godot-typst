use godot::prelude::*;
use godot_typst::Typst;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}