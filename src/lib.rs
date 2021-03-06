use std::cell::RefCell;
use std::rc::Rc;
use std::convert::TryInto;

use js_sys::WebAssembly;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlRenderingContext};
use web_sys::{HtmlElement, HtmlInputElement};

mod utils;
use utils::{Timer, request_animation_frame,
            compile_shader, link_program};

mod cell;
mod js;
mod game;

use game::*;
use cell::*;

const WIDTH: u32 = 200;
const HEIGHT: u32 = 200;
const GAME_CONFIG: GameConfig = GameConfig {
    num_focus: 3,
    to_burn: 0.3,
    time_to_burn: 10,
};

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

    let size_square: f32 = 2.0 / (WIDTH as f32);
    let mut table_points = Vec::new();

    for col in 0..WIDTH {
        for row in 0..HEIGHT {
            let x = (0.5 + col as f32) * size_square - 1.0;
            let y = (0.5 + row as f32) * size_square - 1.0;

            table_points.push(x);
            table_points.push(y);
        }
    }

    let vertices = table_points.as_slice();
    context_array_bind(&context, &vertices, 0, 2)?;

    let canvas_width = 600.0;
    let size = canvas_width / (WIDTH as f32);
    let qtdd = (WIDTH * HEIGHT) as usize;
    let sizes = vec![size;qtdd];
    let sizes = sizes.as_slice();
    context_array_bind(&context, &sizes, 1, 1)?;

    let fps = Rc::new(RefCell::new(new_fps_value().unwrap()));
    {
        let fps = fps.clone();
        let a = Closure::wrap(Box::new(move || {
            *fps.borrow_mut() = new_fps_value().unwrap();
            // js::log(&fps.to_string());
        }) as Box<dyn FnMut()>);
        document
            .get_element_by_id("fps-control")
            .expect("Should have a #fps-control slider on the page")
            .dyn_ref::<HtmlElement>()
            .expect("#fps-control be an `HtmlElement`")
            .set_onchange(Some(a.as_ref().unchecked_ref()));
        a.forget();
    }

    let _timer = Timer::new("animate");
    let mut then = utils::now();

    let universe = Rc::new(RefCell::new(Universe::new(WIDTH, HEIGHT, GAME_CONFIG)));
    {
        let f = Rc::new(RefCell::new(None));
        let g = f.clone();

        let universe = universe.clone();
        let fps = fps.clone();
        *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            let fps = *fps.borrow();
            let interval: f64 = 1000.0/fps as f64;
            let now = utils::now();
            let delta = now - then;
            if delta > interval {
                then = now - (delta % interval);

                let is_over = universe.borrow_mut().tick();
                if is_over {
                    js::pause();
                    let _ = f.borrow_mut().take();
                    return;
                }
                draw_universe(&context.clone(), &universe.borrow()).unwrap();
            }

            request_animation_frame(f.borrow().as_ref().unwrap());
        }) as Box<FnMut()>));

        request_animation_frame(g.borrow().as_ref().unwrap());
    }

    Ok(())
}

fn new_fps_value() -> Result<u32, JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let new_fps = document
        .get_element_by_id("fps-control")
        .expect("Should have a #fps-control slider on the page")
        .dyn_ref::<HtmlInputElement>()
        .expect("#fps-control be an `HtmlElement`")
        .value()
        .parse::<u32>()
        .expect("Could not parse slider value");

    Ok(new_fps)
}

fn draw_universe(context: &WebGlRenderingContext, universe: &Universe) -> Result<(), JsValue> {
    let universe_size = universe.cells.len();

    let fire_colors = vec![RGB::new(249, 199, 63), RGB::new(255, 224, 70),
                           RGB::new(255, 78, 65), RGB::new(218, 51, 48),
                           RGB::new(162, 18, 18),];

    let mut colors = Vec::new();
    for i in 0..universe_size {
        if let Some(cell) = universe.cells.get(i){
            let color: (f32, f32, f32) = match cell {
                Cell::Alive => (0.0, 0.8, 0.0),
                Cell::Burning => {
                    let pos = RGB::time_to_pos(universe.time_cells[i], fire_colors.len() - 1, &universe.game_config);
                    match fire_colors.get(pos) {
                        Some(color) => color.to_float(),
                        None => (1.0, 1.0, 1.0),
                    }
                },
                Cell::Dead => (0.0, 0.0, 0.0)
            };
            colors.push(color.0);
            colors.push(color.1);
            colors.push(color.2);
        }
    }

    let colors = colors.as_slice();
    context_array_bind(context, &colors, 2, 3)?;

    context.clear_color(0.0, 0.0, 0.0, 1.0);
    context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

    context.draw_arrays(
        WebGlRenderingContext::POINTS,
        0,
        universe_size.try_into().unwrap(),
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

