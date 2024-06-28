//! Gradients
use crate::{style::Rgb, Vec2, XY};

/// A linear gradient interpolating between 0 and 1.
pub struct Linear {
    /// Color for the start of the gradient.
    pub start: Rgb<f32>,

    // No allocation for simple start/end gradients.
    /// List of (position, color) intermediate points in the gradient.
    ///
    /// Positions should be in [0, 1].
    /// The values should be sorted by position.
    pub middle: Vec<(f32, Rgb<f32>)>,

    /// Color for the end of the gradient.
    pub end: Rgb<f32>,
}

impl Linear {
    /// Create a simple gradient with only a start and end colors.
    pub fn new(start: impl Into<Rgb<f32>>, end: impl Into<Rgb<f32>>) -> Self {
        let start = start.into();
        let end = end.into();
        Linear {
            start,
            end,
            middle: Vec::new(),
        }
    }

    /// Interpolate the color for the given position.
    pub fn interpolate(&self, x: f32) -> Rgb<f32> {
        // Find the segment
        if x <= 0f32 {
            return self.start;
        }
        if x >= 1f32 {
            return self.end;
        }

        let mut last = (0f32, self.start);
        for point in self.points() {
            if x > point.0 {
                last = point;
                continue;
            }

            let d = point.0 - last.0;
            let x = if d == 0f32 { 0f32 } else { (x - last.0) / d };

            return Rgb::zip(last.1, point.1).interpolate(x);
        }

        panic!("X has an invalid value (NaN?): {x:?}");
    }

    /// Iterates on the points of this gradient.
    pub fn points(&self) -> impl Iterator<Item = (f32, Rgb<f32>)> + '_ {
        std::iter::once((0f32, self.start))
            .chain(self.middle.iter().copied())
            .chain(std::iter::once((1f32, self.end)))
    }
}

/// Radial gradient.
pub struct Radial {
    /// Where the gradient starts.
    ///
    /// This should be in [0, 1] for each component, as a ratio of the total size.
    pub center: XY<f32>,

    /// The gradient to apply according to the distance from the center.
    pub gradient: Linear,
}

impl Interpolator for Radial {
    fn interpolate(&self, pos: Vec2, size: Vec2) -> Rgb<f32> {
        let size_f32 = size.map(|x| x as f32);

        // Find the further corner from `size`.
        //
        // TODO: cache this for the same value of `size`?
        let to_corner = self.center.map(|x| 0.5f32 + (x - 0.5f32).abs()) * size_f32;
        let max_distance = (to_corner.map(|x| x as isize).sq_norm() as f32).sqrt();

        let center = (self.center * size_f32).map(|x| x as isize);

        let sq_dist = (center - pos.signed()).sq_norm();
        let dist = (sq_dist as f32).sqrt();

        self.gradient.interpolate(dist / max_distance)
    }
}

/// An angled linear gradient.
pub struct Angled {
    /// Angle of the gradient in radians.
    ///
    /// 0 = vertical.
    pub angle_rad: f32,

    /// The gradient to apply following the gradient angle.
    pub gradient: Linear,
}

/// Something that can interpolate.
pub trait Interpolator {
    /// Get the color for the given position, given the total size.
    fn interpolate(&self, pos: Vec2, size: Vec2) -> Rgb<f32>;
}

impl Interpolator for Angled {
    fn interpolate(&self, mut pos: Vec2, mut size: Vec2) -> Rgb<f32> {
        use std::f32::consts::{FRAC_PI_2, PI, TAU};

        let mut angle = self.angle_rad;

        // First, normalize the angle: add/remove TAU until we are in [0, TAU[
        while angle < 0f32 {
            angle += TAU;
        }

        while angle >= TAU {
            angle -= TAU;
        }

        // Now there are 4 quadrants we need to handle: [0:PI/2[, [PI/2:PI[, [PI:3PI/2[, [3PI/2, TAU[
        match angle {
            _ if angle < FRAC_PI_2 => (),
            _ if angle < PI => {
                // Here, pos.x = max.x - pos.
                pos = Vec2::new(size.y - pos.y, pos.x);
                size = size.swap();
                angle -= FRAC_PI_2;
            }
            _ if angle < PI + FRAC_PI_2 => {
                pos = size - pos;
                angle -= PI;
            }
            _ => {
                pos = Vec2::new(pos.y, size.x - pos.x);
                size = size.swap();
                angle -= PI + FRAC_PI_2;
            }
        }

        let d = pos.map(|x| x as f32).rotated(angle).y;
        let max = size.map(|x| x as f32).rotated(angle).y;

        self.gradient.interpolate(d / max)
    }
}

/// Bilinear gradient.
///
/// This applies bilinear interpolation to a rectangle with a given color at each corner.
pub struct Bilinear {
    /// Color for the top-left corner.
    pub top_left: Rgb<f32>,
    /// Color for the bottom-left corner.
    pub bottom_left: Rgb<f32>,
    /// Color for the top-right corner.
    pub top_right: Rgb<f32>,
    /// Color for the bottom-right corner.
    pub bottom_right: Rgb<f32>,
}

impl Interpolator for Bilinear {
    fn interpolate(&self, pos: Vec2, size: Vec2) -> Rgb<f32> {
        // TODO: handle size = 0 or 1 in any axis.
        // Size=0 => doesn't matter
        // Size=1 => ??? first value?
        let pos = pos.map(|x| x as f32) / size.map(|x| (x - 1) as f32);

        let top = Linear::new(self.top_left, self.top_right).interpolate(pos.x);
        let bottom = Linear::new(self.bottom_left, self.bottom_right).interpolate(pos.x);

        Linear::new(top, bottom).interpolate(pos.y)
    }
}
