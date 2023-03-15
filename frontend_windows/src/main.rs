use std::rc::Rc;
use native_windows_gui as nwg;
use native_windows_gui::PaintData;

use nes_core::ppu::{SCREEN_WIDTH, SCREEN_HEIGHT};

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let mut window = Default::default();
    nwg::Window::builder()
        .size((SCREEN_WIDTH as i32 * 3, SCREEN_HEIGHT as i32 * 3))
        .title("NES Emulator")
        .build(&mut window)
        .unwrap();

    let mut file_menu: nwg::Menu = Default::default();
    nwg::Menu::builder()
        .text("File")
        .parent(&window)
        .build(&mut file_menu)
        .unwrap();

    let mut open_file: nwg::MenuItem = Default::default();
    nwg::MenuItem::builder()
        .text("Open...")
        .parent(&file_menu)
        .build(&mut open_file)
        .unwrap();

    let mut open_recent: nwg::MenuItem = Default::default();
    nwg::MenuItem::builder()
        .text("Open recent")
        .parent(&file_menu)
        .build(&mut open_recent)
        .unwrap();

    let mut sep = Default::default();
    nwg::MenuSeparator::builder().parent(&mut file_menu).build(&mut sep).unwrap();

    let mut quit_btn = Default::default();
    nwg::MenuItem::builder().parent(&mut file_menu).text("Quit").build(&mut quit_btn).unwrap();

    let window = Rc::new(window);
    let events_window = window.clone();

    let handler = nwg::full_bind_event_handler(&window.handle, move |evt, evt_data, handle| {
        use nwg::Event as E;

        match evt {
            E::OnMenuItemSelected if handle == quit_btn => events_window.close(),
            E::OnMenuItemSelected if handle == open_file => {
                let mut dialog = Default::default();
                nwg::FileDialog::builder()
                    .title("Choose a NES ROM file")
                    .filters("NES ROM(*.nes)")
                    .action(nwg::FileDialogAction::Open)
                    .build(&mut dialog)
                    .unwrap();

                if dialog.run(Some(&events_window.handle)) {
                    let rom_file = dialog.get_selected_item().unwrap();
                    println!("Opened file {rom_file:?}");
                }
            }
            E::OnFileDrop if &handle == &events_window.handle => {
                evt_data.on_file_drop();
            }
            E::OnKeyPress => {
                evt_data.on_key();
            }
            E::OnPaint => {
                let paint: &PaintData = evt_data.on_paint();
                let ps = paint.begin_paint();
                ps.hdc;
                paint.end_paint(&ps);
            }
            _ => {}
        }
    });

    nwg::dispatch_thread_events();
    nwg::unbind_event_handler(&handler);
}