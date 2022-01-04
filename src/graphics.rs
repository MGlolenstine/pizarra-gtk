use std::f64::consts::PI;

use cairo::{Context, LineCap, LineJoin, Matrix};

use pizarra::{
    draw_commands::{DrawCommand, Ellipse}, transform::Transform,
    path_command::{PathCommand, CubicBezierCurve},
    point::Point, style::Style,
};

fn draw_path<T: Point>(ctx: &Context, commands: &[PathCommand<T>], style: Style) {
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

    if let Some(s) = style.stroke {
        ctx.set_line_width(s.size);
        ctx.set_source_rgba(s.color.float_r(), s.color.float_g(), s.color.float_b(), s.color.float_alpha());
        ctx.set_line_cap(LineCap::Round);
        ctx.set_line_join(LineJoin::Round);
        ctx.stroke().unwrap();
    }
    if let Some(color) = style.fill {
        ctx.set_source_rgba(color.float_r(), color.float_g(), color.float_b(), color.float_alpha());
        ctx.fill().unwrap();
    }
}

fn draw_circle<T: Point>(ctx: &Context, center: T, radius: f64, style: Style) {
    ctx.arc(center.x(), center.y(), radius, 0.0, 2.0*PI);

    if let Some(s) = style.stroke {
        ctx.set_source_rgba(s.color.float_r(), s.color.float_g(), s.color.float_b(), s.color.float_alpha());
        ctx.set_line_width(s.size);
        ctx.stroke().unwrap();
    }
    if let Some(color) = style.fill {
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

    if let Some(s) = e.style.stroke {
        ctx.set_line_width(s.size);
        ctx.set_source_rgba(s.color.float_r(), s.color.float_g(), s.color.float_b(), s.color.float_alpha());
        ctx.stroke().unwrap();
    }
    if let Some(color) = e.style.fill {
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
            DrawCommand::Path { .. } | DrawCommand::Ellipse { .. } => {
                ctx.save().unwrap();
                ctx.transform(Matrix::new(t.xx, t.yx, t.xy, t.yy, t.x0, t.y0));
            }
            _ => {}
        }

        match self {
            DrawCommand::Path {
                commands, style,
            }  => {
                draw_path(ctx, commands, *style);
            }

            &DrawCommand::Ellipse(e) => {
                draw_ellipse(ctx, e);
            }

            DrawCommand::ScreenPath {
                commands, style,
            } => {
                draw_path(ctx, commands, *style);
            }

            &DrawCommand::ScreenCircle {
                center, radius, style,
            } => {
                draw_circle(ctx, center, radius, style);
            }
        }

        match &self {
            DrawCommand::Path { .. } | DrawCommand::Ellipse { .. } => {
                ctx.restore().unwrap();
            }
            _ => {}
        }
    }
}
