//  dP""b8  dP"Yb  8888b.   dP"Yb  888888          888888 Yb  dP 88""Yb .dP"Y8 888888 
// dP   `" dP   Yb  8I  Yb dP   Yb   88   ________   88    YbdP  88__dP `Ybo."   88   
// Yb  "88 Yb   dP  8I  dY Yb   dP   88   """"""""   88     8P   88"""  o.`Y8b   88   
//  YboodP  YbodP  8888Y"   YbodP    88              88    dP    88     8bodP'   88   
//
use std::fs::File;
use std::io::{Read, Write};
use std::process::Command;
use godot::prelude::*;
use godot::engine::{Sprite2D, ISprite2D, Texture2D};
use godot::engine::FileAccess;
use godot::engine::file_access::ModeFlags;
use tempfile::tempdir;

#[derive(GodotClass)]
#[class(base = Sprite2D)]
pub struct Typst {
    #[base]
    pub node: Base<Sprite2D>,
    #[export]
    pub typst_expression: GString,
}

#[godot_api]
impl ISprite2D for Typst {
    fn init(node: Base<Sprite2D>) -> Self {
        Typst { 
            node,
            typst_expression: String::new().into(),
        }
    }

    fn ready(&mut self) {
        self.render();
    }

    fn process(&mut self, delta: f64) {
        // Periodically re-render
    }
}

#[godot_api]
impl Typst {
    pub fn render(&mut self) {
        // Render Typst: convert latex expression to SVG, then assign to self
        // Step 1: Create a temporary .typst file
        let dir = tempdir().expect("Failed to create temporary directory");
        let file_path = dir.path().join("expression.typst");
        let mut file = File::create(&file_path)
            .expect("Failed to create .typst file");
        writeln!(file, "{}", self.typst_expression)
            .expect("Failed to write to .typst file");
        // Step 2: Execute 'typst compile'
        // let output_path = dir.path().join("output.svg");
        let status = Command::new("typst")
            .arg("compile")
            .arg(&file_path)
            .arg("--format")
            .arg("svg")
            .status()
            .expect("Failed to execute typst command");
        if !status.success() {
            eprintln!("Error: Typst command failed");
            return;
        }
        // Read the SVG content from the temporary file
        let temp_svg_path = dir.path().join("expression.svg");
        let mut temp_svg_file = File::open(&temp_svg_path)
            .expect("Failed to open temporary SVG file");
        let mut svg_content = String::new();
        temp_svg_file.read_to_string(&mut svg_content)
            .expect("Failed to read SVG content");
        // Path to store the SVG in Godot's resource path
        let godot_res_path = GString::from("res://output.svg");
        // Open the file in write mode
        if let Some(mut file) = FileAccess::open(godot_res_path, ModeFlags::WRITE) {
            // Write the SVG content
            file.store_string(GString::from(svg_content));
            file.flush();
            file.close();
            godot_print!("SVG ready!");
        } else {
            godot_error!("Failed to open file in Godot resource path");
        }
        let svg_texture = load::<Texture2D>("res://output.svg");
        self.node.set_texture(svg_texture);
    }
}
