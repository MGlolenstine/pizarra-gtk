#![windows_subsystem = "windows"]
use std::cell::RefCell;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::rc::Rc;

use cairo::ImageSurface;
use gdk::{DeviceToolType, EventMask, EventType, ModifierType};
use gio::ApplicationFlags;
use glib::clone;
use gtk::prelude::*;
use gtk::{
    AboutDialog, Application, ApplicationWindow, Builder, Button, ButtonsType, ColorButton,
    DialogFlags, DrawingArea, FileChooserAction, FileChooserNative, HeaderBar, Image, MenuItem,
    MessageDialog, MessageType, ResponseType, ScaleButton, Window,
};

use pizarra::prelude::*;

mod config;
mod graphics;
mod logic;

use graphics::Drawable;
use logic::*;

#[macro_use]
extern crate rust_i18n;

i18n!("locales", fallback = "en");

fn yes_no_cancel_dialog<F, G, H>(
    window: &ApplicationWindow,
    message: &str,
    yes_callback: F,
    no_callback: G,
    cancel_callback: H,
) -> Inhibit
where
    F: Fn() -> Inhibit,
    G: Fn() -> Inhibit,
    H: Fn() -> Inhibit,
{
    let message_dialog = MessageDialog::new(
        Some(window),
        DialogFlags::DESTROY_WITH_PARENT,
        MessageType::Question,
        ButtonsType::None,
        message,
    );
    message_dialog.add_button(&t!("Yes"), ResponseType::Yes);
    message_dialog.add_button(&t!("No"), ResponseType::No);
    message_dialog.add_button(&t!("Cancel"), ResponseType::Cancel);

    let response = message_dialog.run();

    message_dialog.hide();

    match response {
        ResponseType::Yes => yes_callback(),
        ResponseType::No => no_callback(),
        _ => cancel_callback(),
    }
}

fn gtk_button(btn: u32) -> MouseButton {
    match btn {
        1 => MouseButton::Left,
        2 => MouseButton::Middle,
        3 => MouseButton::Right,
        _ => MouseButton::Unknown,
    }
}

fn gtk_key(name: &str) -> Key {
    match name {
        "Shift_L" | "Shift_R" => Key::Shift,
        "Escape" => Key::Escape,
        _ => Key::Unknown,
    }
}

fn gtk_flags(flags: ModifierType) -> Flags {
    Flags {
        alt: flags.contains(ModifierType::MOD1_MASK),
        ctrl: flags.contains(ModifierType::CONTROL_MASK),
        shift: flags.contains(ModifierType::SHIFT_MASK),
    }
}

fn gtk_tool(tool: DeviceToolType) -> Option<SelectedTool> {
    match tool {
        DeviceToolType::Eraser => Some(SelectedTool::Eraser),
        _ => None,
    }
}

fn init(app: &Application, filename: Option<PathBuf>) {
    let unsaved_changes_since_last_time_const: Cow<'_, str> =
        t!("You made some changes.\n\nWould you like to save?");
    let unsaved_changes_new_file_const: Cow<'_, str> =
        t!("There are some unsaved changes present.\n\nWould you like to save?");

    gtk::init().expect("Failed to initialize GTK.");

    let locale = std::env::var("LANG").unwrap_or_else(|_| "en_GB.UTF-8".to_string());
    let domain = "tk.categulario.pizarra";
    let locale_dir = "/tk/categulario/pizarra/locales";

    gettextrs::setlocale(gettextrs::LocaleCategory::LcAll, locale).unwrap();
    gettextrs::bindtextdomain(domain, locale_dir).unwrap();
    gettextrs::textdomain(domain).unwrap();

    // Initialize layout from .glade file
    let builder = Builder::from_resource("/tk/categulario/pizarra/pizarra.glade");
    let controller = Rc::new(RefCell::new(Pizarra::new(
        Vec2D::new_screen(1.0, 1.0),
        config::read(),
    )));
    let window: ApplicationWindow = builder.object("main-window").expect("Couldn't get window");
    let header_bar: HeaderBar = builder.object("header-bar").expect("no header bar");
    let surface = Rc::new(RefCell::new(
        ImageSurface::create(cairo::Format::ARgb32, 1, 1).unwrap(),
    ));
    let about_dialog: AboutDialog = builder.object("about-dialog").unwrap();
    let tool_btn: Button = builder.object("tool-menu-btn").unwrap();

    window.set_application(Some(app));

    if let Some(filename) = filename {
        let mut svg = String::new();
        let mut file = File::open(&filename).expect("Could not open given file");

        file.read_to_string(&mut svg)
            .expect("could not read file contents");
        {
            let mut controller = controller.borrow_mut();
            controller.open(&svg).expect("Could not parse given file");
            controller.set_saved(filename);
        }
        set_subtitle(&header_bar, controller.borrow().get_save_status());
    }

    // save on exit
    let unsaved_changes_new_file = unsaved_changes_new_file_const.to_string();
    let unsaved_changes_since_last_time = unsaved_changes_since_last_time_const.to_string();
    window.connect_delete_event(clone!(@strong controller, @strong window => move |_window, _event| {
        let old_save_status = {
            controller.borrow().get_save_status().clone()
        };

        match old_save_status {
            SaveStatus::NewAndChanged => yes_no_cancel_dialog(&window, &unsaved_changes_new_file, || {
                let save_file_chooser = FileChooserNative::new(Some("Guardar"), Some(&window), FileChooserAction::Save, Some("Guardar"), Some("Cancelar"));
                let res = save_file_chooser.run();

                match res {
                    ResponseType::Accept => {
                        if let Some(filename) = save_file_chooser.filename() {
                            save_to_svg_logic_with_error_dialg(&window, controller.clone(), &filename)
                        } else {
                            Inhibit(true)
                        }
                    },
                    _ => {
                        Inhibit(true)
                    },
                }
            }, || {
                Inhibit(false)
            }, || {
                Inhibit(true)
            }),
            SaveStatus::NewAndEmpty => Inhibit(false),
            SaveStatus::Saved(_path) => Inhibit(false),
            SaveStatus::Unsaved(path) => yes_no_cancel_dialog(&window, &unsaved_changes_since_last_time.to_string(), || {
                    save_to_svg_logic_with_error_dialg(&window, controller.clone(), &path)
                }, || {
                    Inhibit(false)
                }, || {
                    Inhibit(true)
                }),
        }
    }));

    // Drawing area
    let drawing_area: DrawingArea = builder.object("drawing-area").expect("No drawing_area");

    let event_mask = EventMask::POINTER_MOTION_MASK
        | EventMask::BUTTON_PRESS_MASK
        | EventMask::BUTTON_RELEASE_MASK
        | EventMask::KEY_PRESS_MASK
        | EventMask::KEY_RELEASE_MASK
        | EventMask::POINTER_MOTION_MASK
        | EventMask::TABLET_PAD_MASK
        | EventMask::SCROLL_MASK
        | EventMask::TOUCH_MASK
        | EventMask::POINTER_MOTION_MASK
        | EventMask::SMOOTH_SCROLL_MASK
        | EventMask::ENTER_NOTIFY_MASK
        | EventMask::LEAVE_NOTIFY_MASK
        | EventMask::PROXIMITY_IN_MASK
        | EventMask::PROXIMITY_OUT_MASK;

    drawing_area.set_support_multidevice(true);
    drawing_area.set_can_focus(true);
    drawing_area.add_events(event_mask);

    drawing_area.connect_draw(
        clone!(@strong controller, @strong surface => move |_dw, ctx| {
            ctx.set_source_surface(&surface.borrow(), 0.0, 0.0).unwrap();
            ctx.paint().unwrap();

            let t = controller.borrow().get_transform();

            if let Some(commands) = controller.borrow().draw_commands_for_current_shape() {
                for command in commands {
                    command.draw(ctx, t);
                }
            }

            for command in controller.borrow().draw_commands_for_tool() {
                command.draw(ctx, t);
            }

            Inhibit(false)
        }),
    );

    drawing_area.connect_key_press_event(clone!(@strong controller => move |_dw, event| {
        if let Some(key_name) = event.keyval().name() {
            let key = gtk_key(key_name.as_str());

            controller.borrow_mut().handle_key_pressed(key);
        }

        Inhibit(false)
    }));

    drawing_area.connect_key_release_event(
        clone!(@strong controller, @strong surface => move |dw, event| {
            if let Some(key_name) = event.keyval().name() {
                let key = gtk_key(key_name.as_str());
                let redraw = controller.borrow_mut().handle_key_released(key);

                if let ShouldRedraw::All = redraw {
                    invalidate_and_redraw(&controller.borrow(), &surface, dw);
                }
            }

            Inhibit(false)
        }),
    );

    drawing_area.connect_scroll_event(
        clone!(@strong controller, @strong surface => move |dw, event| {
            let delta = event.scroll_deltas().unwrap_or_else(|| event.delta());

            controller.borrow_mut().scroll(delta.into(), gtk_flags(event.state()));

            invalidate_and_redraw(&controller.borrow(), &surface, dw);

            Inhibit(false)
        }),
    );

    drawing_area.connect_button_press_event(
        clone!(@strong controller, @strong surface => move |dw, event| {
            if let EventType::ButtonPress = event.event_type() {
                let redraw_hint = controller
                    .borrow_mut()
                    .handle_mouse_button_pressed_flags(
                        gtk_button(event.button()),
                        Vec2D::from(event.position()),
                        event.device_tool().and_then(|dt| gtk_tool(dt.tool_type())),
                    );

                match redraw_hint {
                    ShouldRedraw::All => {
                        invalidate_and_redraw(&controller.borrow(), &surface, dw);
                    }
                    ShouldRedraw::Shape => {
                        dw.queue_draw();
                    }
                    _ => {}
                }
            }

            Inhibit(false)
        }),
    );

    drawing_area.connect_button_release_event(
        clone!(@strong controller, @strong surface, @strong header_bar => move |dw, event| {
            if let EventType::ButtonRelease = event.event_type() {
                let redraw_hint = controller
                    .borrow_mut()
                    .handle_mouse_button_released_flags(
                        gtk_button(event.button()),
                        Vec2D::from(event.position()),
                        gtk_flags(event.state()),
                        event.device_tool().and_then(|dt| gtk_tool(dt.tool_type())),
                    );

                match redraw_hint {
                    ShouldRedraw::All => {
                        invalidate_and_redraw(&controller.borrow(), &surface, dw);
                    }
                    ShouldRedraw::Shape => {
                        dw.queue_draw();
                    }
                    _ => {}
                }
            }

            set_subtitle(&header_bar, controller.borrow().get_save_status());

            Inhibit(false)
        }),
    );

    drawing_area.connect_motion_notify_event(
        clone!(@strong controller, @strong surface => move |dw, event| {
            let (x, y) = event.position();

            let redraw_hint = controller
                .borrow_mut()
                .handle_mouse_move_flags(
                    Vec2D::new_screen(x, y),
                    gtk_flags(event.state()),
                    event.device_tool().and_then(|dt| gtk_tool(dt.tool_type())),
                );

            match redraw_hint {
                ShouldRedraw::All => {
                    invalidate_and_redraw(&controller.borrow(), &surface, dw);
                }
                ShouldRedraw::Shape => {
                    dw.queue_draw();
                }
                _ => {
                    dw.queue_draw();
                }
            }

            Inhibit(false)
        }),
    );

    drawing_area.connect_size_allocate(clone!(@strong controller, @strong surface => move |dw, allocation| {
        controller.borrow_mut().resize(Vec2D::new_screen(allocation.width() as f64, allocation.height() as f64));
        invalidate_and_redraw(&controller.borrow(), &surface, dw);
    }));

    let dwb = Rc::new(RefCell::new(drawing_area));

    // Color chooser
    let color_chooser: ColorButton = builder.object("color-chooser").expect("No color chooser");

    color_chooser.connect_color_set(clone!(@strong controller, @strong dwb => move |chooser| {
        let rgba = chooser.rgba();
        let prev_alpha = controller.borrow().selected_color().alpha();
        controller.borrow_mut().set_color(Color::from_float_rgb(rgba.red(), rgba.green(), rgba.blue()).with_alpha(prev_alpha));
        dwb.borrow().queue_draw();
    }));

    // Zoom buttons
    let zoom_in_btn: Button = builder.object("zoom-in-btn").expect("No zoom in btn");
    zoom_in_btn.connect_clicked(
        clone!(@strong controller, @strong dwb, @strong surface => move |_btn| {
            controller.borrow_mut().zoom_in();
            invalidate_and_redraw(&controller.borrow(), &surface, &dwb.borrow());
        }),
    );

    let zoom_out_btn: Button = builder.object("zoom-out-btn").expect("No zoom out btn");
    zoom_out_btn.connect_clicked(
        clone!(@strong controller, @strong dwb, @strong surface => move |_btn| {
            controller.borrow_mut().zoom_out();
            invalidate_and_redraw(&controller.borrow(), &surface, &dwb.borrow());
        }),
    );

    let zoom_home_btn: Button = builder.object("zoom-home-btn").expect("No zoom home btn");
    zoom_home_btn.connect_clicked(
        clone!(@strong controller, @strong dwb, @strong surface => move |_btn| {
            controller.borrow_mut().go_home();
            invalidate_and_redraw(&controller.borrow(), &surface, &dwb.borrow());
        }),
    );

    // Thickness and alpha
    let thickness_btn: ScaleButton = builder.object("thickness-scale").unwrap();
    thickness_btn.connect_value_changed(clone!(@strong controller => move |_btn, value| {
        controller.borrow_mut().set_stroke(value.into());
    }));

    let alpha_btn: ScaleButton = builder.object("alpha-scale").unwrap();
    alpha_btn.connect_value_changed(clone!(@strong controller => move |_btn, value| {
        controller.borrow_mut().set_alpha((value * 255.0) as u8);
    }));

    // Undo/Redo
    let undo_menu: MenuItem = builder.object("undo-btn").expect("No undo btn");
    undo_menu.connect_activate(clone!(@strong controller, @strong header_bar, @strong dwb, @strong surface => move |_menu| {
        controller.borrow_mut().undo();
        invalidate_and_redraw(&controller.borrow(), &surface, &dwb.borrow());
        set_subtitle(&header_bar, controller.borrow().get_save_status());
    }));

    let redo_menu: MenuItem = builder.object("redo-btn").expect("No reundo btn");
    redo_menu.connect_activate(clone!(@strong controller, @strong header_bar, @strong dwb, @strong surface => move |_menu| {
        controller.borrow_mut().redo();
        invalidate_and_redraw(&controller.borrow(), &surface, &dwb.borrow());
        set_subtitle(&header_bar, controller.borrow().get_save_status());
    }));

    // File management
    let unsaved_changes_new_file = unsaved_changes_new_file_const.to_string();
    let unsaved_changes_since_last_time = unsaved_changes_since_last_time_const.to_string();
    let open_menu: MenuItem = builder.object("open-btn").expect("no open menu");
    open_menu.connect_activate(clone!(@strong controller, @strong header_bar, @strong window, @strong dwb, @strong surface => move |_menu| {
        let save_status = controller.borrow().get_save_status().clone();

        match save_status {
            SaveStatus::NewAndEmpty => open_logic(&window, &header_bar, controller.clone(), surface.clone(), dwb.clone()),
            SaveStatus::NewAndChanged => {
                yes_no_cancel_dialog(&window, &unsaved_changes_new_file, clone!(@strong controller, @strong header_bar, @strong window, @strong dwb, @strong surface => move || {
                    if save_as_with_error_dialog(&window, &header_bar, controller.clone()).is_ok() {
                        open_logic(&window, &header_bar, controller.clone(), surface.clone(), dwb.clone());
                        Inhibit(false)
                    } else {
                        Inhibit(true)
                    }
                }), clone!(@strong controller, @strong header_bar, @strong window, @strong dwb, @strong surface => move || {
                    open_logic(&window, &header_bar, controller.clone(), surface.clone(), dwb.clone());
                    Inhibit(false)
                }), || {
                    Inhibit(false)
                });
            },
            SaveStatus::Unsaved(path) => {
                yes_no_cancel_dialog(&window, &unsaved_changes_since_last_time, clone!(@strong controller, @strong header_bar, @strong window, @strong dwb, @strong surface => move || {
                    if let Inhibit(false) = save_to_svg_logic_with_error_dialg(&window, controller.clone(), &path) {
                        open_logic(&window, &header_bar, controller.clone(), surface.clone(), dwb.clone());
                        Inhibit(false)
                    } else {
                        Inhibit(true)
                    }
                }), || {
                    Inhibit(false)
                }, || {
                    Inhibit(false)
                });
            },
            SaveStatus::Saved(_path) => open_logic(&window, &header_bar, controller.clone(), surface.clone(), dwb.clone()),
        }
    }));

    let new_menu: MenuItem = builder.object("new-btn").expect("no new menu");
    let unsaved_changes_new_file = unsaved_changes_new_file_const.to_string();
    let unsaved_changes_since_last_time = unsaved_changes_since_last_time_const.to_string();
    new_menu.connect_activate(clone!(@strong controller, @strong header_bar, @strong window, @strong dwb, @strong surface => move |_menu| {
        let save_status = controller.borrow().get_save_status().clone();

        // TODO reconsider how this works. I think the existence of the reset()
        // method is broken by default and instead we should replace the
        // controller instance with a new one.
        //
        // Also this method should probably instead just launch a new window in
        // some cases

        match save_status {
            SaveStatus::NewAndEmpty => {},
            SaveStatus::NewAndChanged => {
                yes_no_cancel_dialog(&window, &unsaved_changes_new_file.to_string(), || {
                    if save_as_with_error_dialog(&window, &header_bar, controller.clone()).is_ok() {
                        controller.borrow_mut().reset();
                        invalidate_and_redraw(&controller.borrow(), &surface, &dwb.borrow());
                        set_subtitle(&header_bar, controller.borrow().get_save_status());
                        Inhibit(false)
                    } else {
                        Inhibit(true)
                    }
                }, || {
                    controller.borrow_mut().reset();
                    invalidate_and_redraw(&controller.borrow(), &surface, &dwb.borrow());
                    set_subtitle(&header_bar, controller.borrow().get_save_status());
                    Inhibit(false)
                }, || {
                    Inhibit(false)
                });
            },
            SaveStatus::Unsaved(path) => {
                yes_no_cancel_dialog(&window, &unsaved_changes_since_last_time.to_string(), || {
                    if let Inhibit(false) = save_to_svg_logic_with_error_dialg(&window, controller.clone(), &path) {
                        controller.borrow_mut().reset();
                        invalidate_and_redraw(&controller.borrow(), &surface, &dwb.borrow());
                        set_subtitle(&header_bar, controller.borrow().get_save_status());
                        Inhibit(false)
                    } else {
                        Inhibit(true)
                    }
                }, || {
                    controller.borrow_mut().reset();
                    invalidate_and_redraw(&controller.borrow(), &surface, &dwb.borrow());
                    set_subtitle(&header_bar, controller.borrow().get_save_status());
                    Inhibit(false)
                }, || {
                    Inhibit(false)
                });
            },
            SaveStatus::Saved(_path) => {
                controller.borrow_mut().reset();
                invalidate_and_redraw(&controller.borrow(), &surface, &dwb.borrow());
                set_subtitle(&header_bar, controller.borrow().get_save_status());
            },
        }
    }));

    let save_menu: MenuItem = builder.object("save-btn").expect("no save menu");
    save_menu.connect_activate(clone!(@strong controller, @strong header_bar, @strong window => move |_menu| {
        let save_status = controller.borrow().get_save_status().clone();

        match save_status {
            SaveStatus::NewAndEmpty => {}, // nothing to save actually
            SaveStatus::NewAndChanged => {
                save_as_with_error_dialog(&window, &header_bar, controller.clone()).ok();
            },
            SaveStatus::Unsaved(path) => {
                let inhibit = save_to_svg_logic_with_error_dialg(&window, controller.clone(), &path);

                if inhibit == Inhibit(false) {
                    controller.borrow_mut().set_saved(path);
                    set_subtitle(&header_bar, controller.borrow().get_save_status());
                }
            },
            SaveStatus::Saved(_path) => {},
        }
    }));

    let save_as_menu: MenuItem = builder.object("save-as-btn").expect("no save as menu");
    save_as_menu.connect_activate(
        clone!(@strong controller, @strong header_bar, @strong window => move |_menu| {
            let save_status = controller.borrow().get_save_status().clone();

            match save_status {
                SaveStatus::NewAndEmpty => {},
                SaveStatus::NewAndChanged => {
                    save_as_with_error_dialog(&window, &header_bar, controller.clone()).ok();
                },
                SaveStatus::Unsaved(_path) => {
                    save_as_with_error_dialog(&window, &header_bar, controller.clone()).ok();
                },
                SaveStatus::Saved(_path) => {
                    save_as_with_error_dialog(&window, &header_bar, controller.clone()).ok();
                },
            }
        }),
    );

    let export_menu: MenuItem = builder.object("export-btn").expect("no export menu");
    export_menu.connect_activate(clone!(@strong controller, @strong window => move |_menu| {
        export_logic(&window, controller.clone());
    }));

    let exit_menu: MenuItem = builder.object("exit-btn").expect("no save menu");
    exit_menu.connect_activate(clone!(@strong window => move |_menu| {
        window.close();
    }));

    // Change shape
    let set_pen_menu: MenuItem = builder.object("tool-pen-btn").expect("no pen menu");
    set_pen_menu.connect_activate(clone!(@strong controller, @strong tool_btn => move |_menu| {
        controller.borrow_mut().set_tool(SelectedTool::Shape(ShapeTool::Path));
        tool_btn.set_image(Some(&Image::from_resource("/tk/categulario/pizarra/icons/line.svg")));
    }));

    let set_rectangle_menu: MenuItem = builder.object("tool-rect-btn").expect("no ractangle menu");
    set_rectangle_menu.connect_activate(clone!(@strong controller, @strong tool_btn => move |_menu| {
        controller.borrow_mut().set_tool(SelectedTool::Shape(ShapeTool::Rectangle));
        tool_btn.set_image(Some(&Image::from_resource("/tk/categulario/pizarra/icons/rectangle.svg")));
    }));

    let set_polygon_menu: MenuItem = builder.object("tool-polygon-btn").expect("no polygon menu");
    set_polygon_menu.connect_activate(clone!(@strong controller, @strong tool_btn => move |_menu| {
        controller.borrow_mut().set_tool(SelectedTool::Shape(ShapeTool::Polygon));
        tool_btn.set_image(Some(&Image::from_resource("/tk/categulario/pizarra/icons/polygon.svg")));
    }));

    let set_circle_menu: MenuItem = builder.object("tool-circle-btn").expect("no circle menu");
    set_circle_menu.connect_activate(clone!(@strong controller, @strong tool_btn => move |_menu| {
        controller.borrow_mut().set_tool(SelectedTool::Shape(ShapeTool::CircleByCenterAndPoint));
        tool_btn.set_image(Some(&Image::from_resource("/tk/categulario/pizarra/icons/circle_by_center_and_point.svg")));
    }));

    let set_circle_by_three_points: MenuItem =
        builder.object("tool-circle3-btn").expect("no circle menu");
    set_circle_by_three_points.connect_activate(clone!(@strong controller, @strong tool_btn => move |_menu| {
        controller.borrow_mut().set_tool(SelectedTool::Shape(ShapeTool::CircleThroughThreePoints));
        tool_btn.set_image(Some(&Image::from_resource("/tk/categulario/pizarra/icons/circle_by_three_points.svg")));
    }));

    let set_ellipse_menu: MenuItem = builder.object("tool-ellipse-btn").expect("no ellipse menu");
    set_ellipse_menu.connect_activate(clone!(@strong controller, @strong tool_btn => move |_menu| {
        controller.borrow_mut().set_tool(SelectedTool::Shape(ShapeTool::ThreePointEllipse));
        tool_btn.set_image(Some(&Image::from_resource("/tk/categulario/pizarra/icons/ellipse_by_foci_and_point.svg")));
    }));

    let set_grid_menu: MenuItem = builder.object("tool-grid-btn").expect("no grid menu");
    set_grid_menu.connect_activate(clone!(@strong controller, @strong tool_btn => move |_menu| {
        controller.borrow_mut().set_tool(SelectedTool::Shape(ShapeTool::Grid));
        tool_btn.set_image(Some(&Image::from_resource("/tk/categulario/pizarra/icons/grid.svg")));
    }));

    let set_free_grid_menu: MenuItem = builder
        .object("tool-free-grid-btn")
        .expect("no free grid menu");
    set_free_grid_menu.connect_activate(clone!(@strong controller, @strong tool_btn => move |_menu| {
        controller.borrow_mut().set_tool(SelectedTool::Shape(ShapeTool::FreeGrid));
        tool_btn.set_image(Some(&Image::from_resource("/tk/categulario/pizarra/icons/free_grid.svg")));
    }));

    let set_eraser_menu: MenuItem = builder.object("tool-eraser-btn").expect("no eraser menu");
    set_eraser_menu.connect_activate(clone!(@strong controller, @strong tool_btn => move |_menu| {
        controller.borrow_mut().set_tool(SelectedTool::Eraser);
        tool_btn.set_image(Some(&Image::from_resource("/tk/categulario/pizarra/icons/eraser.svg")));
    }));

    let about_btn: MenuItem = builder.object("about-btn").unwrap();
    about_btn.connect_activate(move |_| {
        about_dialog.set_version(Some(env!("CARGO_PKG_VERSION")));
        let response = about_dialog.run();
        if response == ResponseType::DeleteEvent || response == ResponseType::Cancel {
            about_dialog.hide();
        }
    });

    // Show
    window.show_all();
}

fn pre_launch(app: &Application, file: Option<PathBuf>) {
    let resource_bytes = include_bytes!(concat!(env!("OUT_DIR"), "/res/resources.gresource"));
    let resource_data = gtk::glib::Bytes::from(&resource_bytes[..]);
    gio::resources_register(&gio::Resource::from_data(&resource_data).unwrap());

    let icon_theme = gtk::IconTheme::default().expect("failed to get default icon theme");
    icon_theme.add_resource_path("/tk/categulario/pizarra/icons");

    Window::set_default_icon_name("tk.categulario.pizarra");

    init(app, file);
}

fn main() {
    #[cfg(not(windows))]
    env_logger::init();

    let application = Application::new(
        Some("tk.categulario.pizarra"),
        ApplicationFlags::NON_UNIQUE | ApplicationFlags::HANDLES_OPEN,
    );

    application.connect_activate(move |app| {
        pre_launch(app, None);
    });

    application.connect_open(move |app, files, _hint| {
        pre_launch(app, files.first().and_then(|f| f.path()));
    });

    application.run();
}
