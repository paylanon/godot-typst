//  dP""b8  dP"Yb  8888b.   dP"Yb  888888          888888 Yb  dP 88""Yb .dP"Y8 888888 
// dP   `" dP   Yb  8I  Yb dP   Yb   88   ________   88    YbdP  88__dP `Ybo."   88   
// Yb  "88 Yb   dP  8I  dY Yb   dP   88   """"""""   88     8P   88"""  o.`Y8b   88   
//  YboodP  YbodP  8888Y"   YbodP    88              88    dP    88     8bodP'   88   
//
use godot::prelude::*;
use godot::engine::{Sprite2D, ISprite2D};

#[derive(GodotClass)]
#[class(base = Sprite2D)]
pub struct Typst {
    #[base]
    pub node: Base<Sprite2D>,
    pub typst_expression: String,
}

#[godot_api]
impl ISprite2D for Typst {
    fn init(node: Base<Sprite2D>) -> Self {
        Typst { 
            node,
            typst_expression: String::new(),
        }
    }

    fn ready(&mut self) {
        self.render();
    }

    fn process(&mut self, delta: f64) {}
}

#[godot_api]
impl Typst {
    pub fn render(&mut self) {
        // Render Typst: convert latex expression to SVG, then assign to self
    }
}
