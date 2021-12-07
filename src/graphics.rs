use std::f64::consts::PI;

use cairo::{Context, LineCap, LineJoin, Matrix};

use pizarra::{
    draw_commands::DrawCommand, transform::Transform,
    shape::path::{PathCommand, CubicBezierCurve},
};

pub trait Drawable {
    fn draw(self, ctx: &Context, t: Transform);
}

impl Drawable for DrawCommand {
    fn draw(self, ctx: &Context, t: Transform) {
        ctx.save().unwrap();
        ctx.transform(Matrix::new(t.xx, t.yx, t.xy, t.yy, t.x0, t.y0));

        match self {
            DrawCommand::Path {
                color, commands, thickness,
            } => {
                ctx.set_line_width(thickness);
                ctx.set_source_rgba(color.float_r(), color.float_g(), color.float_b(), color.float_alpha());
                ctx.set_line_cap(LineCap::Round);
                ctx.set_line_join(LineJoin::Round);

                for point in commands.iter() {
                    match *point {
                        PathCommand::MoveTo(p) => {
                            ctx.move_to(p.x, p.y);
                        },
                        PathCommand::LineTo(p) => {
                            ctx.line_to(p.x, p.y);
                        },
                        PathCommand::CurveTo(CubicBezierCurve { pt1, pt2, to }) => {
                            ctx.curve_to(pt1.x, pt1.y, pt2.x, pt2.y, to.x, to.y);
                        },
                    }
                }

                ctx.stroke().unwrap();
            },
            DrawCommand::Circle {
                thickness, center, radius, color,
            } => {
                ctx.set_source_rgba(color.float_r(), color.float_g(), color.float_b(), color.float_alpha());
                ctx.arc(center.x, center.y, radius, 0.0, 2.0*PI);
                ctx.set_line_width(thickness);
                ctx.stroke().unwrap();
            },
            DrawCommand::Ellipse {
                thickness, color, center, semimajor, semiminor, angle,
            } => {
                if semimajor == 0.0 || semiminor == 0.0 {
                    return;
                }

                ctx.set_line_width(thickness);
                ctx.set_source_rgba(color.float_r(), color.float_g(), color.float_b(), color.float_alpha());

                ctx.save().unwrap();
                ctx.translate(center.x, center.y);
                ctx.rotate(angle.radians());
                ctx.scale(semimajor, semiminor);
                ctx.arc(0., 0., 1., 0., 2.0 * PI);
                ctx.restore().unwrap();
                ctx.stroke().unwrap();
            },
        }

        ctx.restore().unwrap();
    }
}
