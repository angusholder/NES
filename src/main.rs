#![allow(dead_code)]

mod cartridge;
mod nes;
mod ops;
#[cfg(test)]
mod test_instructions;
mod instructions;

use std::path::PathBuf;
use std::time::Instant;

use glutin::{event_loop::EventLoop, WindowedContext};
use imgui_glow_renderer::glow::HasContext;
use imgui_glow_renderer::glow;
use imgui_winit_support::WinitPlatform;

type Window = WindowedContext<glutin::PossiblyCurrent>;

fn main() {
    // Common setup for creating a winit window and imgui context, not specifc
    // to this renderer at all except that glutin is used to create the window
    // since it will give us access to a GL context
    let (event_loop, window) = create_window();
    let (mut imgui_platform, mut imgui): (WinitPlatform, imgui::Context) = imgui_init(&window);

    // OpenGL context from glow
    let gl = create_opengl_context(&window);

    // OpenGL renderer from this crate
    let mut imgui_texture_map = imgui_glow_renderer::SimpleTextureMap::default();
    let mut imgui_renderer = imgui_glow_renderer::Renderer::initialize(&gl, &mut imgui, &mut imgui_texture_map, true)
        .expect("failed to create renderer");

    let mut last_frame = Instant::now();

    // Standard winit event loop
    event_loop.run(move |event, _, control_flow| {
        match event {
            glutin::event::Event::NewEvents(_) => {
                let now = Instant::now();
                imgui.io_mut().update_delta_time(now.duration_since(last_frame));
                last_frame = now;
            }
            glutin::event::Event::MainEventsCleared => {
                imgui_platform.prepare_frame(imgui.io_mut(), window.window()).unwrap();
                window.window().request_redraw();
            }
            glutin::event::Event::RedrawRequested(_) => {
                // Clear the screen from the last frame
                unsafe { gl.clear(glow::COLOR_BUFFER_BIT) };

                let ui = imgui.frame();
                ui.show_demo_window(&mut true);

                imgui_platform.prepare_render(ui, window.window());
                let draw_data = imgui.render();

                // This is the only extra render step to add
                imgui_renderer
                    .render(&gl, &imgui_texture_map, draw_data)
                    .expect("error rendering imgui");

                window.swap_buffers().unwrap();
            }
            glutin::event::Event::WindowEvent {
                event: glutin::event::WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = glutin::event_loop::ControlFlow::Exit;
            }
            event => {
                imgui_platform.handle_event(imgui.io_mut(), window.window(), &event);
            }
        }
    });
}

fn create_window() -> (EventLoop<()>, Window) {
    let event_loop = glutin::event_loop::EventLoop::new();
    let window = glutin::window::WindowBuilder::new()
        .with_title("NES Emulator")
        .with_inner_size(glutin::dpi::LogicalSize::new(1024, 768));
    let window = glutin::ContextBuilder::new()
        .with_vsync(true)
        .build_windowed(window, &event_loop)
        .expect("could not create window");
    let window = unsafe {
        window
            .make_current()
            .expect("could not make window context current")
    };
    (event_loop, window)
}

fn create_opengl_context(window: &Window) -> glow::Context {
    unsafe { glow::Context::from_loader_function(|s| window.get_proc_address(s).cast()) }
}

fn imgui_init(window: &Window) -> (WinitPlatform, imgui::Context) {
    let mut imgui_context = imgui::Context::create();
    imgui_context.set_ini_filename(Some(PathBuf::from("imgui.ini")));

    let mut winit_platform = WinitPlatform::init(&mut imgui_context);
    winit_platform.attach_window(
        imgui_context.io_mut(),
        window.window(),
        imgui_winit_support::HiDpiMode::Rounded,
    );

    imgui_context
        .fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

    imgui_context.io_mut().font_global_scale = (1.0 / winit_platform.hidpi_factor()) as f32;

    (winit_platform, imgui_context)
}

