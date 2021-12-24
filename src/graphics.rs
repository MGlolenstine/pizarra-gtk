use std::f64::consts::PI;

use cairo::{Context, LineCap, LineJoin, Matrix};

use pizarra::{
    draw_commands::{DrawCommand, Ellipse}, transform::Transform,
    path_command::{PathCommand, CubicBezierCurve},
    point::Point, color::Color,
};

fn draw_path<T: Point>(ctx: &Context, color: Option<Color>, fill: Option<Color>, commands: &[PathCommand<T>], thickness: f64) {
    for point in commands.iter() {
        match *point {
            PathCommand::MoveTo(p) => {
                ctx.move_to(p.x(), p.y());
            },
            PathCommand::LineTo(p) => {
                ctx.line_to(p.x(), p.y());
            },
            PathCommand::CurveTo(CubicBezierCurve { pt1, pt2, to }) => {
                ctx.curve_to(pt1.x(), pt1.y(), pt2.x(), pt2.y(), to.x(), to.y());
            },
        }
    }

    if let Some(color) = color {
        ctx.set_line_width(thickness);
        ctx.set_source_rgba(color.float_r(), color.float_g(), color.float_b(), color.float_alpha());
        ctx.set_line_cap(LineCap::Round);
        ctx.set_line_join(LineJoin::Round);
        ctx.stroke().unwrap();
    }
    if let Some(color) = fill {
        ctx.set_source_rgba(color.float_r(), color.float_g(), color.float_b(), color.float_alpha());
        ctx.fill().unwrap();
    }
}

fn draw_circle<T: Point>(ctx: &Context, thickness: f64, center: T, radius: f64, color: Option<Color>, fill: Option<Color>) {
    ctx.arc(center.x(), center.y(), radius, 0.0, 2.0*PI);
    ctx.set_line_width(thickness);

    if let Some(color) = color {
        ctx.set_source_rgba(color.float_r(), color.float_g(), color.float_b(), color.float_alpha());
        ctx.stroke().unwrap();
    }
    if let Some(color) = fill {
        ctx.set_source_rgba(color.float_r(), color.float_g(), color.float_b(), color.float_alpha());
        ctx.fill().unwrap();
    }
}

fn draw_ellipse(ctx: &Context, e: Ellipse) {
    if e.semimajor == 0.0 || e.semiminor == 0.0 {
        return;
    }

    ctx.save().unwrap();
    ctx.translate(e.center.x, e.center.y);
    ctx.rotate(e.angle.radians());
    ctx.scale(e.semimajor, e.semiminor);
    ctx.arc(0., 0., 1., 0., 2.0 * PI);
    ctx.restore().unwrap();

    if let Some(color) = e.color {
        ctx.set_line_width(e.thickness);
        ctx.set_source_rgba(color.float_r(), color.float_g(), color.float_b(), color.float_alpha());
        ctx.stroke().unwrap();
    }
    if let Some(color) = e.fill {
        ctx.set_source_rgba(color.float_r(), color.float_g(), color.float_b(), color.float_alpha());
        ctx.fill().unwrap();
    }
}

pub trait Drawable {
    fn draw(&self, ctx: &Context, t: Transform);
}

impl Drawable for DrawCommand {
    fn draw(&self, ctx: &Context, t: Transform) {
        match self {
            DrawCommand::Path { .. } | DrawCommand::Circle { .. } | DrawCommand::Ellipse { .. } => {
                ctx.save().unwrap();
                ctx.transform(Matrix::new(t.xx, t.yx, t.xy, t.yy, t.x0, t.y0));
            }
            _ => {}
        }

        match self {
            DrawCommand::Path {
                color, fill, commands, thickness,
            }  => {
                draw_path(ctx, *color, *fill, commands, *thickness);
            }

            &DrawCommand::Circle {
                thickness, center, radius, color, fill,
            } => {
                draw_circle(ctx, thickness, center, radius, color, fill);
            }

            &DrawCommand::Ellipse(e) => {
                draw_ellipse(ctx, e);
            }

            DrawCommand::ScreenPath {
                color, fill, commands, thickness,
            } => {
                draw_path(ctx, *color, *fill, commands, *thickness);
            }

            &DrawCommand::ScreenCircle {
                thickness, center, radius, color, fill,
            } => {
                draw_circle(ctx, thickness, center, radius, color, fill);
            }
        }

        match &self {
            DrawCommand::Path { .. } | DrawCommand::Circle { .. } | DrawCommand::Ellipse { .. } => {
                ctx.restore().unwrap();
            }
            _ => {}
        }
    }
}
