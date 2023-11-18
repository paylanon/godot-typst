//  dP""b8  dP"Yb  8888b.   dP"Yb  888888          888888 Yb  dP 88""Yb .dP"Y8 888888 
// dP   `" dP   Yb  8I  Yb dP   Yb   88   ________   88    YbdP  88__dP `Ybo."   88   
// Yb  "88 Yb   dP  8I  dY Yb   dP   88   """"""""   88     8P   88"""  o.`Y8b   88   
//  YboodP  YbodP  8888Y"   YbodP    88              88    dP    88     8bodP'   88   
//
use std::fs::File;
use std::io::{Read, Write, Cursor};
use std::process::Command;
use godot::prelude::*;
use godot::engine::{Sprite2D, ISprite2D, Image, ImageTexture};
use usvg::TreeParsing;
use tempfile::tempdir;
use image::{RgbaImage, ImageBuffer, ImageOutputFormat};

#[derive(GodotClass)]
#[class(base = Sprite2D, tool)]
pub struct Typst {
    #[base]
    pub node: Base<Sprite2D>,
    #[export(multiline)]
    pub typst_expression: GString,
    pub time_accumulator: f32,
    pub stored_expr: GString,
}

#[godot_api]
impl ISprite2D for Typst {
    fn init(node: Base<Sprite2D>) -> Self {
        Typst { 
            node,
            typst_expression: String::new().into(),
            time_accumulator: 0.0,
            stored_expr: String::new().into(),
        }
    }

    fn ready(&mut self) {
        self.render();
    }

    fn process(&mut self, delta: f64) {
        // Periodically re-render
        // self.time_accumulator += delta as f32;
        // if self.time_accumulator >= 300.0 {
        //     self.render();
        //     self.time_accumulator = 0.0;
        // }
        // Instant re-render
        self.render();
    }
}

#[godot_api]
impl Typst {
    pub fn render(&mut self) {
        if self.typst_expression != self.stored_expr {
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
            // -- RESVG --
            // Parse the SVG
            let usvg_tree = usvg::Tree::from_str(&svg_content, &usvg::Options::default()).expect("Failed to parse SVG string!");
            let resvg_tree = resvg::Tree::from_usvg(&usvg_tree);
            godot_print!("RESVG vb: {:?}", resvg_tree.view_box);
            godot_print!("RESVG bb: {:?}", resvg_tree.content_area);
            // 595, 842
            let scale_factor = 1.0;
            let pw: u32 = 595;
            let ph: u32 = 842;
            // Create a mutable pixmap buffer
            let mut pixmap_data = vec![0; pw as usize * ph as usize * 4]; // 4 bytes per pixel (RGBA)
            let mut pixmap = tiny_skia::PixmapMut::from_bytes(&mut pixmap_data, pw, ph)
                .expect("Failed to create pixmap");
            // Render the SVG onto the pixmap
            resvg_tree.render(tiny_skia::Transform::from_scale(scale_factor, scale_factor), &mut pixmap);
            // Now `pixmap_data` contains your rendered image
            // Convert this data to a PNG buffer
            let png_buffer = self.convert_rgba_to_png(&pixmap_data, pw, ph);
            // Feed to Godot
            let mut typst_image = Image::new();
            typst_image.load_png_from_buffer(PackedByteArray::from(png_buffer.as_slice()));
            let typst_texture = ImageTexture::create_from_image(typst_image).expect("Failed to create ImageTexture!");
            // svg_texture.update();
            self.node.set_texture(typst_texture.upcast());
            self.stored_expr = self.typst_expression.clone();
        }
    }

    fn convert_rgba_to_png(&self, data: &[u8], width: u32, height: u32) -> Vec<u8> {
        let img: RgbaImage = ImageBuffer::from_raw(width, height, data.to_vec()).unwrap();
        let mut buffer = Cursor::new(Vec::new());
        img.write_to(&mut buffer, ImageOutputFormat::Png).unwrap();
        buffer.into_inner()
    }
}
