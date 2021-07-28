// pathfinder/examples/canvas_glutin_minimal/src/main.rs
//
// Copyright © 2019 The Pathfinder Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Demonstrates how to use the Pathfinder canvas API with `glutin`.

use glutin::dpi::PhysicalSize;
use glutin::event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::WindowBuilder;
use glutin::{ContextBuilder, GlProfile, GlRequest};
use image::GenericImageView;
use pathfinder_canvas::{Canvas, CanvasFontContext, CanvasRenderingContext2D, Path2D};
use pathfinder_color::ColorF;
use pathfinder_geometry::rect::RectF;
use pathfinder_geometry::vector::{vec2f, vec2i};
use pathfinder_gl::{GLDevice, GLVersion};
use pathfinder_renderer::concurrent::rayon::RayonExecutor;
use pathfinder_renderer::concurrent::scene_proxy::SceneProxy;
use pathfinder_renderer::gpu::options::{DestFramebuffer, RendererOptions};
use pathfinder_renderer::gpu::renderer::Renderer;
use pathfinder_renderer::options::BuildOptions;
use pathfinder_resources::embedded::EmbeddedResourceLoader;
use piet::ImageFormat;

fn main() {
    // Calculate the right logical size of the window.
    let event_loop = EventLoop::new();
    let window_size = vec2i(640, 480);
    let physical_window_size = PhysicalSize::new(window_size.x() as f64, window_size.y() as f64);

    // Open a window.
    let window_builder = WindowBuilder::new()
        .with_title("Minimal example")
        .with_inner_size(physical_window_size);

    // Create an OpenGL 3.x context for Pathfinder to use.
    let gl_context = ContextBuilder::new()
        .with_gl(GlRequest::Latest)
        .with_gl_profile(GlProfile::Core)
        .build_windowed(window_builder, &event_loop)
        .unwrap();

    // Load OpenGL, and make the context current.
    let gl_context = unsafe { gl_context.make_current().unwrap() };
    gl::load_with(|name| gl_context.get_proc_address(name) as *const _);

    // Create a Pathfinder renderer.
    let device = GLDevice::new(GLVersion::GL3, 0);
    let dest = DestFramebuffer::full_window(window_size);
    let options = RendererOptions {
        background_color: Some(ColorF::white()),
        ..RendererOptions::default()
    };
    let mut renderer = Renderer::new(device, &EmbeddedResourceLoader, dest, options);

    // Make a canvas. We're going to draw a house.
    let font_source_mem = font_kit::sources::mem::MemSource::empty();
    let font_source_sys = font_kit::source::SystemSource::new();
    let font_source =
        std::sync::Arc::new(font_kit::sources::multi::MultiSource::from_sources(vec![
            Box::new(font_source_mem),
            Box::new(font_source_sys),
        ]));
    let font_context = CanvasFontContext::new(font_source);
    let mut canvas = Canvas::new(window_size.to_f32()).get_context_2d(font_context);
    let mut piet_canvas = piet_pathfinder::PathFinderRenderContext::new(&mut canvas);
    draw_a_house(&mut piet_canvas);
    draw_a_picture(&mut piet_canvas);

    // Render the canvas to screen.
    let scene = SceneProxy::from_scene(canvas.into_canvas().into_scene(), RayonExecutor);
    scene.build_and_render(&mut renderer, BuildOptions::default());
    gl_context.swap_buffers().unwrap();

    // Wait for a keypress.
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            }
            | Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {
                *control_flow = ControlFlow::Wait;
            }
        };
    })
}

fn draw_a_house(canvas: &mut impl piet::RenderContext) {
    // Set line width.
    // Draw walls
    let brush = canvas.solid_brush(piet::Color::BLACK);
    canvas.stroke(
        piet::kurbo::Rect::from_origin_size((75.0, 140.0), (150.0, 110.0)),
        &brush,
        10.0,
    );
    // canvas.stroke_rect(RectF::new(vec2f(75.0, 140.0), vec2f(150.0, 110.0)));

    // Draw door.
    canvas.fill(
        piet::kurbo::Rect::from_origin_size((130., 190.), (40., 60.)),
        &brush,
    );
    // canvas.fill_rect(RectF::new(vec2f(130.0, 190.0), vec2f(40.0, 60.0)));

    // Draw roof.
    let mut path = piet::kurbo::BezPath::new();
    path.move_to((50., 140.));
    path.line_to((150., 60.));
    path.line_to((250., 140.));
    path.close_path();
    canvas.stroke(path, &brush, 10.0);
    // let mut path = Path2D::new();
    // path.move_to(vec2f(50.0, 140.0));
    // path.line_to(vec2f(150.0, 60.0));
    // path.line_to(vec2f(250.0, 140.0));
    // path.close_path();
    // canvas.stroke_path(path);
}

fn draw_a_picture(canvas: &mut impl piet::RenderContext) {
    use image::io::Reader as ImageReader;
    let img = ImageReader::open("./piet-pathfinder/test.png")
        .unwrap()
        .decode()
        .unwrap();
    let img = canvas
        .make_image(
            img.width() as usize,
            img.height() as usize,
            img.as_bytes(),
            ImageFormat::RgbaSeparate,
        )
        .unwrap();
    canvas.draw_image(
        &img,
        piet::kurbo::Rect {
            x0: 300.0,
            y0: 300.0,
            x1: 600.0,
            y1: 600.0,
        },
        piet::InterpolationMode::Bilinear,
    )
}
