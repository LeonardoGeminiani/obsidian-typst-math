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
use js_sys::Math::log;
use typst::text;
use wasm_bindgen::prelude::*;
use web_sys::{console, ImageData};

mod diagnostic;
mod file_entry;
mod render;
mod world;

use crate::world::SystemWorld;

const KEY_WORD_MAX: usize = 100;

struct SvgBuffEl {
    key_word: [u8; KEY_WORD_MAX],
    output: String,
}

const BUFFER_SIZE: usize = 500;

#[wasm_bindgen]
pub struct Compiler {
    resizer: fr::Resizer,
    world: SystemWorld,
    svg_buff: CircularBuffer<500, SvgBuffEl>,
}

use circular_buffer::CircularBuffer;

#[wasm_bindgen]
impl Compiler {
    #[wasm_bindgen(constructor)]
    pub fn new(root: String, request_data: &js_sys::Function) -> Self {
        console_error_panic_hook::set_once();

        Self {
            world: SystemWorld::new(root, request_data),
            resizer: fr::Resizer::default(),
            svg_buff: CircularBuffer::<BUFFER_SIZE, SvgBuffEl>::new(),
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
        render::to_image(&mut self.resizer, document, pixel_per_pt, size, display)
    }

    pub fn compile_svg(&mut self, text: String, _key_word: String , path: String) -> Result<String, JsValue> {
        let vec: &[u8] = _key_word.as_bytes();

        // Create a fixed-size array filled with zeros
        let mut fixed_len_array: [u8; KEY_WORD_MAX] = [0; KEY_WORD_MAX];

        // Copy as much data from the slice as possible into the fixed-size array
        let copy_len = vec.len().min(KEY_WORD_MAX);
        fixed_len_array[..copy_len].copy_from_slice(&vec[..copy_len]);

        let log = fixed_len_array.to_vec();
        console::log_1(&JsValue::from(log));

        if vec.len() <= KEY_WORD_MAX {
            // optimize
            console::log_1(&"optimize".into());

            for el in self.svg_buff.iter() {
                let ver_cmp = el.key_word;

                if ver_cmp.eq(&fixed_len_array) {
                    console::log_1(&"optimiZEED!".into());
                    return Ok(el.output.clone());
                }
            }
        }

        let ret = self
            .world
            .compile(text, path)
            .map(|document| render::to_svg(document));

        if let Ok(ref output) = ret {
            self.svg_buff.push_front(SvgBuffEl {
                key_word: fixed_len_array.clone(),
                output: output.to_string(),
            });
        }

        console::log_1(&"compiled!".into());

        return ret;
    }

    pub fn add_font(&mut self, data: Vec<u8>) {
        self.world.add_font(data);
    }
}
