use std::f64::consts::PI;

use cairo::{Context, LineCap, LineJoin, Matrix};

use pizarra::{
    draw_commands::DrawCommand, transform::Transform,
    shape::path::PathCommand,
};

pub trait Drawable {
    fn draw(self, ctx: &Context, t: Transform);
}

impl Drawable for DrawCommand {
    fn draw(self, ctx: &Context, t: Transform) {
        ctx.save();
        ctx.set_matrix(Matrix::new(t.xx, t.yx, t.xy, t.yy, t.x0, t.y0));

        match self {
            DrawCommand::Path {
                color, commands, thickness,
            } => {
                ctx.set_line_width(thickness);
                ctx.set_source_rgba(color.r, color.g, color.b, color.a);
                ctx.set_line_cap(LineCap::Round);
                ctx.set_line_join(LineJoin::Round);

                for point in commands.iter() {
                    match point {
                        PathCommand::MoveTo(p) => {
                            ctx.move_to(p.x, p.y);
                        },
                        PathCommand::LineTo(p) => {
                            ctx.line_to(p.x, p.y);
                        },
                        PathCommand::CurveTo(c) => {
                            ctx.curve_to(c.pt1.x, c.pt1.y, c.pt2.x, c.pt2.y, c.to.x, c.to.y);
                        },
                    }
                }

                ctx.stroke();
            },
            DrawCommand::Circle {
                thickness, center, radius, color,
            } => {
                ctx.set_source_rgba(color.r, color.g, color.b, color.a);
                ctx.arc(center.x, center.y, radius, 0.0, 2.0*PI);
                ctx.set_line_width(thickness);
                ctx.stroke();
            },
            DrawCommand::Ellipse {
                thickness, color, center, semimajor, semiminor, angle,
            } => {
                if semimajor == 0.0 || semiminor == 0.0 {
                    return;
                }

                ctx.set_line_width(thickness);
                ctx.set_source_rgba(color.r, color.g, color.b, color.a);

                ctx.save();
                ctx.translate(center.x, center.y);
                ctx.rotate(angle.radians());
                ctx.scale(semimajor, semiminor);
                ctx.arc(0., 0., 1., 0., 2.0 * PI);
                ctx.restore();
                ctx.stroke();
            },
        }

        ctx.restore();
    }
}
