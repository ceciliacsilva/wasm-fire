use std::cell::RefCell;
use std::rc::Rc;

use js_sys::WebAssembly;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlRenderingContext};

use rand::prelude::*;

mod utils;
use utils::{Timer, request_animation_frame,
            compile_shader, link_program, hello};

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

#[derive(Clone)]
struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
    time_cells: Vec<u8>,
    game_configs: GameConfig,
}

#[derive(Clone)]
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
        count += self.cells[nw].is_burning() as u8;

        let n = self.get_index(north, column);
        count += self.cells[n].is_burning() as u8;

        let ne = self.get_index(north, east);
        count += self.cells[ne].is_burning() as u8;

        let w = self.get_index(row, west);
        count += self.cells[w].is_burning() as u8;

        let e = self.get_index(row, east);
        count += self.cells[e].is_burning() as u8;

        let sw = self.get_index(south, west);
        count += self.cells[sw].is_burning() as u8;

        let s = self.get_index(south, column);
        count += self.cells[s].is_burning() as u8;

        let se = self.get_index(south, east);
        count += self.cells[se].is_burning() as u8;

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

        let width = 100;
        let height = 100;
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

        let time_cells = (0..width * height)
            .map(|_i| {
                0
            })
            .collect();

        let game_configs = GameConfig {
            to_burn: 0.3,
            time_to_burn: 10,
            num_focus,
        };

        Universe {
            width,
            height,
            cells,
            time_cells,
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
        attribute vec2 position;
        attribute float point_size;
        attribute vec3 color;
        varying vec3 u_color;

        void main() {
            gl_Position =  vec4(position, 0, 1);
            gl_PointSize = point_size;
            u_color = color;
        }
    "#,
    )?;
    let frag_shader = compile_shader(
        &context,
        WebGlRenderingContext::FRAGMENT_SHADER,
        r#"
        precision mediump float;
        varying vec3 u_color;

        void main() {
            gl_FragColor = vec4(u_color, 1.0);
        }
    "#,
    )?;
    let program = link_program(&context, &vert_shader, &frag_shader)?;
    context.use_program(Some(&program));

    let width = 100;
    let height = 100;
    let size_square: f32 = 2.0 / (width as f32);

    let mut table_points = Vec::new();

    for col in 0..width {
        for row in 0..height {
            let x = (0.5 + col as f32) * size_square - 1.0;
            let y = (0.5 + row as f32) * size_square - 1.0;

            table_points.push(x);
            table_points.push(y);
        }
    }

    let vertices = table_points.as_slice();
    context_array_bind(&context, &vertices, 0, 2)?;

    let canvas_width = 600.0;
    let canvas_height = 600.0;

    let size = canvas_width / (width as f32);
    let qtdd = width * height;
    let sizes = vec![size;qtdd];
    let sizes = sizes.as_slice();
    context_array_bind(&context, &sizes, 1, 1)?;

    let _timer = Timer::new("animate");

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let fps = 1;
    let mut then = utils::now();
    let interval: f64 = 1000.0/fps as f64;

    let universe = Rc::new(RefCell::new(Universe::new()));
    {
        let u = universe.clone();
        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            request_animation_frame(f.borrow().as_ref().unwrap());

            let now = utils::now();
            let delta = now - then;
            if delta > interval {
                then = now - (delta % interval);

                u.borrow_mut().tick();
                animate(&context.clone(), &u.borrow()).unwrap();
            }
        }) as Box<FnMut()>));

        request_animation_frame(g.borrow().as_ref().unwrap());
    }

    Ok(())
}

fn animate(context: &WebGlRenderingContext, universe: &Universe) -> Result<(), JsValue> {
    let qtdd = 10000;
    let mut colors = Vec::new();
    for i in 0..10000 {
        if let Some(cell) = universe.cells.get(i){
            let color: (f32, f32, f32) = match cell {
                Cell::Alive => (0.0, 0.8, 0.0),
                Cell::Burning => (0.8, 0.305882, 0.105882),
                Cell::Dead => (0.0, 0.0, 0.0)
            };
            colors.push(color.0);
            colors.push(color.1);
            colors.push(color.2);
        }
    }

    // let mut rng = rand::thread_rng();
    // for i in 0..qtdd {
    //     let r = rng.gen();
    //     let g = rng.gen();
    //     let b = rng.gen();

    //     colors.push(r);
    //     colors.push(g);
    //     colors.push(b);
    // }

    let colors = colors.as_slice();
    context_array_bind(context, &colors, 2, 3)?;

    context.clear_color(0.0, 0.0, 0.0, 1.0);
    context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

    context.draw_arrays(
        WebGlRenderingContext::POINTS,
        0,
        qtdd
    );

    Ok(())
}

fn context_array_bind(context: &WebGlRenderingContext, data: &[f32], index: u32, size: i32) -> Result<(), JsValue> {
    let memory_buffer = wasm_bindgen::memory()
        .dyn_into::<WebAssembly::Memory>()?
        .buffer();
    let data_location = data.as_ptr() as u32 / 4;
    let data_array = js_sys::Float32Array::new(&memory_buffer)
        .subarray(data_location, data_location + data.len() as u32);

    let buffer = context.create_buffer().ok_or("failed to create buffer")?;
    context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));
    context.buffer_data_with_array_buffer_view(
        WebGlRenderingContext::ARRAY_BUFFER,
        &data_array,
        WebGlRenderingContext::STATIC_DRAW,
    );
    context.vertex_attrib_pointer_with_i32(index, size, WebGlRenderingContext::FLOAT, false, 0, 0);
    context.enable_vertex_attrib_array(index);
    context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, None);

    Ok(())
}

