use crate::{
    layout::{Affine, Point, Rect},
    prelude::Image,
    view::ViewId,
};

use super::{Color, Curve};

/// Ways to draw the end of a line.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LineCap {
    /// The end of the line is squared off.
    Butt,

    /// The end of the line is rounded.
    Round,

    /// The end of the line is squared off and extends past the end of the line.
    Square,
}

/// Ways to join two lines.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LineJoin {
    /// The lines are joined with a sharp corner.
    Miter,

    /// The lines are joined with a rounded corner.
    Round,

    /// The lines are joined with a beveled corner.
    Bevel,
}

/// Properties of a stroke.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stroke {
    /// The width of the stroke.
    pub width: f32,

    /// The miter limit of the stroke.
    pub miter: f32,

    /// The cap of the stroke.
    pub cap: LineCap,

    /// The join of the stroke.
    pub join: LineJoin,
}

impl Default for Stroke {
    fn default() -> Self {
        Self {
            width: 1.0,
            miter: 4.0,
            cap: LineCap::Butt,
            join: LineJoin::Miter,
        }
    }
}

impl From<f32> for Stroke {
    fn from(value: f32) -> Self {
        Self {
            width: value,
            ..Default::default()
        }
    }
}

/// Ways to fill a shape.
#[derive(Clone, Debug, PartialEq)]
pub enum Shader {
    /// A solid color.
    Solid(Color),
}

/// Ways to blend two colors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlendMode {
    /// Replaces the destination with zero.
    Clear,

    /// Replaces the destination with the source.
    Source,

    /// Preserves the destination.
    Destination,

    /// Source over destination.
    SourceOver,

    /// Destination over source.
    DestinationOver,
}

/// A paint that can be used to fill or stroke a shape.
#[derive(Clone, Debug, PartialEq)]
pub struct Paint {
    /// The shader of the paint.
    pub shader: Shader,

    /// The blend mode of the paint.
    pub blend: BlendMode,

    /// Whether the paint should be anti-aliased.
    pub anti_alias: bool,
}

impl Default for Paint {
    fn default() -> Self {
        Self {
            shader: Shader::Solid(Color::BLACK),
            blend: BlendMode::SourceOver,
            anti_alias: true,
        }
    }
}

impl From<Color> for Paint {
    fn from(value: Color) -> Self {
        Self {
            shader: Shader::Solid(value),
            ..Default::default()
        }
    }
}

/// Rule determining if a point is inside a shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FillRule {
    /// A point is inside the shape if a ray from the point crosses a non-zero sum of signed edge
    /// crossings.
    NonZero,

    /// A point is inside the shape if a ray from the point crosses an odd number of edges.
    EvenOdd,
}

/// A mask that can be used to clip a layer.
#[derive(Clone, Debug, PartialEq)]
pub struct Mask {
    /// The curve of the mask.
    pub curve: Curve,

    /// The fill rule of the mask.
    pub fill: FillRule,
}

impl From<Rect> for Mask {
    fn from(value: Rect) -> Self {
        Self {
            curve: Curve::from(value),
            fill: FillRule::NonZero,
        }
    }
}

/// A primitive that can be drawn on a canvas.
#[derive(Clone, Debug, PartialEq)]
pub enum Primitive {
    /// A rectangle.
    Rect {
        /// The rectangle to draw.
        rect: Rect,

        /// The paint to fill the rectangle with.
        paint: Paint,
    },

    /// A filled curve.
    Fill {
        /// The curve to draw.
        curve: Curve,

        /// The fill rule of the curve.
        fill: FillRule,

        /// The paint to fill the curve with.
        paint: Paint,
    },

    /// A stroked curve.
    Stroke {
        /// The curve to draw.
        curve: Curve,

        /// The stroke properties of the curve.
        stroke: Stroke,

        /// The paint to stroke the curve with.
        paint: Paint,
    },

    /// An image.
    Image {
        /// The top-left corner of the image.
        point: Point,

        /// The image to draw.
        image: Image,
    },

    /// A layer that can be transformed and masked.
    Layer {
        /// The primitives of the layer.
        primitives: Vec<Primitive>,

        /// The transformation of the layer.
        transform: Affine,

        /// The mask of the layer.
        mask: Option<Mask>,

        /// The view of the layer.
        view: Option<ViewId>,
    },
}

/// A canvas that can be drawn on.
#[derive(Clone, Debug, PartialEq)]
pub struct Canvas {
    primitives: Vec<Primitive>,
}

impl Default for Canvas {
    fn default() -> Self {
        Self::new()
    }
}

impl Canvas {
    /// Create a new canvas.
    pub fn new() -> Self {
        Self {
            primitives: Vec::new(),
        }
    }

    /// Get the primitives of the canvas.
    pub fn primitives(&self) -> &[Primitive] {
        &self.primitives
    }

    /// Clear the canvas.
    pub fn clear(&mut self) {
        self.primitives.clear();
    }

    /// Draw a rectangle.
    pub fn rect(&mut self, rect: Rect, paint: Paint) {
        self.primitives.push(Primitive::Rect { rect, paint });
    }

    /// Draw a trigger rectangle.
    pub fn trigger(&mut self, rect: Rect) {
        self.primitives.push(Primitive::Rect {
            rect,
            paint: Paint {
                shader: Shader::Solid(Color::TRANSPARENT),
                blend: BlendMode::Destination,
                anti_alias: false,
            },
        });
    }

    /// Fill a curve.
    pub fn fill(&mut self, curve: Curve, fill: FillRule, paint: Paint) {
        self.primitives.push(Primitive::Fill { curve, fill, paint });
    }

    /// Stroke a curve.
    pub fn stroke(&mut self, curve: Curve, stroke: Stroke, paint: Paint) {
        self.primitives.push(Primitive::Stroke {
            curve,
            stroke,
            paint,
        });
    }

    /// Draw an image.
    pub fn image(&mut self, point: Point, image: Image) {
        self.primitives.push(Primitive::Image { point, image });
    }

    /// Draw a layer.
    pub fn layer(
        &mut self,
        transform: Affine,
        mask: Option<Mask>,
        view: Option<ViewId>,
        f: impl FnOnce(&mut Self),
    ) {
        let mut layer = Canvas::new();

        f(&mut layer);

        self.primitives.push(Primitive::Layer {
            primitives: layer.primitives,
            transform,
            mask,
            view,
        });
    }

    /// Draw a layer with a transformation.
    pub fn transform(&mut self, transform: Affine, f: impl FnOnce(&mut Self)) {
        self.layer(transform, None, None, f);
    }

    /// Draw a layer with a mask.
    pub fn mask(&mut self, mask: Mask, f: impl FnOnce(&mut Self)) {
        self.layer(Affine::IDENTITY, Some(mask), None, f);
    }

    /// Draw a layer with a view.
    pub fn view(&mut self, view: ViewId, f: impl FnOnce(&mut Self)) {
        self.layer(Affine::IDENTITY, None, Some(view), f);
    }

    /// Get the view at a point.
    pub fn view_at(&self, point: Point) -> Option<ViewId> {
        fn recurse(primitives: &[Primitive], view: Option<ViewId>, point: Point) -> Option<ViewId> {
            for primitive in primitives.iter().rev() {
                match primitive {
                    Primitive::Rect { rect, .. } => {
                        if rect.contains(point) {
                            return view;
                        }
                    }
                    Primitive::Fill { curve, fill, .. } => {
                        if curve.contains(point, *fill) {
                            return view;
                        }
                    }
                    Primitive::Stroke { .. } => {}
                    Primitive::Image {
                        point: image_point,
                        image,
                    } => {
                        let rect = Rect::min_size(*image_point, image.size());

                        if rect.contains(point) {
                            return view;
                        }
                    }
                    Primitive::Layer {
                        primitives,
                        transform,
                        mask,
                        view,
                    } => {
                        let point = transform.inverse() * point;

                        if let Some(mask) = mask {
                            if !mask.curve.contains(point, mask.fill) {
                                continue;
                            }
                        }

                        let view = recurse(primitives, *view, point);

                        if view.is_some() {
                            return view;
                        }
                    }
                }
            }

            None
        }

        recurse(&self.primitives, None, point)
    }
}
