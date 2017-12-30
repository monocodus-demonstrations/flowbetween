use super::super::traits::*;
use ui::canvas::*;

use std::ops::*;

use curves::*;
use curves::bezier;

// Minimum distance between points to use to fit to a curve
const MIN_DISTANCE: f64 = 2.0;

///
/// The ink brush draws a solid line with width based on pressure
/// 
pub struct InkBrush {
    /// The blend mode that this brush will use
    blend_mode: BlendMode,

    /// Width at pressure 0%
    min_width: f32,

    /// Width at pressure 100%
    max_width: f32,

    // Distance to scale up at the start of the brush stroke
    scale_up_distance: f32
}

impl InkBrush {
    ///
    /// Creates a new ink brush with the default settings
    /// 
    pub fn new(definition: &InkDefinition, drawing_style: BrushDrawingStyle) -> InkBrush {
        use BrushDrawingStyle::*;

        let blend_mode = match drawing_style {
            Draw    => BlendMode::SourceOver,
            Erase   => BlendMode::DestinationOut
        };

        InkBrush {
            blend_mode:         blend_mode,
            min_width:          definition.min_width,
            max_width:          definition.max_width,
            scale_up_distance:  definition.scale_up_distance
        }
    }
}

///
/// Ink brush coordinate (used for curve fitting)
/// 
#[derive(Clone, Copy)]
struct InkCoord {
    x: f64,
    y: f64,
    pressure: f64
}

impl InkCoord {
    pub fn pressure(&self) -> f64 { self.pressure }
    pub fn set_pressure(&mut self, new_pressure: f64) {
        self.pressure = new_pressure;
    }

    pub fn to_coord2(&self) -> (Coord2, f64) {
        (Coord2(self.x, self.y), self.pressure)
    }
}

impl<'a> From<&'a BrushPoint> for InkCoord {
    fn from(src: &'a BrushPoint) -> InkCoord {
        InkCoord {
            x: src.position.0 as f64,
            y: src.position.1 as f64,
            pressure: src.pressure as f64
        }
    }
}

impl Add<InkCoord> for InkCoord {
    type Output=InkCoord;

    #[inline]
    fn add(self, rhs: InkCoord) -> InkCoord {
        InkCoord {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            pressure: self.pressure + rhs.pressure
        }
    }
}

impl Sub<InkCoord> for InkCoord {
    type Output=InkCoord;

    #[inline]
    fn sub(self, rhs: InkCoord) -> InkCoord {
        InkCoord {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            pressure: self.pressure - rhs.pressure
        }
    }
}

impl Mul<f64> for InkCoord {
    type Output=InkCoord;

    #[inline]
    fn mul(self, rhs: f64) -> InkCoord {
        InkCoord {
            x: self.x * rhs,
            y: self.y * rhs,
            pressure: self.pressure * rhs
        }
    }
}

impl Coordinate for InkCoord {
    #[inline]
    fn from_components(components: &[f64]) -> InkCoord {
        InkCoord { x: components[0], y: components[1], pressure: components[2] }
    }

    #[inline]
    fn origin() -> InkCoord {
        InkCoord { x: 0.0, y: 0.0, pressure: 0.0 }
    }

    #[inline]
    fn len() -> usize { 3 }

    #[inline]
    fn get(&self, index: usize) -> f64 { 
        match index {
            0 => self.x,
            1 => self.y,
            2 => self.pressure,
            _ => panic!("InkCoord only has three components")
        }
    }

    fn from_biggest_components(p1: InkCoord, p2: InkCoord) -> InkCoord {
        InkCoord {
            x: f64::from_biggest_components(p1.x, p2.x),
            y: f64::from_biggest_components(p1.y, p2.y),
            pressure: f64::from_biggest_components(p1.pressure, p2.pressure)
        }
    }

    fn from_smallest_components(p1: InkCoord, p2: InkCoord) -> InkCoord {
        InkCoord {
            x: f64::from_smallest_components(p1.x, p2.x),
            y: f64::from_smallest_components(p1.y, p2.y),
            pressure: f64::from_smallest_components(p1.pressure, p2.pressure)
        }
    }

    #[inline]
    fn distance_to(&self, target: &InkCoord) -> f64 {
        let dist_x = target.x-self.x;
        let dist_y = target.y-self.y;
        let dist_p = target.pressure-self.pressure;

        f64::sqrt(dist_x*dist_x + dist_y*dist_y + dist_p*dist_p)
    }

    #[inline]
    fn dot(&self, target: &Self) -> f64 {
        self.x*target.x + self.y*target.y + self.pressure*target.pressure
    }
}

///
/// Bezier curve using InkCoords
/// 
#[derive(Clone, Copy)]
struct InkCurve {
    pub start_point:    InkCoord,
    pub end_point:      InkCoord,
    pub control_points: (InkCoord, InkCoord)
}

impl InkCurve {
    ///
    /// Converts to a pair of offset curves
    /// 
    pub fn to_offset_curves(&self, min_width: f64, max_width: f64) -> (Vec<bezier::Curve>, Vec<bezier::Curve>) {
        // Fetch the coordinates for the offset curve
        let (start, start_pressure) = self.start_point().to_coord2();
        let (end, end_pressure)     = self.end_point().to_coord2();
        let cp1                     = self.control_points.0.to_coord2().0;
        let cp2                     = self.control_points.1.to_coord2().0;

        // Create the top and bottom offsets
        let start_offset    = start_pressure*(max_width-min_width) + min_width;
        let end_offset      = end_pressure*(max_width-min_width) + min_width;
        let base_curve      = bezier::Curve::from_points(start, end, cp1, cp2);

        let offset_up       = bezier::offset(&base_curve, start_offset, end_offset);
        let offset_down     = bezier::offset(&base_curve, -start_offset, -end_offset);

        (offset_up, offset_down)
    }
}

impl BezierCurve for InkCurve {
    type Point = InkCoord;

    fn from_points(start: InkCoord, end: InkCoord, control_point1: InkCoord, control_point2: InkCoord) -> InkCurve {
        InkCurve {
            start_point:    start,
            end_point:      end,
            control_points: (control_point1, control_point2)
        }
    }

    #[inline]
    fn start_point(&self) -> InkCoord {
        self.start_point
    }

    #[inline]
    fn end_point(&self) -> InkCoord {
        self.end_point
    }

    #[inline]
    fn control_points(&self) -> (InkCoord, InkCoord) {
        self.control_points
    }
}

impl Brush for InkBrush {
    fn prepare_to_render(&self, gc: &mut GraphicsPrimitives) {
        // Set the blend mode (mainly so we can act as an eraser as well as a primary brush)
        gc.blend_mode(self.blend_mode);

        // Set the fill colour
        gc.fill_color(Color::Rgba(0.0, 0.0, 0.0, 1.0));
    }

    fn render_brush(&self, gc: &mut GraphicsPrimitives, points: &Vec<BrushPoint>) {
        // TODO: somewhat glitchy, not sure why (lines disappear sometimes, or sometimes end up with a line to infinity)

        // Nothing to draw if there are no points in the brush stroke (or only one point)
        if points.len() <= 2 {
            return;
        }

        // Convert points to ink points
        let ink_points: Vec<InkCoord> = points.iter().map(|point| InkCoord::from(point)).collect();

        // Average points that are very close together so we don't overdo 
        // the curve fitting
        let mut averaged_points = vec![];
        let mut last_point      = ink_points[0];
        averaged_points.push(last_point);

        for point in ink_points.iter().skip(1) {
            // If the distance between this point and the last one is below a 
            // threshold, average them together
            let distance = last_point.distance_to(point);

            if distance < MIN_DISTANCE {
                // Average this point with the previous average
                // TODO: (We should really total up the number of points we're 
                // averaging over)
                let num_averaged    = averaged_points.len();
                let current_average = averaged_points[num_averaged-1];
                let averaged_point  = (current_average + last_point) * 0.5;

                // Update the earlier point (and don't update last_point: we'll 
                // keep averaging until we find a new point far enough away)
                averaged_points[num_averaged-1] = averaged_point;
            } else {
                // Keep this point
                averaged_points.push(*point);

                // Update the last point
                last_point = *point;
            }
        }

        // Smooth out the points to remove any jitteryness
        let mut ink_points = InkCoord::smooth(&averaged_points, &[0.1, 0.25, 0.3, 0.25, 0.1]);

        // Scale up the pressure at the start of the brush stroke
        let mut distance    = 0.0;
        let mut last_point  = ink_points[0];
        let scale_up_distance = self.scale_up_distance as f64;
        for point in ink_points.iter_mut() {
            // Compute the current distnace
            distance += last_point.distance_to(point);
            last_point = *point;

            // Scale the pressure by the distance
            if distance > scale_up_distance { break; }

            let pressure = point.pressure();
            point.set_pressure(pressure * (distance/scale_up_distance));
        }

        // Fit these points to a curve
        let curve = InkCurve::fit_from_points(&ink_points, 1.0);
        
        // Draw a variable width line for this curve
        if let Some(curve) = curve {
            let offset_curves: Vec<(Vec<bezier::Curve>, Vec<bezier::Curve>)> 
                = curve.iter().map(|ink_curve| ink_curve.to_offset_curves(self.min_width as f64, self.max_width as f64)).collect();

            gc.new_path();
            
            // Upper portion
            let Coord2(x, y) = offset_curves[0].0[0].start_point();
            gc.move_to(x as f32, y as f32);
            for curve_list in offset_curves.iter() {
                for curve_section in curve_list.0.iter() {
                    gc_draw_bezier(gc, curve_section);
                }
            }

            // Lower portion (reverse everything)
            let last_section    = &offset_curves[offset_curves.len()-1].1;
            let last_curve      = &last_section[last_section.len()-1];
            let Coord2(x, y)    = last_curve.end_point();
            gc.line_to(x as f32, y as f32);

            for curve_list in offset_curves.iter().rev() {
                for curve_section in curve_list.1.iter().rev() {
                    let start       = curve_section.start_point();
                    let (cp1, cp2)  = curve_section.control_points();

                    gc.bezier_curve_to(start.x() as f32, start.y() as f32, cp2.x() as f32, cp2.y() as f32, cp1.x() as f32, cp1.y() as f32);
                }
            }

            gc.fill();
        }
    }

    ///
    /// Retrieves the definition for this brush
    /// 
    fn to_definition(&self) -> (BrushDefinition, BrushDrawingStyle) {
        let definition = BrushDefinition::Ink(InkDefinition {
            min_width:          self.min_width,
            max_width:          self.max_width,
            scale_up_distance:  self.scale_up_distance
        });
        
        let drawing_style = match self.blend_mode {
            BlendMode::DestinationOut   => BrushDrawingStyle::Erase,
            _                           => BrushDrawingStyle::Draw
        };

        (definition, drawing_style)
    }
}
