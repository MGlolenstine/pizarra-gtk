#![windows_subsystem = "windows"]
use std::rc::Rc;
use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::env;

use gtk::{
    Application, ApplicationWindow, DrawingArea, Builder, ColorButton,
    Button, MenuItem, FileChooserNative, FileChooserAction, ResponseType,
    HeaderBar, MessageDialog, DialogFlags, MessageType, ButtonsType, Window,
    ScaleButton, AboutDialog, Image,
};
use gdk::{EventMask, EventType, ScrollDirection};
use gtk::prelude::*;
use gio::prelude::*;
use gio::ApplicationFlags;
use glib::clone;
use cairo::ImageSurface;
use simple_logger::SimpleLogger;

use pizarra::prelude::*;

mod graphics;
mod logic;

use graphics::Drawable;
use logic::*;

const UNSAVED_CHANGES_SINCE_LAST_TIME: &'static str = "Hiciste algunos trazos desde la última vez\n\n¿Los quieres guardar?";
const UNSAVED_CHANGES_NEW_FILE: &'static str = "Hay algunos trazos aquí\n\n¿Los quieres guardar?";

fn yes_no_cancel_dialog<F, G, H>(window: &ApplicationWindow, message: &str, yes_callback: F, no_callback: G, cancel_callback: H) -> Inhibit
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
    message_dialog.add_button("Sí", ResponseType::Yes);
    message_dialog.add_button("No", ResponseType::No);
    message_dialog.add_button("Cancelar", ResponseType::Cancel);

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
        _ => Key::Unknown,
    }
}

fn init(app: &Application, filename: Option<PathBuf>) {
    // Initialize layout from .glade file
    let builder = Builder::from_resource("/tk/categulario/pizarra/pizarra.glade");
    let controller = Rc::new(RefCell::new(Pizarra::new(Vec2DScreen::new(1.0, 1.0))));
    let window: ApplicationWindow = builder.get_object("main-window").expect("Couldn't get window");
    let header_bar: HeaderBar = builder.get_object("header-bar").expect("no header bar");
    let surface = Rc::new(RefCell::new(ImageSurface::create(cairo::Format::ARgb32, 1, 1).unwrap()));
    let about_dialog: AboutDialog = builder.get_object("about-dialog").unwrap();
    let tool_btn: Button = builder.get_object("tool-menu-btn").unwrap();

    window.set_application(Some(app));

    if let Some(filename) = filename {
        let mut svg = String::new();
        let mut file = File::open(&filename).expect("Could not open given file");

        file.read_to_string(&mut svg).expect("could not read file contents");
        {
            let mut controller = controller.borrow_mut();
            controller.open(&svg).expect("Could not parse given file");
            controller.set_saved(filename);
        }
        set_subtitle(&header_bar, controller.borrow().get_save_status());
    }

    // save on exit
    window.connect_delete_event(clone!(@strong controller, @strong window => move |_window, _event| {
        let old_save_status = {
            controller.borrow().get_save_status().clone()
        };

        match old_save_status {
            SaveStatus::NewAndChanged => yes_no_cancel_dialog(&window, "Ya hiciste algunos dibujos\n\n¿Deseas guardar?", || {
                let save_file_chooser = FileChooserNative::new(Some("Guardar"), Some(&window), FileChooserAction::Save, Some("Guardar"), Some("Cancelar"));
                let res = save_file_chooser.run();

                match res {
                    ResponseType::Accept => {
                        if let Some(filename) = save_file_chooser.get_filename() {
                            save_to_svg_logic_with_inhibit(&window, controller.clone(), &filename)
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
            SaveStatus::Unsaved(path) => yes_no_cancel_dialog(&window, "Hay cambios desde la última vez que guardaste\n\n¿Quieres guardarlos?", || {
                    save_to_svg_logic_with_inhibit(&window, controller.clone(), &path)
                }, || {
                    Inhibit(false)
                }, || {
                    Inhibit(true)
                }),
        }
    }));

    // Drawing area
    let drawing_area: DrawingArea = builder.get_object("drawing-area").expect("No drawing_area");

    let event_mask = EventMask::POINTER_MOTION_MASK
        | EventMask::BUTTON_PRESS_MASK | EventMask::BUTTON_RELEASE_MASK
        | EventMask::KEY_PRESS_MASK | EventMask::KEY_RELEASE_MASK
        | EventMask::POINTER_MOTION_MASK | EventMask::TABLET_PAD_MASK
        | EventMask::SCROLL_MASK;

    drawing_area.set_can_focus(true);
    drawing_area.add_events(event_mask);

    drawing_area.connect_draw(clone!(@strong controller, @strong surface => move |_dw, ctx| {
        ctx.set_source_surface(&surface.borrow(), 0.0, 0.0);
        ctx.paint();

        if let Some(commands) = controller.borrow().draw_commands_for_current_shape() {
            let t = controller.borrow().get_transform();

            for command in commands {
                command.draw(&ctx, t);
            }
        }

        Inhibit(false)
    }));

    drawing_area.connect_key_press_event(clone!(@strong controller => move |_dw, event| {
        if let Some(key_name) = event.get_keyval().name() {
            let key = gtk_key(key_name.as_str());

            controller.borrow_mut().handle_key_pressed(key);
        }

        Inhibit(false)
    }));

    drawing_area.connect_key_release_event(clone!(@strong controller => move |_dw, event| {
        if let Some(key_name) = event.get_keyval().name() {
            let key = gtk_key(key_name.as_str());

            controller.borrow_mut().handle_key_released(key);
        }

        Inhibit(false)
    }));

    drawing_area.connect_scroll_event(clone!(@strong controller, @strong surface => move |dw, event| {
        if let Some(direction) = event.get_scroll_direction() {
            match direction {
                ScrollDirection::Up => {
                    controller.borrow_mut().scroll_up();
                },
                ScrollDirection::Down => {
                    controller.borrow_mut().scroll_down();
                },
                ScrollDirection::Left => {
                    controller.borrow_mut().scroll_left();
                },
                ScrollDirection::Right => {
                    controller.borrow_mut().scroll_right();
                },
                _ => {},
            }

            invalidate_and_redraw(&controller.borrow(), &surface, dw);
        }

        Inhibit(false)
    }));

    drawing_area.connect_button_press_event(clone!(@strong controller, @strong surface => move |dw, event| {
        if let EventType::ButtonPress = event.get_event_type() {
            let redraw_hint = controller
                .borrow_mut()
                .handle_mouse_button_pressed(
                    gtk_button(event.get_button()),
                    Vec2DScreen::from(event.get_position())
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
    }));

    drawing_area.connect_button_release_event(clone!(@strong controller, @strong surface, @strong header_bar => move |dw, event| {
        if let EventType::ButtonRelease = event.get_event_type() {
            let redraw_hint = controller
                .borrow_mut()
                .handle_mouse_button_released(
                    gtk_button(event.get_button()),
                    Vec2DScreen::from(event.get_position())
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
    }));

    drawing_area.connect_motion_notify_event(clone!(@strong controller, @strong surface => move |dw, event| {
        let (x, y) = event.get_position();

        let redraw_hint = controller
            .borrow_mut()
            .handle_mouse_move(Vec2DScreen::new(x, y));

        match redraw_hint {
            ShouldRedraw::All => {
                invalidate_and_redraw(&controller.borrow(), &surface, dw);
            }
            ShouldRedraw::Shape => {
                dw.queue_draw();
            }
            _ => {}
        }

        Inhibit(false)
    }));

    drawing_area.connect_size_allocate(clone!(@strong controller, @strong surface => move |dw, allocation| {
        controller.borrow_mut().resize(Vec2DScreen::new(allocation.width as f64, allocation.height as f64));
        invalidate_and_redraw(&controller.borrow(), &surface, dw);
    }));

    let dwb = Rc::new(RefCell::new(drawing_area));

    // Color chooser
    let color_chooser: ColorButton = builder.get_object("color-chooser").expect("No color chooser");

    color_chooser.connect_color_set(clone!(@strong controller, @strong dwb => move |chooser| {
        let rgba = chooser.get_rgba();
        controller.borrow_mut().set_color(Color::from_rgba(rgba.red, rgba.green, rgba.blue, rgba.alpha));
        dwb.borrow().queue_draw();
    }));

    // Zoom buttons
    let zoom_in_btn: Button = builder.get_object("zoom-in-btn").expect("No zoom in btn");
    zoom_in_btn.connect_clicked(clone!(@strong controller, @strong dwb, @strong surface => move |_btn| {
        controller.borrow_mut().zoom_in();
        invalidate_and_redraw(&controller.borrow(), &surface, &dwb.borrow());
    }));

    let zoom_out_btn: Button = builder.get_object("zoom-out-btn").expect("No zoom out btn");
    zoom_out_btn.connect_clicked(clone!(@strong controller, @strong dwb, @strong surface => move |_btn| {
        controller.borrow_mut().zoom_out();
        invalidate_and_redraw(&controller.borrow(), &surface, &dwb.borrow());
    }));

    let zoom_home_btn: Button = builder.get_object("zoom-home-btn").expect("No zoom home btn");
    zoom_home_btn.connect_clicked(clone!(@strong controller, @strong dwb, @strong surface => move |_btn| {
        controller.borrow_mut().go_home();
        invalidate_and_redraw(&controller.borrow(), &surface, &dwb.borrow());
    }));

    // Thickness and alpha
    let thickness_btn: ScaleButton = builder.get_object("thickness-scale").unwrap();
    thickness_btn.connect_value_changed(clone!(@strong controller => move |_btn, value| {
        controller.borrow_mut().set_stroke(value);
    }));

    let alpha_btn: ScaleButton = builder.get_object("alpha-scale").unwrap();
    alpha_btn.connect_value_changed(clone!(@strong controller => move |_btn, value| {
        controller.borrow_mut().set_alpha(value);
    }));

    // Undo/Redo
    let undo_menu: MenuItem = builder.get_object("undo-btn").expect("No undo btn");
    undo_menu.connect_activate(clone!(@strong controller, @strong header_bar, @strong dwb, @strong surface => move |_menu| {
        controller.borrow_mut().undo();
        invalidate_and_redraw(&controller.borrow(), &surface, &dwb.borrow());
        set_subtitle(&header_bar, controller.borrow().get_save_status());
    }));

    let redo_menu: MenuItem = builder.get_object("redo-btn").expect("No reundo btn");
    redo_menu.connect_activate(clone!(@strong controller, @strong header_bar, @strong dwb, @strong surface => move |_menu| {
        controller.borrow_mut().redo();
        invalidate_and_redraw(&controller.borrow(), &surface, &dwb.borrow());
        set_subtitle(&header_bar, controller.borrow().get_save_status());
    }));

    // File management
    let open_menu: MenuItem = builder.get_object("open-btn").expect("no open menu");
    open_menu.connect_activate(clone!(@strong controller, @strong header_bar, @strong window, @strong dwb, @strong surface => move |_menu| {
        let save_status = controller.borrow().get_save_status().clone();

        match save_status {
            SaveStatus::NewAndEmpty => open_logic(&window, &header_bar, controller.clone(), surface.clone(), dwb.clone()),
            SaveStatus::NewAndChanged => {
                yes_no_cancel_dialog(&window, UNSAVED_CHANGES_NEW_FILE, clone!(@strong controller, @strong header_bar, @strong window, @strong dwb, @strong surface => move || {
                    save_as_logic(&window, &header_bar, controller.clone());
                    open_logic(&window, &header_bar, controller.clone(), surface.clone(), dwb.clone());
                    Inhibit(false)
                }), clone!(@strong controller, @strong header_bar, @strong window, @strong dwb, @strong surface => move || {
                    open_logic(&window, &header_bar, controller.clone(), surface.clone(), dwb.clone());
                    Inhibit(false)
                }), || {
                    Inhibit(false)
                });
            },
            SaveStatus::Unsaved(path) => {
                yes_no_cancel_dialog(&window, UNSAVED_CHANGES_SINCE_LAST_TIME, clone!(@strong controller, @strong header_bar, @strong window, @strong dwb, @strong surface => move || {
                    save_to_svg_logic(controller.clone(), &path);
                    open_logic(&window, &header_bar, controller.clone(), surface.clone(), dwb.clone());
                    Inhibit(false)
                }), || {
                    Inhibit(false)
                }, || {
                    Inhibit(false)
                });
            },
            SaveStatus::Saved(_path) => open_logic(&window, &header_bar, controller.clone(), surface.clone(), dwb.clone()),
        }
    }));

    let new_menu: MenuItem = builder.get_object("new-btn").expect("no new menu");
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
                yes_no_cancel_dialog(&window, UNSAVED_CHANGES_NEW_FILE, || {
                    save_as_logic(&window, &header_bar, controller.clone());
                    Inhibit(false)
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
                yes_no_cancel_dialog(&window, UNSAVED_CHANGES_SINCE_LAST_TIME, || {
                    save_to_svg_logic(controller.clone(), &path);
                    controller.borrow_mut().reset();
                    invalidate_and_redraw(&controller.borrow(), &surface, &dwb.borrow());
                    set_subtitle(&header_bar, controller.borrow().get_save_status());
                    Inhibit(false)
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

    let save_menu: MenuItem = builder.get_object("save-btn").expect("no save menu");
    save_menu.connect_activate(clone!(@strong controller, @strong header_bar, @strong window => move |_menu| {
        let save_status = controller.borrow().get_save_status().clone();

        match save_status {
            SaveStatus::NewAndEmpty => {}, // nothing to save actually
            SaveStatus::NewAndChanged => {
                save_as_logic(&window, &header_bar, controller.clone());
            },
            SaveStatus::Unsaved(path) => {
                save_to_svg_logic(controller.clone(), &path);
                controller.borrow_mut().set_saved(path.clone());
                set_subtitle(&header_bar, controller.borrow().get_save_status());
            },
            SaveStatus::Saved(_path) => {},
        }
    }));

    let save_as_menu: MenuItem = builder.get_object("save-as-btn").expect("no save as menu");
    save_as_menu.connect_activate(clone!(@strong controller, @strong header_bar, @strong window => move |_menu| {
        let save_status = controller.borrow().get_save_status().clone();

        match save_status {
            SaveStatus::NewAndEmpty => {},
            SaveStatus::NewAndChanged => {
                save_as_logic(&window, &header_bar, controller.clone());
            },
            SaveStatus::Unsaved(_path) => {
                save_as_logic(&window, &header_bar, controller.clone());
            },
            SaveStatus::Saved(_path) => {
                save_as_logic(&window, &header_bar, controller.clone());
            },
        }
    }));

    let export_menu: MenuItem = builder.get_object("export-btn").expect("no export menu");
    export_menu.connect_activate(clone!(@strong controller, @strong window => move |_menu| {
        export_logic(&window, controller.clone());
    }));

    // Change shape
    let set_pen_menu: MenuItem = builder.get_object("tool-pen-btn").expect("no pen menu");
    set_pen_menu.connect_activate(clone!(@strong controller, @strong tool_btn => move |_menu| {
        controller.borrow_mut().set_tool(SelectedTool::Shape(ShapeTool::Path));
        tool_btn.set_image(Some(&Image::from_resource("/tk/categulario/pizarra/icons/line.svg")));
    }));

    let set_rectangle_menu: MenuItem = builder.get_object("tool-rect-btn").expect("no ractangle menu");
    set_rectangle_menu.connect_activate(clone!(@strong controller, @strong tool_btn => move |_menu| {
        controller.borrow_mut().set_tool(SelectedTool::Shape(ShapeTool::Rectangle));
        tool_btn.set_image(Some(&Image::from_resource("/tk/categulario/pizarra/icons/rectangle.svg")));
    }));

    let set_polygon_menu: MenuItem = builder.get_object("tool-polygon-btn").expect("no polygon menu");
    set_polygon_menu.connect_activate(clone!(@strong controller, @strong tool_btn => move |_menu| {
        controller.borrow_mut().set_tool(SelectedTool::Shape(ShapeTool::Polygon));
        tool_btn.set_image(Some(&Image::from_resource("/tk/categulario/pizarra/icons/polygon.svg")));
    }));

    let set_circle_menu: MenuItem = builder.get_object("tool-circle-btn").expect("no circle menu");
    set_circle_menu.connect_activate(clone!(@strong controller, @strong tool_btn => move |_menu| {
        controller.borrow_mut().set_tool(SelectedTool::Shape(ShapeTool::CircleByCenterAndPoint));
        tool_btn.set_image(Some(&Image::from_resource("/tk/categulario/pizarra/icons/circle_by_center_and_point.svg")));
    }));

    let set_circle_by_three_points: MenuItem = builder.get_object("tool-circle3-btn").expect("no circle menu");
    set_circle_by_three_points.connect_activate(clone!(@strong controller, @strong tool_btn => move |_menu| {
        controller.borrow_mut().set_tool(SelectedTool::Shape(ShapeTool::CircleByThreePoints));
        tool_btn.set_image(Some(&Image::from_resource("/tk/categulario/pizarra/icons/circle_by_three_points.svg")));
    }));

    let set_ellipse_menu: MenuItem = builder.get_object("tool-ellipse-btn").expect("no ellipse menu");
    set_ellipse_menu.connect_activate(clone!(@strong controller, @strong tool_btn => move |_menu| {
        controller.borrow_mut().set_tool(SelectedTool::Shape(ShapeTool::ThreePointEllipse));
        tool_btn.set_image(Some(&Image::from_resource("/tk/categulario/pizarra/icons/ellipse_by_foci_and_point.svg")));
    }));

    let set_eraser_menu: MenuItem = builder.get_object("tool-eraser-btn").expect("no eraser menu");
    set_eraser_menu.connect_activate(clone!(@strong controller, @strong tool_btn => move |_menu| {
        controller.borrow_mut().set_tool(SelectedTool::Eraser);
        tool_btn.set_image(Some(&Image::from_resource("/tk/categulario/pizarra/icons/eraser.svg")));
    }));

    let about_btn: MenuItem = builder.get_object("about-btn").unwrap();
    about_btn.connect_activate(move |_| {
        let response = about_dialog.run();
        if response == ResponseType::DeleteEvent || response == ResponseType::Cancel {
            about_dialog.hide();
        }
    });

    // Show
    window.show_all();
}

fn main() {
    SimpleLogger::new().env().init().unwrap();

    let application = Application::new(
        Some("tk.categulario.pizarra"),
        ApplicationFlags::NON_UNIQUE,
    ).expect("failed to initialize GTK application");

    let arguments: Vec<_> = env::args().collect();

    application.connect_activate(move |app| {
        let resource_bytes = include_bytes!(concat!(env!("OUT_DIR"), "/res/resources.gresource"));
        let resource_data = glib::Bytes::from(&resource_bytes[..]);
        gio::resources_register(&gio::Resource::from_data(&resource_data).unwrap());

        let icon_theme = gtk::IconTheme::get_default().expect("failed to get default icon theme");
        icon_theme.add_resource_path("/tk/categulario/pizarra/icons");

        Window::set_default_icon_name("tk.categulario.pizarra");

        init(app, arguments.get(1).map(|f| f.into()));
    });

    application.run(&[]);
}
