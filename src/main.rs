#![allow(dead_code)]

mod cartridge;
mod nes;
mod ops;
#[cfg(test)]
mod test_instructions;
mod instructions;
mod ppu;

use std::path::PathBuf;
use std::time::Instant;
use glow::{Context as OpenGL, Texture as GLTexture};

use glutin::{event_loop::EventLoop, WindowedContext};
use imgui::Ui;
use imgui_glow_renderer::glow::HasContext;
use imgui_glow_renderer::{glow, TextureMap};
use imgui_winit_support::WinitPlatform;

type Window = WindowedContext<glutin::PossiblyCurrent>;

pub const SCREEN_WIDTH: i32 = 256;
pub const SCREEN_HEIGHT: i32 = 240;
pub const SCREEN_DIMENSIONS: [i32; 2] = [SCREEN_WIDTH, SCREEN_HEIGHT];
pub const PATTERN_TABLE_WIDTH: i32 = 128;
pub const PATTERN_TABLE_HEIGHT: i32 = 128;
pub const PATTERN_TABLE_DIMENSIONS: [i32; 2] = [PATTERN_TABLE_WIDTH, PATTERN_TABLE_HEIGHT];

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

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

    let mut show_demo_window = false;
    let mut show_ppu_viewer = true;
    let mut show_nametable_viewer = false;

    let nes_texture: GLTexture = create_nes_texture(&gl);
    fill_texture(&gl, nes_texture, SCREEN_DIMENSIONS, 0xFF00FFFF);
    let pattern_tables_textures = [
        create_nes_texture(&gl),
        create_nes_texture(&gl),
    ];
    fill_texture(&gl, pattern_tables_textures[0], PATTERN_TABLE_DIMENSIONS, 0x00FF00FF);
    fill_texture(&gl, pattern_tables_textures[1], PATTERN_TABLE_DIMENSIONS, 0x5000FFFF);

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
                if show_demo_window {
                    ui.show_demo_window(&mut show_demo_window);
                }
                ui.main_menu_bar(|| {
                    ui.menu("Menu", || {
                        if ui.menu_item("Pattern Table Viewer...") {
                            show_ppu_viewer = true;
                        }
                        if ui.menu_item("Nametable Viewer...") {
                            show_nametable_viewer = true;
                        }
                        ui.separator();
                        if ui.menu_item("Show ImGUI demo window...") {
                            show_demo_window = true;
                        }
                        ui.separator();
                        if ui.menu_item("Exit") {
                            *control_flow = glutin::event_loop::ControlFlow::Exit;
                        }
                    });
                });
                ui.window("NES View")
                    .content_size([SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32])
                    .resizable(false)
                    .build(|| {
                        display_image(ui, nes_texture, [0.0, 0.0], SCREEN_DIMENSIONS);
                    });
                if show_ppu_viewer {
                    let gap = 4.0;
                    ui.window("Pattern Table Viewer")
                        .content_size([(PATTERN_TABLE_WIDTH * 2) as f32 + gap, PATTERN_TABLE_HEIGHT as f32])
                        .resizable(false)
                        .opened(&mut show_ppu_viewer)
                        .build(|| {
                            display_image(ui, pattern_tables_textures[0], [0.0, 0.0], PATTERN_TABLE_DIMENSIONS);
                            display_image(ui, pattern_tables_textures[1], [PATTERN_TABLE_WIDTH as f32 + gap, 0.0], PATTERN_TABLE_DIMENSIONS);
                        });
                }
                if show_nametable_viewer {
                    ui.window("Nametable Viewer")
                        .content_size([(SCREEN_WIDTH * 2) as f32, (SCREEN_HEIGHT * 2) as f32])
                        .resizable(false)
                        .opened(&mut show_nametable_viewer)
                        .build(|| {

                        });
                }

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
            glutin::event::Event::WindowEvent { event: glutin::event::WindowEvent::DroppedFile(path), .. } => {
                cartridge::parse_rom(&path).unwrap();
            }
            event => {
                imgui_platform.handle_event(imgui.io_mut(), window.window(), &event);
            }
        }
    });
}

fn create_nes_texture(gl: &OpenGL) -> GLTexture {
    let nes_texture: GLTexture;
    unsafe {
        nes_texture = gl.create_texture().unwrap();
        gl.bind_texture(glow::TEXTURE_2D, Some(nes_texture));
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::NEAREST as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::NEAREST as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, glow::CLAMP_TO_BORDER as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, glow::CLAMP_TO_BORDER as i32);
        gl.bind_texture(glow::TEXTURE_2D, None);
    }
    nes_texture
}

fn write_texture(gl: &OpenGL, nes_texture: GLTexture, buffer: &[u8], size: [i32; 2]) {
    unsafe {
        gl.bind_texture(glow::TEXTURE_2D, Some(nes_texture));
        gl.tex_image_2d(glow::TEXTURE_2D, 0, glow::RGBA as i32, size[0], size[1], 0, glow::RGBA, glow::UNSIGNED_BYTE, Some(buffer));
        gl.bind_texture(glow::TEXTURE_2D, None);
    }
}

fn fill_texture(gl: &OpenGL, texture: GLTexture, size: [i32; 2], color: u32) {
    let mut contents = vec![0; (size[0] * size[1] * 4) as usize];
    fill_color(&mut contents, color);
    write_texture(&gl, texture, &contents, size);
}

fn fill_color(bytes: &mut [u8], rgba: u32) {
    for i in 0..bytes.len() / 4 {
        let offset = i * 4;
        bytes[offset] = (rgba >> 24) as u8;
        bytes[offset + 1] = (rgba >> 16) as u8;
        bytes[offset + 2] = (rgba >> 8) as u8;
        bytes[offset + 3] = rgba as u8;
    }
}

fn display_image(ui: &Ui, texture: GLTexture, offset: [f32; 2], size: [i32; 2]) {
    let pos = ui.window_pos();
    let win_min = ui.window_content_region_min();

    let start = [pos[0] + win_min[0] + offset[0], pos[1] + win_min[1] + offset[1]];

    ui.get_window_draw_list()
        .add_image(
            imgui::TextureId::new(texture as _),
            start,
            [start[0] + size[0] as f32, start[1] + size[1] as f32]
        )
        .build();
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

fn create_opengl_context(window: &Window) -> OpenGL {
    unsafe { OpenGL::from_loader_function(|s| window.get_proc_address(s).cast()) }
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

