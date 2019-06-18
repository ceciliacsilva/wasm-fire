use std::cell::RefCell;
use std::rc::Rc;

use js_sys::WebAssembly;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlShader};

use rand::prelude::*;

#[wasm_bindgen]
extern {
    pub fn alert(s: &str);
}

#[wasm_bindgen]
pub fn say_hi(){
    alert("Hello.");
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
    web_sys::console::time_with_label("some operation");

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

    web_sys::console::time_end_with_label("some operation");

    Ok(())
}

fn now() -> f64 {
    web_sys::window()
        .expect("should have a Window")
        .performance()
        .expect("should have a Performance")
        .now()
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<FnMut(f32)>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` Ok");
}


pub fn compile_shader(
    context: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

pub fn link_program(
    context: &WebGlRenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}
