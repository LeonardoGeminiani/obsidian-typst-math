// mod obsidian;
// use js_sys::JsString;
// use wasm_bindgen::prelude::*;

// #[wasm_bindgen]
// pub struct ExampleCommand {
//     id: JsString,
//     name: JsString,
// }

// // #[wasm_bindgen]
// // impl ExampleCommand {
// //     #[wasm_bindgen(getter)]
// //     pub fn id(&self) -> JsString {
// //         self.id.clone()
// //     }

// //     #[wasm_bindgen(setter)]
// //     pub fn set_id(&mut self, id: &str) {
// //         self.id = JsString::from(id)
// //     }

// //     #[wasm_bindgen(getter)]
// //     pub fn name(&self) -> JsString {
// //         self.name.clone()
// //     }

// //     #[wasm_bindgen(setter)]
// //     pub fn set_name(&mut self, name: &str) {
// //         self.name = JsString::from(name)
// //     }

// //     pub fn callback(&self) {
// //         obsidian::Notice::new("hello from rust");
// //     }
// // }

// #[wasm_bindgen]
// pub fn test(str: JsString) {
//     obsidian::Notice::new("caop");
// }


// #[wasm_bindgen]
// pub fn onload(plugin: &obsidian::Plugin) {
//     let cmd = ExampleCommand {
//         id: JsString::from("example"),
//         name: JsString::from("Example"),
//     };
//     plugin.addCommand(JsValue::from(cmd))
// }


use fast_image_resize as fr;
use wasm_bindgen::prelude::*;
use web_sys::ImageData;

mod diagnostic;
mod file_entry;
mod render;
mod world;

use crate::world::SystemWorld;

#[wasm_bindgen]
pub struct Compiler {
    resizer: fr::Resizer,
    world: SystemWorld,
}

#[wasm_bindgen]
impl Compiler {
    #[wasm_bindgen(constructor)]
    pub fn new(root: String, request_data: &js_sys::Function) -> Self {
        console_error_panic_hook::set_once();

        Self {
            world: SystemWorld::new(root, request_data),
            resizer: fr::Resizer::default(),
        }
    }

    pub fn compile_image(
        &mut self,
        text: String,
        path: String,
        pixel_per_pt: f32,
        size: u32,
        display: bool,
    ) -> Result<ImageData, JsValue> {
        let document = self.world.compile(text, path)?;
        render::to_image(
            &mut self.resizer,
            document,
            pixel_per_pt,
            size,
            display,
        )
    }

    pub fn compile_svg(&mut self, text: String, path: String) -> Result<String, JsValue> {
        self.world
            .compile(text, path)
            .map(|document| render::to_svg(document))
    }

    pub fn add_font(&mut self, data: Vec<u8>) {
        self.world.add_font(data);
    }
}