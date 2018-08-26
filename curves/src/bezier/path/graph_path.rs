use super::path::*;
use super::super::curve::*;
use super::super::intersection::*;
use super::super::super::geo::*;
use super::super::super::coordinate::*;

use std::ops::Range;

const CLOSE_DISTANCE: f64 = 0.01;

///
/// Kind of a graph path edge
/// 
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GraphPathEdgeKind {
    /// An exterior edge
    /// 
    /// These edges represent a transition between the inside and the outside of the path
    Exterior, 

    /// An interior edge
    /// 
    /// These edges are on the inside of the path
    Interior
}

///
/// Enum representing an edge in a graph path
/// 
#[derive(Copy, Clone, Debug)]
pub enum GraphPathEdge {
    /// An exterior edge
    Exterior(usize),

    /// An interior edge
    Interior(usize)
}

impl GraphPathEdge {
    ///
    /// Converts this edge into a kind and a edge number
    /// 
    #[inline]
    pub fn to_kind(&self) -> (GraphPathEdgeKind, usize) {
        match self {
            GraphPathEdge::Exterior(point_index) => (GraphPathEdgeKind::Exterior, *point_index),
            GraphPathEdge::Interior(point_index) => (GraphPathEdgeKind::Interior, *point_index)
        }
    }

    ///
    /// Sets the target point index for this edge
    /// 
    #[inline]
    pub fn set_target(&mut self, new_target: usize) {
        match self {
            GraphPathEdge::Exterior(ref mut point_index) => *point_index = new_target,
            GraphPathEdge::Interior(ref mut point_index) => *point_index = new_target
        }
    }
}

///
/// A graph path is a path where each point can have more than one connected edge. Edges are categorized
/// into interior and exterior edges depending on if they are on the outside or the inside of the combined
/// shape.
/// 
#[derive(Clone, Debug)]
pub struct GraphPath<Point> {
    /// The points in this graph and their edges. Each 'point' here consists of two control points and an end point
    points: Vec<(Point, Point, Point, Vec<GraphPathEdge>)>
}

impl<Point: Coordinate> Geo for GraphPath<Point> {
    type Point = Point;
}

impl<Point: Coordinate+Coordinate2D> GraphPath<Point> {
    ///
    /// Creates a graph path from a bezier path
    /// 
    pub fn from_path<P: BezierPath<Point=Point>>(path: &P) -> GraphPath<Point> {
        // All edges are exterior for a single path
        let mut points = vec![];

        // Push the start point (with an open path)
        let start_point = path.start_point();
        points.push((Point::origin(), Point::origin(), start_point, vec![]));

        // We'll add edges to the previous point
        let mut last_point = 0;
        let mut next_point = 1;

        // Iterate through the points in the path
        for (cp1, cp2, end_point) in path.points() {
            // Push the points
            points.push((cp1, cp2, end_point, vec![]));

            // Add an edge from the last point to the next point
            points[last_point].3.push(GraphPathEdge::Exterior(next_point));

            // Update the last/next pooints
            last_point += 1;
            next_point += 1;
        }

        // Close the path
        if last_point > 0 {
            // Graph actually has some edges
            if start_point.distance_to(&points[last_point].2) < CLOSE_DISTANCE {
                // Start point the same as the last point. Change initial control points
                points[0].0 = points[last_point].0.clone();
                points[0].1 = points[last_point].1.clone();

                // Remove the last point (we're replacing it with an edge back to the start)
                points.pop();
                last_point -= 1;
            } else {
                // Need to draw a line to the last point
                let close_vector = points[last_point].2 - start_point;
                points[0].0 = close_vector * 0.33;
                points[0].1 = close_vector * 0.66;
            }

            // Add an edge from the start point to the end point
            points[last_point].3.push(GraphPathEdge::Exterior(0));
        } else {
            // Just a start point and no edges: remove the start point as it doesn't really make sense
            points.pop();
        }

        // Create the graph path from the points
        GraphPath {
            points: points
        }
    }

    ///
    /// Returns the number of points in this graph. Points are numbered from 0 to this value.
    /// 
    #[inline]
    pub fn num_points(&self) -> usize {
        self.points.len()
    }

    ///
    /// Returns an iterator of the edges connected to a particular point
    ///
    #[inline]
    pub fn edges<'a>(&'a self, point_num: usize) -> impl 'a+Iterator<Item=GraphEdge<'a, Point>> {
        self.points[point_num].3
            .iter()
            .map(move |edge| {
                let (kind, end_point) = edge.to_kind();
                GraphEdge {
                    kind:           kind,
                    graph:          self,
                    start_point:    point_num,
                    end_point:      end_point
                }
            })
    }

    ///
    /// Merges in another path
    /// 
    /// This adds the edges in the new path to this path without considering if they are internal or external 
    ///
    pub fn merge(self, merge_path: GraphPath<Point>) -> GraphPath<Point> {
        // Copy the points from this graph
        let mut new_points  = self.points;

        // Add in points from the merge path
        let offset          = new_points.len();
        new_points.extend(merge_path.points.into_iter()
            .map(|(cp1, cp2, p, mut edges)| {
                // Update the offsets in the edges
                for mut edge in &mut edges {
                    let (_, index) = edge.to_kind();
                    edge.set_target(index + offset);
                }

                // Generate the new edge
                (cp1, cp2, p, edges)
            }));

        // Combined path
        GraphPath {
            points: new_points
        }
    }

    ///
    /// Searches two ranges of points in this object and detects collisions between them, subdividing the edges
    /// and creating branch points at the appropriate places.
    /// 
    fn detect_collisions(&mut self, collide_from: Range<usize>, collide_to: Range<usize>, accuracy: f64) {
        // Iterate through the points in the 'from' range
        for src_idx in collide_from {
            for src_edge in 0..self.points[src_idx].3.len() {
                // Compare to each point in the collide_to range
                for tgt_idx in collide_to.clone() {
                    for tgt_edge in 0..self.points[tgt_idx].3.len() {
                        // Don't collide edges against themselves
                        if src_idx == tgt_idx && src_edge == tgt_edge { continue; }

                        // Create edge objects for each side
                        let (_, src_end_idx)    = self.points[src_idx].3[src_edge].to_kind();
                        let (_, tgt_end_idx)    = self.points[tgt_idx].3[tgt_edge].to_kind();
                        let src_edge            = GraphEdge::new(self, src_idx, src_end_idx);
                        let tgt_edge            = GraphEdge::new(self, tgt_idx, tgt_end_idx);

                        // Quickly reject edges with non-overlapping bounding boxes
                        let src_edge_bounds     = src_edge.fast_bounding_box::<Bounds<_>>();
                        let tgt_edge_bounds     = tgt_edge.fast_bounding_box::<Bounds<_>>();
                        if !src_edge_bounds.overlaps(&tgt_edge_bounds) { continue; }

                        // Find the collisions between these two edges (these a)
                        let collisions          = curve_intersects_curve(&src_edge, &tgt_edge, accuracy);

                        // The are the points we need to divide the existing edges at and add branches

                        // Need to break the edges at each of these points
                        // Points at 0 and 1 just add branches without subdividing
                        // Subdivisions from source and target need to be put back in the source/target lists
                    }
                }
            }
        }
    }

    ///
    /// Collides this path against another, generating a merged path
    /// 
    /// Anywhere this graph intersects the second graph, a point with two edges will be generated. All edges will be left as
    /// interior or exterior depending on how they're set on the graph they originate from.
    /// 
    /// Working out the collision points is the first step to performing path arithmetic: the resulting graph can be altered
    /// to specify edge types - knowing if an edge is an interior or exterior edge makes it possible to tell the difference
    /// between a hole cut into a shape and an intersection.
    /// 
    pub fn collide(mut self, collide_path: GraphPath<Point>, accuracy: f64) -> GraphPath<Point> {
        // Generate a merged path with all of the edges
        let collision_offset    = self.points.len();
        self                    = self.merge(collide_path);

        // Search for collisions between our original path and the new one
        let total_points = self.points.len();
        self.detect_collisions(0..collision_offset, collision_offset..total_points, accuracy);

        // Return the result
        self
    }
}

///
/// Represents an edge in a graph path
/// 
#[derive(Clone)]
pub struct GraphEdge<'a, Point: 'a> {
    /// The graph that this point is for
    graph: &'a GraphPath<Point>,

    /// The kind of edge that this represents
    kind: GraphPathEdgeKind,

    /// The initial point of this edge
    start_point: usize,

    /// The end point of this edge
    end_point: usize
}

impl<'a, Point: 'a> GraphEdge<'a, Point> {
    ///
    /// Creates a new graph edge (with an edge kind of 'exterior')
    /// 
    #[inline]
    fn new(graph: &'a GraphPath<Point>, start_point: usize, end_point: usize) -> GraphEdge<'a, Point> {
        GraphEdge {
            graph:          graph,
            kind:           GraphPathEdgeKind::Exterior,
            start_point:    start_point,
            end_point:      end_point
        }
    }

    ///
    /// Returns if this is an interior or an exterior edge in the path
    /// 
    pub fn kind(&self) -> GraphPathEdgeKind {
        self.kind
    }

    ///
    /// Returns the index of the start point of this edge
    /// 
    #[inline]
    pub fn start_point_index(&self) -> usize {
        self.start_point
    }

    ///
    /// Returns the index of the end point of this edge
    /// 
    #[inline]
    pub fn end_point_index(&self) -> usize {
        self.end_point
    }
}

impl<'a, Point: 'a+Coordinate> Geo for GraphEdge<'a, Point> {
    type Point = Point;
}

impl<'a, Point: 'a+Coordinate> BezierCurve for GraphEdge<'a, Point> {
    ///
    /// The start point of this curve
    /// 
    #[inline]
    fn start_point(&self) -> Self::Point {
        self.graph.points[self.start_point].2.clone()
    }

    ///
    /// The end point of this curve
    /// 
    #[inline]
    fn end_point(&self) -> Self::Point {
        self.graph.points[self.end_point].2.clone()
    }

    ///
    /// The control points in this curve
    /// 
    #[inline]
    fn control_points(&self) -> (Self::Point, Self::Point) {
        (self.graph.points[self.end_point].0.clone(), self.graph.points[self.end_point].1.clone())
    }
}