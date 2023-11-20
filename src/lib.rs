//  dP""b8  dP"Yb  8888b.   dP"Yb  888888          888888 Yb  dP 88""Yb .dP"Y8 888888 
// dP   `" dP   Yb  8I  Yb dP   Yb   88   ________   88    YbdP  88__dP `Ybo."   88   
// Yb  "88 Yb   dP  8I  dY Yb   dP   88   """"""""   88     8P   88"""  o.`Y8b   88   
//  YboodP  YbodP  8888Y"   YbodP    88              88    dP    88     8bodP'   88   
//
use std::fs::File;
use std::io::{Read, Write, Cursor};
use std::sync::{Arc, Mutex};
use std::thread;
use std::process::Command;
use godot::prelude::*;
use godot::engine::{Sprite2D, ISprite2D, Image, ImageTexture};
use usvg::TreeParsing;
use tempfile::tempdir;
use image::{DynamicImage, RgbaImage, ImageBuffer, ImageOutputFormat};

#[derive(GodotClass)]
#[class(base = Sprite2D, tool)]
pub struct Typst {
    #[base]
    pub node: Base<Sprite2D>,
    #[export(multiline)]
    pub typst_expression: GString,
    pub stored_expr: GString,
    pub shared_queue: Arc<Mutex<Vec<Vec<u8>>>>,
    pub time_accumulator: f32,
    pub is_job_active: bool,
}

#[godot_api]
impl ISprite2D for Typst {
    fn init(node: Base<Sprite2D>) -> Self {
        Typst { 
            node,
            typst_expression: String::new().into(),
            stored_expr: String::new().into(),
            shared_queue: Arc::new(Mutex::new(Vec::new())),
            time_accumulator: 0.0,
            is_job_active: false,
        }
    }

    fn ready(&mut self) {
        self.bake_png();
    }

    fn process(&mut self, delta: f64) {
        if self.typst_expression != self.stored_expr {
            self.start_job();
        }
        if self.is_job_active {
            self.time_accumulator += delta as f32;
            if self.time_accumulator >= 4.0 {
                self.bake_png();
                self.is_job_active = false;
            }
        }
        let mut queue = self.shared_queue.lock().unwrap();
        if let Some(png_buffer) = queue.pop() {
            godot_print!("Updating Typst node...");
            // Update node with new texture
            let mut typst_image = Image::new();
            typst_image.load_png_from_buffer(PackedByteArray::from(png_buffer.as_slice()));
            let typst_texture = ImageTexture::create_from_image(typst_image).expect("Failed to create ImageTexture!");
            // svg_texture.update();
            self.node.set_texture(typst_texture.upcast());
        }
    }
}

#[godot_api]
impl Typst {
    pub fn start_job(&mut self) {
        self.is_job_active = true;
        self.time_accumulator = 0.0;
        self.stored_expr = self.typst_expression.clone();
    }
    pub fn bake_png(&mut self) {
        // MULTI-THREAD
        let expression = self.typst_expression.clone().to_string();
        let queue_clone = Arc::clone(&self.shared_queue);
        thread::spawn(move || {
            // Create a temporary .typst file
            let dir = tempdir().expect("Failed to create temporary directory");
            let file_path = dir.path().join("expression.typst");
            let mut file = File::create(&file_path)
                .expect("Failed to create .typst file");
            writeln!(file, "{}", expression)
                .expect("Failed to write to .typst file");
            // Execute 'typst compile'
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
            // Read SVG content from temporary file
            let temp_svg_path = dir.path().join("expression.svg");
            let mut temp_svg_file = File::open(&temp_svg_path)
                .expect("Failed to open temporary SVG file");
            let mut svg_content = String::new();
            temp_svg_file.read_to_string(&mut svg_content)
                .expect("Failed to read SVG content");
            // -- RESVG --
            // Parse SVG
            let usvg_tree = usvg::Tree::from_str(&svg_content, &usvg::Options::default()).expect("Failed to parse SVG string!");
            let resvg_tree = resvg::Tree::from_usvg(&usvg_tree);
            // godot_print!("RESVG vb: {:?}", resvg_tree.view_box);
            // godot_print!("RESVG bb: {:?}", resvg_tree.content_area);
            // 595, 842
            let scale_factor = 2.0;
            let pw: u32 = 1190;
            let ph: u32 = 1684;
            // Create mutable pixmap buffer
            let mut pixmap_data = vec![0; pw as usize * ph as usize * 4]; // 4 bytes per pixel (RGBA)
            let mut pixmap = tiny_skia::PixmapMut::from_bytes(&mut pixmap_data, pw, ph)
                .expect("Failed to create pixmap");
            // Render SVG onto pixmap
            resvg_tree.render(tiny_skia::Transform::from_scale(scale_factor, scale_factor), &mut pixmap);
            // Convert to a PNG buffer
            let convert_rgba_to_png = |pixmap_data: &[u8], pw: u32, ph: u32| -> Vec<u8> {
                let img: RgbaImage = ImageBuffer::from_raw(pw, ph, pixmap_data.to_vec()).unwrap();
                let dynamic_img = DynamicImage::ImageRgba8(img);
                // Crop png to content area
                let cropped_img = if let Some(content_area) = resvg_tree.content_area {
                    let cx = (content_area.x() * scale_factor) as u32;
                    let cy = (content_area.y() * scale_factor) as u32; 
                    let cw = (content_area.width() * scale_factor) as u32;
                    let ch = (content_area.height() * scale_factor) as u32;
                    dynamic_img.crop_imm(cx, cy, cw, ch)
                } else {
                    dynamic_img
                };
                let mut buffer = Cursor::new(Vec::new());
                cropped_img.write_to(&mut buffer, ImageOutputFormat::Png).unwrap();
                buffer.into_inner()
            };
            let png_buffer = convert_rgba_to_png(&pixmap_data, pw, ph);
            let mut queue = queue_clone.lock().unwrap();
            queue.push(png_buffer);
        });
    }
}
