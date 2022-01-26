use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::rc::Rc;
use std::cell::RefCell;

use gtk::{
    ApplicationWindow, DrawingArea, FileChooserNative, FileChooserAction,
    ResponseType, HeaderBar, MessageDialog, DialogFlags, MessageType,
    ButtonsType, Window,
};
use gtk::prelude::*;
use cairo::{ImageSurface, Context};

use pizarra::prelude::*;

use crate::graphics::Drawable;

/// Padding in pixels around the bbox of the drawing when exporting or saving
const RENDER_PADDING: f64 = 20.0;

fn ensure_extension(filename: &Path, extension: &str) -> PathBuf {
    if let Some(ext) = filename.extension() {
        if ext != extension {
            filename.with_extension(extension)
        } else {
            filename.into()
        }
    } else {
        filename.with_extension(extension)
    }
}

pub fn set_subtitle(header_bar: &HeaderBar, save_status: &SaveStatus) {
    match save_status {
        SaveStatus::NewAndEmpty => {
            header_bar.set_title(Some("Pizarra"));
            header_bar.set_subtitle(None);
        },
        SaveStatus::NewAndChanged => {
            header_bar.set_title(Some("*Dibujo sin guardar"));
            header_bar.set_subtitle(None);
        },
        SaveStatus::Unsaved(path) => {
            header_bar.set_title(Some(&format!("*{}", path.file_name().unwrap().to_string_lossy())));
            header_bar.set_subtitle(Some(&format!("{}", path.parent().unwrap().display())));
        },
        SaveStatus::Saved(path) => {
            header_bar.set_title(Some(&format!("{}", path.file_name().unwrap().to_string_lossy())));
            header_bar.set_subtitle(Some(&format!("{}", path.parent().unwrap().display())));
        },
    }
}

fn dialog(window: &ApplicationWindow, message: &str, msg_type: MessageType) {
    let message_dialog = MessageDialog::new(
        Some(window),
        DialogFlags::DESTROY_WITH_PARENT,
        msg_type,
        ButtonsType::None,
        message,
    );

    message_dialog.add_button("chales", ResponseType::Ok);

    message_dialog.run();
    message_dialog.hide();
}

fn save_to_svg_logic(controller: Rc<RefCell<Pizarra>>, filename: &Path) -> std::io::Result<()> {
    let svg_data = controller.borrow_mut().to_svg();
    let svgfilename = ensure_extension(filename, "svg");
    let mut svgfile = File::create(&svgfilename)?;

    svgfile.write_all(svg_data.as_bytes())?;

    controller.borrow_mut().set_saved(svgfilename);

    Ok(())
}

pub fn save_to_svg_logic_with_error_dialg(window: &ApplicationWindow, controller: Rc<RefCell<Pizarra>>, filename: &Path) -> Inhibit {
    match save_to_svg_logic(controller, filename) {
        Ok(_) => Inhibit(false),
        Err(e) => {
            dialog(window, &format!("Falló esto:\n\n{}", e), MessageType::Error);

            Inhibit(true)
        }
    }
}

/// Implements the logic of the _save-as_ feature
fn save_as_logic(window: &ApplicationWindow, header_bar: &HeaderBar, controller: Rc<RefCell<Pizarra>>) -> std::io::Result<()>
{
    let save_file_chooser = FileChooserNative::new(Some("Guardar"), Some(window), FileChooserAction::Save, Some("Guardar"), Some("Cancelar"));
    let res = save_file_chooser.run();

    if res == ResponseType::Accept {
        if let Some(filename) = save_file_chooser.filename() {
            save_to_svg_logic(controller.clone(), &filename)?;
            set_subtitle(header_bar, controller.borrow().get_save_status());
        }
    }

    Ok(())
}

pub fn save_as_with_error_dialog(window: &ApplicationWindow, header_bar: &HeaderBar, controller: Rc<RefCell<Pizarra>>) -> Result<(), ()>
{
    match save_as_logic(window, header_bar, controller) {
        Ok(_) => Ok(()),
        Err(e) => {
            dialog(window, &format!("Falló esto:\n\n{}", e), MessageType::Error);

            Err(())
        }
    }
}

/// Logic of the open dialog
pub fn open_logic(window: &ApplicationWindow, header_bar: &HeaderBar, controller: Rc<RefCell<Pizarra>>, surface: Rc<RefCell<ImageSurface>>, dwb: Rc<RefCell<DrawingArea>>) {
    let open_file_chooser = FileChooserNative::new(Some("Abrir"), Some(window), FileChooserAction::Open, Some("Abrir"), Some("Cancelar"));
    let res = open_file_chooser.run();

    if res == ResponseType::Accept {
        if let Some(filename) = open_file_chooser.filename() {
            if let Ok(mut file) = File::open(&filename) {
                let mut svg = String::new();

                if file.read_to_string(&mut svg).is_ok() {
                    let ans = { controller.borrow_mut().open(&svg) };
                    if ans.is_ok() {
                        controller.borrow_mut().set_saved(filename);
                        invalidate_and_redraw(&controller.borrow(), &surface, &dwb.borrow());
                        set_subtitle(header_bar, controller.borrow().get_save_status());
                    } else {
                        dialog(window, "No pude interpretar el formato de este archivo :(", MessageType::Error);
                    }
                } else {
                    dialog(window, "No pude leer los contenidos del archivo :(", MessageType::Error);
                }
            } else {
                dialog(window, "Ese archivo no existe :(", MessageType::Error);
            }
        }
    }
}

/// Implements the logic of the export feature
pub fn export_logic<P: IsA<Window>>(window: &P, controller: Rc<RefCell<Pizarra>>) {
    let export_file_chooser = FileChooserNative::new(Some("Exportar"), Some(window), FileChooserAction::Save, Some("Exportar"), Some("Cancelar"));
    let res = export_file_chooser.run();

    if res == ResponseType::Accept {
        if let Some(filename) = export_file_chooser.filename() {
            if let Some([topleft, bottomright]) = controller.borrow().get_bounds() {
                let pngfilename = ensure_extension(&filename, "png");
                let width = (bottomright.x - topleft.x).abs() + 2.0 * RENDER_PADDING;
                let height = (bottomright.y - topleft.y).abs() + 2.0 * RENDER_PADDING;
                let surface = ImageSurface::create(cairo::Format::ARgb32, width as i32, height as i32).unwrap();
                let context = cairo::Context::new(&surface).unwrap();

                render_drawing(&controller.borrow(), &context, topleft);

                surface.write_to_png(&mut File::create(pngfilename).unwrap()).unwrap();
            }
        }
    }
}

/// Redraws the visible portion of the screen from the stored shapes, not
/// including the shape being drawn.
///
/// Called on translate or rotate but not during the drawing phase of a new
/// shape
pub fn invalidate_and_redraw(controller: &Pizarra, surface: &RefCell<ImageSurface>, dw: &DrawingArea) {
    let t = controller.get_transform();
    let commands = controller.draw_commands_for_screen();
    let p = controller.get_dimensions();

    let width = p.x.val();
    let height = p.y.val();

    let new_surface = ImageSurface::create(cairo::Format::ARgb32, width as i32, height as i32).unwrap();
    let context = cairo::Context::new(&new_surface).unwrap();

    let bgcolor = controller.bgcolor();

    context.set_source_rgb(bgcolor.float_r(), bgcolor.float_g(), bgcolor.float_b());
    context.paint().unwrap();

    // content
    for cmd in commands {
        cmd.draw(&context, t);
    }

    surface.replace(new_surface);

    dw.queue_draw();
}

/// Renders the entire drawing to a cairo context. Used for exporting to png and
/// potentially other formats.
fn render_drawing(controller: &Pizarra, ctx: &Context, topleft: Vec2DWorld) {
    let t = Transform::new_translate(
        ((topleft - Vec2DWorld::new(RENDER_PADDING, RENDER_PADDING)) * -1.0).to_vec2d()
    );
    let bgcolor = controller.bgcolor();

    ctx.set_source_rgb(bgcolor.float_r(), bgcolor.float_g(), bgcolor.float_b());
    ctx.paint().unwrap();

    for cmd in controller.draw_commands_for_drawing() {
        cmd.draw(ctx, t);
    }
}
