use std::f64::consts::PI;

use cairo::{Context, LineCap, LineJoin};

use pizarra::{
    draw_commands::DrawCommand, transform::Transform,
    shape::path::PathCommand,
};

pub trait Drawable {
    fn draw(self, ctx: &Context, t: Transform);
}

impl Drawable for DrawCommand {
    fn draw(self, ctx: &Context, t: Transform) {
        match self {
            DrawCommand::Path {
                color, commands, thickness,
            } => {
                ctx.set_line_width(thickness * t.scale_factor());
                ctx.set_source_rgba(color.r, color.g, color.b, color.a);
                ctx.set_line_cap(LineCap::Round);
                ctx.set_line_join(LineJoin::Round);

                for point in commands.iter() {
                    match point {
                        PathCommand::MoveTo(p) => {
                            let p = t.to_screen_coordinates(*p);
                            ctx.move_to(p.x, p.y);
                        },
                        PathCommand::LineTo(p) => {
                            let p = t.to_screen_coordinates(*p);
                            ctx.line_to(p.x, p.y);
                        },
                        PathCommand::CurveTo(c) => {
                            let pt1 = t.to_screen_coordinates(c.pt1);
                            let pt2 = t.to_screen_coordinates(c.pt2);
                            let to = t.to_screen_coordinates(c.to);

                            ctx.curve_to(pt1.x, pt1.y, pt2.x, pt2.y, to.x, to.y);
                        },
                    }
                }

                ctx.stroke();
            },
            DrawCommand::Circle {
                thickness, center, radius, color,
            } => {
                let c = t.to_screen_coordinates(center);

                ctx.set_source_rgba(color.r, color.g, color.b, color.a);
                ctx.arc(c.x, c.y, radius * t.scale_factor(), 0.0, 2.0*PI);
                ctx.set_line_width(thickness * t.scale_factor());
                ctx.stroke();
            },
            DrawCommand::Ellipse {
                thickness, color, center, bigside, smallside, angle,
            } => {
                let center = t.to_screen_coordinates(center);

                if bigside == 0.0 || smallside == 0.0 {
                    return;
                }

                dbg!(angle);

                ctx.set_line_width(thickness * t.scale_factor());
                ctx.set_source_rgba(color.r, color.g, color.b, color.a);

                ctx.save();
                ctx.translate(center.x, center.y);
                ctx.rotate(angle.radians());
                ctx.scale(bigside / 2., smallside / 2.);
                ctx.arc(0., 0., 1., 0., 2.0 * PI);
                ctx.restore();
                ctx.stroke();
            },
        }
    }
}
