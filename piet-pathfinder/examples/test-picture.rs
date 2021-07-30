//! Run the piet-test examples with the coregraphics backend.

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::sync::Arc;

use gl::types::GLvoid;
use surfman::SystemConnection;
use surfman::{ContextAttributeFlags, ContextAttributes, SurfaceAccess, SurfaceType};

use euclid::default::Size2D;
use pathfinder_canvas::{vec2f, vec2i, CanvasFontContext, ColorF, Transform2F};
use pathfinder_gl::{GLDevice, GLVersion};
use pathfinder_gpu::Device;
use pathfinder_renderer::concurrent::rayon::RayonExecutor;
use pathfinder_renderer::gpu::options::{DestFramebuffer, RendererMode, RendererOptions};
use pathfinder_renderer::gpu::renderer::Renderer;
use pathfinder_renderer::options::{BuildOptions, RenderTransform};
use pathfinder_resources::embedded::EmbeddedResourceLoader;
use piet::kurbo::Size;
use piet::{samples, RenderContext};
use piet_pathfinder::PathFinderRenderContext;
use surfman::Connection;

const SCALE: f64 = 2.0;
const FILE_PREFIX: &str = "pathfinder-test-";

fn main() {
    samples::samples_main(run_sample, FILE_PREFIX, None);
}

fn run_sample(idx: usize, base_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let sample = samples::get(idx)?;
    let size = sample.size();

    let file_name = format!("{}{}.png", FILE_PREFIX, idx);
    let path = base_dir.join(file_name);

    let mut canvas = pathfinder_canvas::Canvas::new(vec2f(size.width as f32, size.height as f32));
    let font_source = Arc::new(piet_pathfinder::FontSource::new(vec![Box::new(
        font_kit::source::SystemSource::new(),
    )]));
    let font_context = CanvasFontContext::new(font_source.clone());
    let mut rendering_context = canvas.get_context_2d(font_context);
    let mut piet_context = PathFinderRenderContext::new(&mut rendering_context, font_source);

    sample.draw(&mut piet_context)?;

    piet_context.finish()?;
    std::mem::drop(piet_context);
    let mut scene = rendering_context.into_canvas().into_scene();

    // openGL bits
    let connection = Connection::new().unwrap();
    let adapter = connection.create_adapter().unwrap();
    let mut device = connection.create_device(&adapter).unwrap();
    let context_attributes = ContextAttributes {
        version: surfman::GLVersion::new(3, 3),
        flags: ContextAttributeFlags::empty(),
    };
    let context_descriptor = device
        .create_context_descriptor(&context_attributes)
        .unwrap();
    let mut context = device.create_context(&context_descriptor, None).unwrap();
    let surface = device
        .create_surface(
            &context,
            SurfaceAccess::GPUOnly,
            SurfaceType::Generic {
                size: Size2D::new(size.width as i32, size.height as i32),
            },
        )
        .unwrap();
    device
        .bind_surface_to_context(&mut context, surface)
        .unwrap();
    device.make_context_current(&context).unwrap();
    gl::load_with(|symbol_name| device.get_proc_address(&context, symbol_name));

    let gl_device = GLDevice::new(GLVersion::GL3, 0);
    let texture = gl_device.create_texture(
        pathfinder_gpu::TextureFormat::RGBA8,
        vec2i(size.width as i32, size.height as i32),
    );
    let framebuffer = gl_device.create_framebuffer(texture);
    let mode = RendererMode::default_for_device(&gl_device);
    let options = RendererOptions {
        background_color: Some(ColorF::white()),
        dest: DestFramebuffer::Other(framebuffer),
        ..RendererOptions::default()
    };
    let mut renderer = Renderer::new(gl_device, &EmbeddedResourceLoader, mode, options);
    scene.build_and_render(
        &mut renderer,
        BuildOptions {
            transform: RenderTransform::Transform2D(
                Transform2F::default().scale(vec2f(SCALE as f32, SCALE as f32)),
            ),
            ..BuildOptions::default()
        },
        RayonExecutor,
    );
    let mut data: Vec<u8> = vec![0; size.width as usize * size.height as usize * 4];
    unsafe {
        gl::ReadPixels(
            0,
            0,
            size.width as i32,
            size.height as i32,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            data.as_mut_ptr() as *mut GLvoid,
        );
    }
    // data.reverse();

    device.destroy_context(&mut context).unwrap();

    let file = File::create(path)?;
    let w = BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, size.width as u32, size.height as u32);
    encoder.set_color(png::ColorType::RGBA);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&data).map_err(Into::into)
}
