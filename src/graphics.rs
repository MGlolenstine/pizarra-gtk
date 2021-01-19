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
            DrawCommand::Line {
                color, line, thickness,
            } => {
                ctx.set_line_width(thickness * t.scale_factor());
                ctx.set_source_rgb(color.r, color.g, color.b);
                ctx.set_line_cap(LineCap::Round);
                ctx.set_line_join(LineJoin::Round);

                let mut iter_points = line.iter();

                if let Some(first_point) = iter_points.next() {
                    let p = t.to_screen_coordinates(*first_point);

                    ctx.move_to(p.x, p.y);

                    for point in iter_points {
                        let p = t.to_screen_coordinates(*point);
                        ctx.line_to(p.x, p.y);
                    }

                    ctx.stroke();
                }
            },
            DrawCommand::Path {
                color, line, thickness,
            } => {
                ctx.set_line_width(thickness * t.scale_factor());
                ctx.set_source_rgb(color.r, color.g, color.b);
                ctx.set_line_cap(LineCap::Round);
                ctx.set_line_join(LineJoin::Round);

                for point in line.iter() {
                    if point.is_nan() {
                        dbg!(point);
                    }

                    match point {
                        PathCommand::MoveTo(p) => {
                            let p = t.to_screen_coordinates(*p);
                            ctx.move_to(p.x, p.y);
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

                ctx.set_source_rgb(color.r, color.g, color.b);
                ctx.arc(c.x, c.y, radius * t.scale_factor(), 0.0, 2.0*PI);
                ctx.set_line_width(thickness * t.scale_factor());
                ctx.stroke();
            },
            DrawCommand::Ellipse {
                bbox, thickness, color,
            } => {
                let bbox = [t.to_screen_coordinates(bbox[0]), t.to_screen_coordinates(bbox[1])];
                let min = bbox[0].min(bbox[1]);
                let max = bbox[0].max(bbox[1]);
                let dimensions = max - min;

                if dimensions.x == 0.0 || dimensions.y == 0.0 {
                    return;
                }

                ctx.set_line_width(thickness * t.scale_factor());
                ctx.set_source_rgb(color.r, color.g, color.b);

                ctx.save();
                ctx.translate(min.x + dimensions.x / 2., min.y + dimensions.y / 2.);
                ctx.scale(dimensions.x / 2., dimensions.y / 2.);
                ctx.arc(0., 0., 1., 0., 2.0 * PI);
                ctx.restore();
                ctx.stroke();
            },
        }
    }
}
