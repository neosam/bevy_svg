use std::path::PathBuf;
use bevy::{math::{Vec2, Vec3}, prelude::{Color, Transform}};
use lyon_svg::parser::ViewBox;
use lyon_tessellation::math::Point;
use usvg::{IsDefault, NodeExt};

use crate::{bundle::SvgBundle, utils::{ColorExt, TransformExt}};

/// A loaded and deserialized SVG file.
#[derive(Debug)]
pub struct Svg {
    /// The name of the file.
    pub file: String,
    /// Width of the SVG.
    pub width: f64,
    /// Height of the SVG.
    pub height: f64,
    /// ViewBox of the SVG.
    pub view_box: ViewBox,
    /// Origin of the coordinate system and as such the origin for the Bevy position.
    pub origin: Origin,
    pub paths: Vec<PathDescriptor>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// Origin of the coordinate system.
pub enum Origin {
    /// Top left of the image or viewbox, this is the default for a SVG.
    TopLeft,
    /// Center of the image or viewbox.
    Center,
}

impl Default for Origin {
    fn default() -> Self {
        Origin::TopLeft
    }
}

/// Builder for loading a SVG file and building a [`SvgBundle`].
pub struct SvgBuilder {
    file: PathBuf,
    origin: Origin,
    translation: Vec3,
    scale: Vec2,
}

impl SvgBuilder {
    /// Create a [`SvgBuilder`] to load a SVG from a file.
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> SvgBuilder {
        SvgBuilder {
            file: PathBuf::from(path.as_ref()),
            origin: Origin::default(),
            translation: Vec3::default(),
            scale: Vec2::new(1.0, 1.0),
        }
    }

    /// Change the origin of the SVG's coordinate system. The origin is also the
    /// Bevy origin.
    pub fn origin(mut self, origin: Origin) -> SvgBuilder {
        self.origin = origin;
        self
    }

    /// Position at which the [`SvgBundle`] will be spawned in Bevy. The origin
    /// of the SVG coordinate system will be at this position.
    pub fn position(mut self, translation: Vec3) ->  SvgBuilder {
        self.translation = translation;
        self
    }

    /// Value by which the SVG will be scaled, default is (1.0, 1.0).
    pub fn scale(mut self, scale: Vec2) ->  SvgBuilder {
        self.scale = scale;
        self
    }

    /// Load and finish the SVG content into a [`SvgBundle`], which then will be
    /// spawned by the [`SvgPlugin`].
    pub fn build<'s>(self) -> Result<SvgBundle, Box<dyn std::error::Error>> {
        let mut opt = usvg::Options::default();
        opt.fontdb.load_system_fonts();
        //opt.keep_named_groups = true;

        let svg_data = std::fs::read(&self.file)?;
        let svg_tree = usvg::Tree::from_data(&svg_data, &opt)?;

        let view_box = svg_tree.svg_node().view_box;
        let size = svg_tree.svg_node().size;

        println!("view_box: {:?}", view_box);
        println!("size: {:?}", size.to_screen_size());
        let mut transform = usvg::utils::view_box_to_transform(view_box.rect, view_box.aspect, size.to_screen_size().to_size());
        println!("svg::utils::view_box_to_transform: {:#?}", transform);

        let translation = match self.origin {
            Origin::Center => self.translation + Vec3::new(
                -size.width() as f32 * self.scale.x / 2.0,
                size.height() as f32 * self.scale.y / 2.0,
                0.0
            ),
            Origin::TopLeft => self.translation,
        };

        let mut descriptors = Vec::new();

        for node in svg_tree.root().descendants() {
            render_node(&node, &mut transform, &mut descriptors);
        }

        let svg = Svg {
            file: self.file.file_name().unwrap().to_string_lossy().to_string(),
            width: size.width(),
            height: size.height(),
            view_box: ViewBox {
                x: view_box.rect.x(),
                y: view_box.rect.y(),
                w: view_box.rect.width(),
                h: view_box.rect.height(),
            },
            origin: self.origin,
            paths: descriptors,
        };

        Ok(SvgBundle::new(svg).at_position(translation).with_scale(self.scale))
    }
}

fn render_node(node: &usvg::Node, transform: &mut usvg::Transform, descriptors: &mut Vec<PathDescriptor>) {
    match *node.borrow() {
        usvg::NodeKind::Path(ref p) => {
            println!("NodeKind::Path");
            let mut t = node.abs_transform();
            t.append(&node.transform());
            let t = t.to_bevy();

            if let Some(ref fill) = p.fill {
                let color = fill.paint.to_bevy_with_alpha_u8(fill.opacity.to_u8());

                descriptors.push(PathDescriptor {
                    segments: convert_path(p).collect(),
                    abs_transform: t,
                    color,
                    draw_type: DrawType::Fill,
                });
            }

            if let Some(ref stroke) = p.stroke {
                let (color, stroke_opts) = convert_stroke(stroke);

                descriptors.push(PathDescriptor {
                    segments: convert_path(p).collect(),
                    abs_transform: t,
                    color,
                    draw_type: DrawType::Stroke(stroke_opts),
                });
            }
        }
        usvg::NodeKind::Svg(_) => {
            println!("NodeKind::Svg");
            render_group(node, transform, descriptors)
        }
        usvg::NodeKind::Group(ref g) => {
            println!("NodeKind::Group(id: {}) Start", g.id);
            render_group_impl(node, g, transform, descriptors);
            println!("NodeKind::Group(id: {}) End", g.id);
        }
        usvg::NodeKind::Defs => {
            println!("NodeKind::Defs: {:?}", &node);
        }
        _ => {
            println!("_ => : {:#?}", node);
        }
    }
}

fn concat(a: &mut usvg::Transform, b: usvg::Transform) {
    if a.is_identity() {
        a.a = b.a;
        a.b = b.b;
        a.c = b.c;
        a.d = b.d;
        a.e = b.e;
        a.f = b.f;
    } else if b.is_identity() {
        {}
    } else if !a.has_skew() && !b.has_skew() {
        // just scale and translate
        a.a = a.a * b.a;
        a.b = 0.0;
        a.c = 0.0;
        a.d = a.d * b.d;
        a.e = a.a * b.e + a.e;
        a.f = a.d * b.f + a.f;
    } else {
        a.append(&b);
    }
}

pub(crate) fn render_group(parent: &usvg::Node, transform: &mut usvg::Transform, descriptors: &mut Vec<PathDescriptor>) {
    let mut g_bbox = usvg::Rect::new_bbox();

    for node in parent.children() {
        concat(transform, node.transform());
        render_node(&node, transform, descriptors);
    }
}

fn render_group_impl(node: &usvg::Node, g: &usvg::Group, transform: &mut usvg::Transform, descriptors: &mut Vec<PathDescriptor>) {
    let bbox = {
        render_group(node, transform, descriptors)
    };

    // // At this point, `sub_pixmap` has probably the same size as the viewbox.
    // // So instead of clipping, masking and blending the whole viewbox, which can be very expensive,
    // // we're trying to reduce `sub_pixmap` to it's actual content trimming
    // // all transparent borders.
    // //
    // // Basically, if viewbox is 2000x2000 and the current group is 20x20, there is no point
    // // in blending the whole viewbox, we can blend just the current group region.
    // //
    // // Transparency trimming is not yet allowed on groups with filter,
    // // because filter expands the pixmap and it should be handled separately.
    // let (tx, ty, mut sub_pixmap) = if g.filter.is_none() {
    //     trim_transparency(sub_pixmap)?
    // } else {
    //     (0, 0, sub_pixmap)
    // };

    // // During the background rendering for filters,
    // // an opacity, a filter, a clip and a mask should be ignored for the inner group.
    // // So we are simply rendering the `sub_img` without any postprocessing.
    // //
    // // SVG spec, 15.6 Accessing the background image
    // // 'Any filter effects, masking and group opacity that might be set on A[i] do not apply
    // // when rendering the children of A[i] into BUF[i].'
    // if *state == RenderState::BackgroundFinished {
    //     let paint = tiny_skia::PixmapPaint::default();
    //     canvas.pixmap.draw_pixmap(tx, ty, sub_pixmap.as_ref(), &paint,
    //                               tiny_skia::Transform::identity(), None);
    //     return bbox;
    // }

    // // Filter can be rendered on an object without a bbox,
    // // as long as filter uses `userSpaceOnUse`.
    if let Some(ref id) = g.filter {
        println!("g.filter(id {})", id);
    //     if let Some(filter_node) = node.tree().defs_by_id(id) {
    //         if let usvg::NodeKind::Filter(ref filter) = *filter_node.borrow() {
    //             let ts = usvg::Transform::from_native(curr_ts);
    //             let background = prepare_filter_background(node, filter, &sub_pixmap);
    //             let fill_paint = prepare_filter_fill_paint(node, filter, bbox, ts, &sub_pixmap);
    //             let stroke_paint = prepare_filter_stroke_paint(node, filter, bbox, ts, &sub_pixmap);
    //             crate::filter::apply(filter, bbox, &ts, &node.tree(),
    //                                  background.as_ref(), fill_paint.as_ref(), stroke_paint.as_ref(),
    //                                  &mut sub_pixmap);
    //         }
    //     }
    }

    // // Clipping and masking can be done only for objects with a valid bbox.
    // if let Some(bbox) = bbox {
        if let Some(ref id) = g.clip_path {
            println!("g.clip_path(id {})", id);
            // if let Some(clip_node) = node.tree().defs_by_id(id) {
            //     if let usvg::NodeKind::ClipPath(ref cp) = *clip_node.borrow() {
            //         let mut sub_canvas = Canvas::from(sub_pixmap.as_mut());
            //         sub_canvas.translate(-tx as f32, -ty as f32);
            //         sub_canvas.apply_transform(curr_ts);
            //         crate::clip::clip(&clip_node, cp, bbox, &mut sub_canvas);
            //     }
            // }
        }

        if let Some(ref id) = g.mask {
            println!("g.mask(id {})", id);
    //         if let Some(mask_node) = node.tree().defs_by_id(id) {
    //             if let usvg::NodeKind::Mask(ref mask) = *mask_node.borrow() {
    //                 let mut sub_canvas = Canvas::from(sub_pixmap.as_mut());
    //                 sub_canvas.translate(-tx as f32, -ty as f32);
    //                 sub_canvas.apply_transform(curr_ts);
    //                 crate::mask::mask(&mask_node, mask, bbox, &mut sub_canvas);
    //             }
    //         }
        }
    // }

    // let mut paint = tiny_skia::PixmapPaint::default();
    // paint.quality = tiny_skia::FilterQuality::Nearest;
    // if !g.opacity.is_default() {
        // paint.opacity = g.opacity.value() as f32;
    // }

    // canvas.pixmap.draw_pixmap(tx, ty, sub_pixmap.as_ref(), &paint,
                            //   tiny_skia::Transform::identity(), None);

}

#[derive(Debug)]
pub struct PathDescriptor {
    pub segments: Vec<lyon_svg::path::PathEvent>,
    pub abs_transform: Transform,
    pub color: Color,
    pub draw_type: DrawType,
}

#[derive(Debug)]
pub enum DrawType {
    Fill,
    Stroke(lyon_tessellation::StrokeOptions),
}

// Taken from https://github.com/nical/lyon/blob/74e6b137fea70d71d3b537babae22c6652f8843e/examples/wgpu_svg/src/main.rs
struct PathConvIter<'a> {
    iter: std::slice::Iter<'a, usvg::PathSegment>,
    prev: Point,
    first: Point,
    needs_end: bool,
    deferred: Option<lyon_svg::path::PathEvent>,
}

impl<'l> Iterator for PathConvIter<'l> {
    type Item = lyon_svg::path::PathEvent;
    fn next(&mut self) -> Option<Self::Item> {
        use lyon_svg::path::PathEvent;
        if self.deferred.is_some() {
            return self.deferred.take();
        }

        let next = self.iter.next();
        match next {
            Some(usvg::PathSegment::MoveTo { x, y }) => {
                if self.needs_end {
                    let last = self.prev;
                    let first = self.first;
                    self.needs_end = false;
                    self.prev = point(x, y);
                    self.deferred = Some(PathEvent::Begin { at: self.prev });
                    self.first = self.prev;
                    Some(PathEvent::End {
                        last,
                        first,
                        close: false,
                    })
                } else {
                    self.first = point(x, y);
                    Some(PathEvent::Begin { at: self.first })
                }
            }
            Some(usvg::PathSegment::LineTo { x, y }) => {
                self.needs_end = true;
                let from = self.prev;
                self.prev = point(x, y);
                Some(PathEvent::Line {
                    from,
                    to: self.prev,
                })
            }
            Some(usvg::PathSegment::CurveTo {
                x1,
                y1,
                x2,
                y2,
                x,
                y,
            }) => {
                self.needs_end = true;
                let from = self.prev;
                self.prev = point(x, y);
                Some(PathEvent::Cubic {
                    from,
                    ctrl1: point(x1, y1),
                    ctrl2: point(x2, y2),
                    to: self.prev,
                })
            }
            Some(usvg::PathSegment::ClosePath) => {
                self.needs_end = false;
                self.prev = self.first;
                Some(PathEvent::End {
                    last: self.prev,
                    first: self.first,
                    close: true,
                })
            }
            None => {
                if self.needs_end {
                    self.needs_end = false;
                    let last = self.prev;
                    let first = self.first;
                    Some(PathEvent::End {
                        last,
                        first,
                        close: false,
                    })
                } else {
                    None
                }
            }
        }
    }
}

fn point(x: &f64, y: &f64) -> Point {
    Point::new((*x) as f32, (*y) as f32)
}

fn convert_path<'a>(p: &'a usvg::Path) -> PathConvIter<'a> {
    PathConvIter {
        iter: p.data.iter(),
        first: Point::new(0.0, 0.0),
        prev: Point::new(0.0, 0.0),
        deferred: None,
        needs_end: false,
    }
}

fn convert_stroke(s: &usvg::Stroke) -> (Color, lyon_tessellation::StrokeOptions) {
    let color = match s.paint {
        usvg::Paint::Color(c) =>
            Color::rgba_u8(c.red, c.green, c.blue, s.opacity.to_u8()),
        _ => Color::default(),
    };

    let linecap = match s.linecap {
        usvg::LineCap::Butt => lyon_tessellation::LineCap::Butt,
        usvg::LineCap::Square => lyon_tessellation::LineCap::Square,
        usvg::LineCap::Round => lyon_tessellation::LineCap::Round,
    };
    let linejoin = match s.linejoin {
        usvg::LineJoin::Miter => lyon_tessellation::LineJoin::Miter,
        usvg::LineJoin::Bevel => lyon_tessellation::LineJoin::Bevel,
        usvg::LineJoin::Round => lyon_tessellation::LineJoin::Round,
    };

    let opt = lyon_tessellation::StrokeOptions::tolerance(0.01)
        .with_line_width(s.width.value() as f32)
        .with_line_cap(linecap)
        .with_line_join(linejoin);

    (color, opt)
}
