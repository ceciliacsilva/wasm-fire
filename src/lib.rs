use std::cell::RefCell;
use std::rc::Rc;

use js_sys::WebAssembly;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlRenderingContext};

use rand::prelude::*;

mod utils;
use utils::{Timer, request_animation_frame, compile_shader, link_program};

#[wasm_bindgen]
extern {
    pub fn alert(s: &str);
}

#[wasm_bindgen]
pub fn say_hi(){
    alert("Hello.");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cell {
    Dead,
    Burning,
    Alive,
}

impl Cell {
    fn live_to_burn(&mut self) {
        if let Cell::Alive = *self {
            *self = Cell::Burning
        };
    }

    fn is_burning(&self) -> u8 {
        match *self {
            Cell::Dead => 0,
            Cell::Burning => 1,
            Cell::Alive => 0,
        }
    }
}

struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
    time_cells: Vec<u8>,
    game_configs: GameConfig,
}

struct GameConfig {
    to_burn: f32,
    time_to_burn: u8,
    num_focus: u8,
}

impl Universe {
    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn burning_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;

        let north = if row == 0 {
            self.height - 1
        } else {
            row - 1
        };

        let south = if row == self.height - 1 {
            0
        } else {
            row + 1
        };

        let west = if column == 0 {
            self.width - 1
        } else {
            column - 1
        };

        let east = if column == self.width - 1 {
            0
        } else {
            column + 1
        };

        let nw = self.get_index(north, west);
        count += self.cells[nw] as u8;

        let n = self.get_index(north, column);
        count += self.cells[n] as u8;

        let ne = self.get_index(north, east);
        count += self.cells[ne] as u8;

        let w = self.get_index(row, west);
        count += self.cells[w] as u8;

        let e = self.get_index(row, east);
        count += self.cells[e] as u8;

        let sw = self.get_index(south, west);
        count += self.cells[sw] as u8;

        let s = self.get_index(south, column);
        count += self.cells[s] as u8;

        let se = self.get_index(south, east);
        count += self.cells[se] as u8;

        count
    }
}

impl Universe {
    fn tick (&mut self) {
        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];

                if cell == Cell::Alive {
                    let burning_neighbor = self.burning_neighbor_count(row, col);

                    if burning_neighbor > 0 {
                        let mut rng = rand::thread_rng();
                        let prob_burn: f32 = rng.gen();

                        if prob_burn < self.game_configs.to_burn {
                            next[idx].live_to_burn();
                        }
                    }
                }

                if cell == Cell::Burning {
                    self.time_cells[idx] += 1;
                }
            }
        }

        self.cells = next;
    }

    fn new() -> Universe {
        utils::set_panic_hook();

        let width = 128;
        let height = 128;
        let num_focus = 3;
        let mut rng = rand::thread_rng();
        let mut pos = Vec::new();

        for _i in 0..num_focus {
            let n: u32 = rng.gen_range(0, width * height);
            pos.push(n);
        }

        let cells = (0..width * height)
            .map(|i| {
                if pos.contains(&i) {
                    Cell::Burning
                } else {
                    Cell::Alive
                }
            })
            .collect();

        let game_configs = GameConfig {
            to_burn: 0.7,
            time_to_burn: 10,
            num_focus,
        };

        Universe {
            width,
            height,
            cells,
            time_cells: Vec::new(),
            game_configs,
        }
    }
}

#[wasm_bindgen]
pub fn start() -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let context = canvas
        .get_context("webgl")?
        .unwrap()
        .dyn_into::<WebGlRenderingContext>()?;

    let vert_shader = compile_shader(
        &context,
        WebGlRenderingContext::VERTEX_SHADER,
        r#"
        attribute vec4 position;
        attribute float point_size;
        void main() {
            gl_Position = position;
            gl_PointSize = point_size;
        }
    "#,
    )?;
    let frag_shader = compile_shader(
        &context,
        WebGlRenderingContext::FRAGMENT_SHADER,
        r#"
        void main() {
            gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
        }
    "#,
    )?;
    let program = link_program(&context, &vert_shader, &frag_shader)?;
    context.use_program(Some(&program));

    let vertices: [f32; 9] = [-0.9, -0.9, 0.0, 0.7, -0.7, 0.0, 0.0, 0.7, 0.0];
    let memory_buffer = wasm_bindgen::memory()
        .dyn_into::<WebAssembly::Memory>()?
        .buffer();
    let vertices_location = vertices.as_ptr() as u32 / 4;
    let vert_array = js_sys::Float32Array::new(&memory_buffer)
        .subarray(vertices_location, vertices_location + vertices.len() as u32);

    let buffer = context.create_buffer().ok_or("failed to create buffer")?;
    context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));
    context.buffer_data_with_array_buffer_view(
        WebGlRenderingContext::ARRAY_BUFFER,
        &vert_array,
        WebGlRenderingContext::STATIC_DRAW,
    );
    context.vertex_attrib_pointer_with_i32(0, 3, WebGlRenderingContext::FLOAT, false, 0, 0);
    context.enable_vertex_attrib_array(0);

    let _timer = Timer::new("animate");

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move |_dt| {
        animate(&context.clone()).unwrap();
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<FnMut(f32)> ));

    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())
}

fn animate(
    context: &WebGlRenderingContext,
) -> Result<(), JsValue> {

    let mut rng = rand::thread_rng();
    let size: f32 = rng.gen();

    let sizes: [f32; 3] = [size * 10.0, size * 10.0, size * 10.0];
    let memory_buffer = wasm_bindgen::memory()
        .dyn_into::<WebAssembly::Memory>()?
        .buffer();
    let sizes_location = sizes.as_ptr() as u32 / 4;
    let sizes_array = js_sys::Float32Array::new(&memory_buffer)
        .subarray(sizes_location, sizes_location + sizes.len() as u32);

    let buffer = context.create_buffer().ok_or("failed to create buffer")?;
    context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));
    context.buffer_data_with_array_buffer_view(
        WebGlRenderingContext::ARRAY_BUFFER,
        &sizes_array,
        WebGlRenderingContext::STATIC_DRAW,
    );
    context.vertex_attrib_pointer_with_i32(1, 1, WebGlRenderingContext::FLOAT, false, 0, 0);
    context.enable_vertex_attrib_array(1);

    context.clear_color(0.0, 0.5, 0.0, 1.0);
    context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

    context.draw_arrays(
        WebGlRenderingContext::POINTS,
        0,
        3
    );

    Ok(())
}

