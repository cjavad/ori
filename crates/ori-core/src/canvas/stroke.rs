use std::hash::{Hash, Hasher};

use crate::{
    canvas::CurveSegment,
    layout::{Point, Vector},
};

use super::Curve;

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

impl Hash for Stroke {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.width.to_bits().hash(state);
        self.miter.to_bits().hash(state);
        self.cap.hash(state);
        self.join.hash(state);
    }
}

impl Curve {
    const MAX_ERROR: f32 = 1.0;
    const MAX_DEPTH: u8 = 6;

    fn offset_line(&mut self, p0: Point, p1: Point, offset: f32) {
        let normal = line_normal(p0, p1);

        let offset = normal * offset;

        self.line_to(p1 + offset);
    }

    fn offset_quad_bezier(&mut self, p0: Point, p1: Point, p2: Point, offset: f32, depth: u8) {
        let n0 = quad_bezier_normal(p0, p1, p2, 0.0);
        let n1 = quad_bezier_normal(p0, p1, p2, 0.5);
        let n2 = quad_bezier_normal(p0, p1, p2, 1.0);

        let op0 = p0 + n0 * offset;
        let op1 = p1 + n1 * offset;
        let op2 = p2 + n2 * offset;

        let real_center = quad_bezier(p0, p1, p2, 0.5) + n0 * offset;
        let center = quad_bezier(op0, op1, op2, 0.5);

        let error = (center - real_center).length();

        if error < Self::MAX_ERROR || depth >= Self::MAX_DEPTH {
            self.quad_to(op1, op2);
            return;
        }

        let [p01, center, p12] = Self::divide_quad_bezier(p0, p1, p2, 0.5);

        self.offset_quad_bezier(p0, p01, center, offset, depth + 1);
        self.offset_quad_bezier(center, p12, p2, offset, depth + 1);
    }

    /// Divide a quadratic Bézier curve at a given point.
    ///
    /// This returns the following points in order:
    /// - The control point for the first curve.
    /// - The end point for the first curve and the start point for the second curve.
    /// - The control point for the second curve.
    fn divide_quad_bezier(p0: Point, p1: Point, p2: Point, t: f32) -> [Point; 3] {
        let p01 = p0.lerp(p1, t);
        let p12 = p1.lerp(p2, t);

        let center = p01.lerp(p12, t);

        [p01, center, p12]
    }

    fn offset_cubic_bezier(
        &mut self,
        p0: Point,
        p1: Point,
        p2: Point,
        p3: Point,
        offset: f32,
        depth: u8,
    ) {
        let n0 = cubic_bezier_normal(p0, p1, p2, p3, 0.0);
        let n1 = cubic_bezier_normal(p0, p1, p2, p3, 1.0 / 3.0);
        let n2 = cubic_bezier_normal(p0, p1, p2, p3, 2.0 / 3.0);
        let n3 = cubic_bezier_normal(p0, p1, p2, p3, 1.0);

        let op0 = p0 + n0 * offset;
        let op1 = p1 + n1 * offset;
        let op2 = p2 + n2 * offset;
        let op3 = p3 + n3 * offset;

        let real_center = cubic_bezier(p0, p1, p2, p3, 1.0 / 3.0) + n0 * offset;
        let center = cubic_bezier(op0, op1, op2, op3, 1.0 / 3.0);

        let error = (center - real_center).length();

        if error < offset * Self::MAX_ERROR || depth >= Self::MAX_DEPTH {
            self.cubic_to(op1, op2, op3);
            return;
        }

        let [p01, p012, center, p123, p23] = Self::divide_cubic_bezier(p0, p1, p2, p3, 1.0 / 3.0);

        self.offset_cubic_bezier(p0, p01, p012, center, offset, depth + 1);
        self.offset_cubic_bezier(center, p123, p23, p3, offset, depth + 1);
    }

    /// Divide a cubic Bézier curve at a given point.
    ///
    /// This returns the following points in order:
    /// - The first control point of the first curve.
    /// - The second control point of the first curve.
    /// - The end point of the first curve and the start point of the second curve.
    /// - The first control point of the second curve.
    /// - The second control point of the second curve.
    fn divide_cubic_bezier(p0: Point, p1: Point, p2: Point, p3: Point, t: f32) -> [Point; 5] {
        let p01 = p0.lerp(p1, t);
        let p12 = p1.lerp(p2, t);
        let p23 = p2.lerp(p3, t);

        let p012 = p01.lerp(p12, t);
        let p123 = p12.lerp(p23, t);

        let center = p012.lerp(p123, t);

        [p01, p012, center, p123, p23]
    }

    fn stroke_line_cap(&mut self, p: Point, n: Vector, t: Vector, stroke: Stroke) {
        let r = stroke.width / 2.0;

        match stroke.cap {
            LineCap::Butt => {
                self.line_to(p + n * r);
            }
            LineCap::Round => {
                let p0 = p - n * r;
                let p1 = p + n * r;

                let hat = t * r * 0.55;
                let nat = n * r * 0.55;

                let c = p + t * r;

                self.cubic_to(p0 + hat, c - nat, c);
                self.cubic_to(c + nat, p1 + hat, p1);
            }
            LineCap::Square => {
                let p0 = p - n * r;
                let p1 = p + n * r;

                let hat = t * r;

                self.line_to(p0 + hat);
                self.line_to(p1 + hat);
                self.line_to(p1);
            }
        }
    }

    fn stroke_join(
        &mut self,
        pivot: Point,
        n0: Vector,
        n1: Vector,
        r: f32,
        join: LineJoin,
        miter_limit: f32,
    ) {
        if n0.cross(n1) * r > 0.0 {
            self.line_to(pivot + n1 * r);
            return;
        }

        match join {
            LineJoin::Miter => self.stroke_miter(pivot, n0, n1, r, miter_limit),
            LineJoin::Round => self.stroke_round(pivot, n0, n1, r),
            LineJoin::Bevel => self.stroke_bevel(pivot, n1, r),
        }
    }

    fn stroke_miter(&mut self, pivot: Point, n0: Vector, n1: Vector, r: f32, limit: f32) {
        let miter = n0 + n1;
        let miter = miter.normalize();

        let amount = miter.dot(n1);

        if 1.0 / amount >= limit {
            self.stroke_bevel(pivot, n1, r);
            return;
        }

        let miter = miter * r / amount;

        let p1 = pivot + n1 * r;

        let m = pivot + miter;

        self.line_to(m);
        self.line_to(p1);
    }

    fn stroke_round(&mut self, pivot: Point, n0: Vector, n1: Vector, r: f32) {
        let p1 = pivot + n1 * r;

        let nmid = Vector::normalize(n0 + n1);

        let mid = pivot + nmid * r;

        let dot = n0.dot(nmid);
        let cos_theta_over_2 = f32::sqrt((1.0 + dot) / 2.0);
        let inv_cos_theta_over_2 = 1.0 / cos_theta_over_2;

        let c0 = pivot + Vector::normalize(n0 + nmid) * r * inv_cos_theta_over_2;
        let c1 = pivot + Vector::normalize(n1 + nmid) * r * inv_cos_theta_over_2;

        self.conic_to(c0, mid, cos_theta_over_2);
        self.conic_to(c1, p1, cos_theta_over_2);
    }

    fn stroke_bevel(&mut self, pivot: Point, n1: Vector, r: f32) {
        let p1 = pivot + n1 * r;

        self.line_to(p1);
    }

    pub(super) fn stroke_impl(&mut self, curve: &Curve, stroke: Stroke) {
        let mut p0 = Point::ZERO;
        let mut n0 = None;

        let mut outside = Curve::new();
        let mut segments = curve.iter().peekable();

        let mut first = None;

        let r = stroke.width / 2.0;

        loop {
            let Some(segment) = segments.next() else {
                break;
            };

            match segment {
                CurveSegment::Move(p1) => {
                    p0 = p1;
                    n0 = None;
                    first = None;
                }
                CurveSegment::Line(p1) => {
                    let n1 = line_normal(p0, p1);

                    match n0 {
                        Some(n0) => {
                            self.stroke_join(p0, n0, n1, r, stroke.join, stroke.miter);
                            outside.stroke_join(p0, n0, n1, -r, stroke.join, stroke.miter);
                        }
                        None => {
                            self.move_to(p0 + n1 * r);
                            outside.move_to(p0 - n1 * r);
                            first = Some((p0, n1));
                        }
                    }

                    self.offset_line(p0, p1, r);
                    outside.offset_line(p0, p1, -r);

                    n0 = Some(n1);
                    p0 = p1;
                }
                CurveSegment::Quad(p1, p2) => {
                    let n1 = quad_bezier_normal(p0, p1, p2, 0.0);

                    match n0 {
                        Some(n0) => {
                            self.stroke_join(p0, n0, n1, r, stroke.join, stroke.miter);
                            outside.stroke_join(p0, n0, n1, -r, stroke.join, stroke.miter);
                        }
                        None => {
                            self.move_to(p0 + n1 * r);
                            outside.move_to(p0 - n1 * r);
                            first = Some((p0, n1));
                        }
                    }

                    self.offset_quad_bezier(p0, p1, p2, r, 0);
                    outside.offset_quad_bezier(p0, p1, p2, -r, 0);

                    n0 = Some(quad_bezier_normal(p0, p1, p2, 1.0));
                    p0 = p2;
                }
                CurveSegment::Cubic(p1, p2, p3) => {
                    let n1 = cubic_bezier_normal(p0, p1, p2, p3, 0.0);

                    match n0 {
                        Some(n0) => {
                            self.stroke_join(p0, n0, n1, r, stroke.join, stroke.miter);
                            outside.stroke_join(p0, n0, n1, -r, stroke.join, stroke.miter);
                        }
                        None => {
                            self.move_to(p0 + n1 * r);
                            outside.move_to(p0 - n1 * r);
                            first = Some((p0, n1));
                        }
                    }

                    self.offset_cubic_bezier(p0, p1, p2, p3, r, 0);
                    outside.offset_cubic_bezier(p0, p1, p2, p3, -r, 0);

                    n0 = Some(cubic_bezier_normal(p0, p1, p2, p3, 1.0));
                    p0 = p3;
                }
                CurveSegment::Close => {
                    self.close();

                    let (pf, nf) = first.unwrap_or((p0, n0.unwrap()));
                    self.move_to(pf - nf * r);
                    self.append_reverse(&outside);
                    self.close();
                }
            }

            if matches!(segments.peek(), None | Some(CurveSegment::Move(_))) {
                if let Some(n) = segment_normal(segment, p0, 1.0) {
                    self.stroke_line_cap(p0, -n, -n.hat(), stroke);
                    self.append_reverse(&outside);

                    let (pf, nf) = first.unwrap_or((p0, n));
                    self.stroke_line_cap(pf, nf, nf.hat(), stroke);
                    self.close();
                }
            }
        }
    }
}

fn line_normal(p0: Point, p1: Point) -> Vector {
    (p1 - p0).hat().normalize()
}

fn quad_bezier(p0: Point, p1: Point, p2: Point, t: f32) -> Point {
    let p01 = p0.lerp(p1, t);
    let p12 = p1.lerp(p2, t);

    p01.lerp(p12, t)
}

fn quad_bezier_tangent(p0: Point, p1: Point, p2: Point, t: f32) -> Vector {
    let p01 = p0.lerp(p1, t);
    let p12 = p1.lerp(p2, t);

    p12 - p01
}

fn quad_bezier_normal(p0: Point, p1: Point, p2: Point, t: f32) -> Vector {
    quad_bezier_tangent(p0, p1, p2, t).hat().normalize()
}

fn cubic_bezier(p0: Point, p1: Point, p2: Point, p3: Point, t: f32) -> Point {
    let p01 = p0.lerp(p1, t);
    let p12 = p1.lerp(p2, t);
    let p23 = p2.lerp(p3, t);

    let p012 = p01.lerp(p12, t);
    let p123 = p12.lerp(p23, t);

    p012.lerp(p123, t)
}

fn cubic_bezier_tangent(p0: Point, p1: Point, p2: Point, p3: Point, t: f32) -> Vector {
    let p01 = p0.lerp(p1, t);
    let p12 = p1.lerp(p2, t);
    let p23 = p2.lerp(p3, t);

    let p012 = p01.lerp(p12, t);
    let p123 = p12.lerp(p23, t);

    p123 - p012
}

fn cubic_bezier_normal(p0: Point, p1: Point, p2: Point, p3: Point, t: f32) -> Vector {
    cubic_bezier_tangent(p0, p1, p2, p3, t).hat().normalize()
}

fn segment_normal(segment: CurveSegment, p0: Point, t: f32) -> Option<Vector> {
    match segment {
        CurveSegment::Move(_) => None,
        CurveSegment::Line(p) => Some(line_normal(p0, p)),
        CurveSegment::Quad(p1, p2) => Some(quad_bezier_normal(p0, p1, p2, t)),
        CurveSegment::Cubic(p1, p2, p3) => Some(cubic_bezier_normal(p0, p1, p2, p3, t)),
        CurveSegment::Close => None,
    }
}
