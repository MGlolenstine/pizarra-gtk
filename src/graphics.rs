use std::f64::consts::PI;

use cairo::{Context, LineCap, LineJoin, Matrix};

use pizarra::{
    draw_commands::DrawCommand, transform::Transform,
    path_command::{PathCommand, CubicBezierCurve},
    point::{Vec2D, Unit, WorldUnit}, style::Style, geom::Ellipse,
};

fn draw_path<T: Unit>(ctx: &Context, commands: &[PathCommand<T>], style: Style<T>) {
    for point in commands.iter() {
        match *point {
            PathCommand::MoveTo(p) => {
                ctx.move_to(p.x.val(), p.y.val());
            },
            PathCommand::LineTo(p) => {
                ctx.line_to(p.x.val(), p.y.val());
            },
            PathCommand::CurveTo(CubicBezierCurve { pt1, pt2, to }) => {
                ctx.curve_to(pt1.x.val(), pt1.y.val(), pt2.x.val(), pt2.y.val(), to.x.val(), to.y.val());
            },
        }
    }

    if let Some(s) = style.stroke {
        ctx.set_line_width(s.size.val());
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

fn draw_circle<T: Unit>(ctx: &Context, center: Vec2D<T>, radius: T, style: Style<T>) {
    ctx.arc(center.x.val(), center.y.val(), radius.val(), 0.0, 2.0*PI);

    if let Some(s) = style.stroke {
        ctx.set_source_rgba(s.color.float_r(), s.color.float_g(), s.color.float_b(), s.color.float_alpha());
        ctx.set_line_width(s.size.val());
        ctx.stroke().unwrap();
    }
    if let Some(color) = style.fill {
        ctx.set_source_rgba(color.float_r(), color.float_g(), color.float_b(), color.float_alpha());
        ctx.fill().unwrap();
    }
}

fn draw_ellipse(ctx: &Context, e: Ellipse<WorldUnit>, style: Style<WorldUnit>) {
    if e.semimajor == 0.0.into() || e.semiminor == 0.0.into() {
        return;
    }

    ctx.save().unwrap();
    ctx.translate(e.center.x.val(), e.center.y.val());
    ctx.rotate(e.angle.radians());
    ctx.scale(e.semimajor.val(), e.semiminor.val());
    ctx.arc(0., 0., 1., 0., 2.0 * PI);
    ctx.restore().unwrap();

    if let Some(s) = style.stroke {
        ctx.set_line_width(s.size.val());
        ctx.set_source_rgba(s.color.float_r(), s.color.float_g(), s.color.float_b(), s.color.float_alpha());
        ctx.stroke().unwrap();
    }

    if let Some(color) = style.fill {
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

            &DrawCommand::Ellipse { ellipse, style } => {
                draw_ellipse(ctx, ellipse, style);
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
