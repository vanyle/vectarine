use std::{cmp::Ordering, collections::HashMap, ops};

use crate::{lua_env::lua_vec2::Vec2, space2::transform2::Transform2};

// guaranted 1e-3 precision
const EPSILON: f32 = 1e-4;
const EPSILON2: f32 = EPSILON * EPSILON;

// Used wrapped as  to identify  precisely a collision
// - start: vertice where the collision happened
// - is_edge:
//      -> false: the collision only involves the vertice
//         previously identified as `start`
//      -> true: the collision involves the edge of the
//         polygon that starts with the vertice `start`,
//         the other vertice being the next rotating
//         trigonometrically (counter-clockwise)
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct PolygonCollision2Key {
    start: usize,
    is_edge: bool,
}

// Used wrapped as  to identify  precisely a collision
// every necessary information can be deduced from the normal
// of the collision and the center + radius of the circle
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct CircleCollision2Key {}

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum ShapeCollision2Key {
    PolygonCollision2Key(PolygonCollision2Key),
    CircleCollision2Key(CircleCollision2Key),
}

#[derive(Clone)]
pub struct Collision2 {
    normal: Vec2,
    depth: f32,
    location: Vec2,
    key1: ShapeCollision2Key,
    key2: ShapeCollision2Key,
}

impl Collision2 {
    pub fn get_normal(&self) -> Vec2 {
        self.normal
    }

    pub fn get_depth(&self) -> f32 {
        self.depth
    }

    pub fn get_location(&self) -> Vec2 {
        self.location
    }

    pub fn get_key1(&self) -> ShapeCollision2Key {
        self.key1.clone()
    }

    pub fn get_key2(&self) -> ShapeCollision2Key {
        self.key2.clone()
    }
}

impl ops::Neg for Collision2 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self {
            normal: -self.normal,
            depth: self.depth,
            location: self.location,
            key1: self.key1,
            key2: self.key2,
        }
    }
}

#[derive(Clone)]
pub struct Collision2Details {
    collision_map: HashMap<(ShapeCollision2Key, ShapeCollision2Key), Collision2>,
}

impl Collision2Details {
    pub fn new() -> Self {
        Self {
            collision_map: HashMap::new(),
        }
    }

    pub fn update_collision_details(&mut self, collisions: Vec<Collision2>) {
        self.collision_map = HashMap::from_iter(collisions.iter().map(|collision| {
            (
                (collision.get_key1(), collision.get_key2()),
                collision.clone(),
            )
        }));
    }
}

impl Default for Collision2Details {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct ConvexPolygon {
    vertices: Vec<Vec2>,
}

#[derive(Clone)]
pub struct Circle {
    center: Vec2,
    radius: f32,
}

#[derive(Clone)]
pub enum Shape2 {
    ConvexPolygon(ConvexPolygon),
    Circle(Circle),
}

#[derive(Clone)]
pub struct Collider2 {
    dbvh_index: usize,
    transform: Transform2,
    shape: Shape2,
    bounding_box: Vec2,
}

impl Collider2 {
    pub fn get_dbvh_index(&self) -> usize {
        self.dbvh_index
    }

    pub fn get_transform(&self) -> Transform2 {
        self.transform
    }

    pub fn transformed_shape(&self, parent_transform: Transform2) -> Shape2 {
        let transform = parent_transform + self.transform;
        match &self.shape {
            Shape2::ConvexPolygon(polygon) => Shape2::ConvexPolygon(ConvexPolygon {
                vertices: polygon
                    .vertices
                    .iter()
                    .map(|pos| transform.apply(*pos))
                    .collect(),
            }),
            Shape2::Circle(circle) => Shape2::Circle(Circle {
                center: transform.apply(circle.center),
                radius: circle.radius,
            }),
        }
    }
}

pub fn compute_bounding_box(shape: Shape2) -> Vec2 {
    let bounding_box_size = match shape {
        Shape2::ConvexPolygon(polygon) => {
            let max_dist2 = polygon
                .vertices
                .iter()
                .map(|e| e.length_sq())
                .max_by(|a, b| {
                    a.partial_cmp(b).unwrap_or(if a.is_nan() {
                        Ordering::Less
                    } else {
                        Ordering::Greater
                    })
                })
                .expect("Polygon should have vertices");
            2.0 * max_dist2.sqrt()
        }
        Shape2::Circle(circle) => circle.radius,
    };
    Vec2::new(bounding_box_size, bounding_box_size)
}

pub fn point_and_polygon_with_thickness_collide(
    point: Vec2,
    polygon: &ConvexPolygon,
    thickness: f32,
) -> Option<Collision2> {
    let edges = polygon.vertices.iter().zip(
        polygon
            .vertices
            .iter()
            .skip(1)
            .chain(std::iter::once(&polygon.vertices[0])),
    );
    let signed_dist_to_edges = edges.map(|(v_a, v_b)| (*v_b - *v_a).cross(point - *v_a));
    let (edge_start_option, depth) = signed_dist_to_edges
        .enumerate()
        .try_fold(
            (None, f32::MAX),
            |(edge_start_option, depth), (i, signed_dist)| {
                if signed_dist < -thickness {
                    Err((Some(i), signed_dist))
                } else if signed_dist < depth {
                    Ok((Some(i), signed_dist))
                } else {
                    Ok((edge_start_option, depth))
                }
            },
        )
        .ok()?;
    let edge_start = edge_start_option?;

    let v_a = polygon.vertices[edge_start % (polygon.vertices.len())];
    let v_b = polygon.vertices[(edge_start + 1) % (polygon.vertices.len())];
    let normal = if (v_a - v_b).length_sq() > EPSILON2 {
        Some((v_a - v_b).normalized())
    } else {
        let backup_normal_direction = (v_a + v_b).scale(0.5) - point;
        if backup_normal_direction.length_sq() < EPSILON2 {
            return None;
        }
        Some(backup_normal_direction.normalized())
    }?;
    Some(Collision2 {
        normal,
        depth,
        location: point,
        key1: ShapeCollision2Key::CircleCollision2Key(CircleCollision2Key {}),
        key2: ShapeCollision2Key::PolygonCollision2Key(PolygonCollision2Key {
            start: edge_start,
            is_edge: true,
        }),
    })
}

pub fn point_and_polygon_collide(point: Vec2, polygon: &ConvexPolygon) -> Option<Collision2> {
    point_and_polygon_with_thickness_collide(point, polygon, 0.0)
}

pub fn circles_collide(circle: Circle, other_circle: Circle) -> Option<Collision2> {
    let max_dist = circle.radius + other_circle.radius;
    let dist2 = (circle.center - other_circle.center).length_sq();
    if dist2 >= max_dist * max_dist {
        return None;
    }
    if dist2 < EPSILON2 {
        Some(Collision2 {
            normal: Vec2::one(),
            depth: max_dist,
            location: (circle.center + other_circle.center).scale(0.5),
            key1: ShapeCollision2Key::CircleCollision2Key(CircleCollision2Key {}),
            key2: ShapeCollision2Key::CircleCollision2Key(CircleCollision2Key {}),
        })
    } else {
        let dist = dist2.sqrt();
        let depth = f32::max(0.0, max_dist - dist);
        Some(Collision2 {
            normal: (circle.center - other_circle.center).scale(1.0 / dist),
            depth,
            location: (circle.center.scale(other_circle.radius)
                + other_circle.center.scale(circle.radius))
            .scale(1.0 / max_dist),
            key1: ShapeCollision2Key::CircleCollision2Key(CircleCollision2Key {}),
            key2: ShapeCollision2Key::CircleCollision2Key(CircleCollision2Key {}),
        })
    }
}

pub fn circle_and_polygon_collide(
    circle: Circle,
    polygon: ConvexPolygon,
    from_circle: bool,
) -> Option<Collision2> {
    let mut collision =
        point_and_polygon_with_thickness_collide(circle.center, &polygon, circle.radius)?;
    let depth = collision.depth + circle.radius;
    if depth < 0.0 {
        return None;
    }
    collision.location = collision.location - collision.normal.scale(circle.radius);
    collision.depth = depth;
    if from_circle {
        // Swap keys, the normal is from the circle, meaning
        // the polygon (key1) is colliding with the circle (key2)
        collision.key1 = collision.key2;
        collision.key2 = ShapeCollision2Key::CircleCollision2Key(CircleCollision2Key {});
        Some(-collision)
    } else {
        Some(collision)
    }
}

pub fn collision_option_to_collisions(collision: Option<Collision2>) -> Vec<Collision2> {
    collision.into_iter().collect()
}

// Cannot handle weird collisions with only edge/edge intersections
pub fn polygons_collisions(
    polygon: ConvexPolygon,
    other_polygon: ConvexPolygon,
) -> Vec<Collision2> {
    let mut collisions = Vec::new();
    for (i, _) in polygon.vertices.iter().enumerate() {
        let point = polygon.vertices[i % (polygon.vertices.len())];
        if let Some(mut collision) = point_and_polygon_collide(point, &other_polygon) {
            collision.key1 = ShapeCollision2Key::PolygonCollision2Key(PolygonCollision2Key {
                start: i,
                is_edge: false,
            });
            collisions.push(collision);
        }
    }
    for (j, _) in other_polygon.vertices.iter().enumerate() {
        let point = other_polygon.vertices[j % (other_polygon.vertices.len())];
        if let Some(mut collision) = point_and_polygon_collide(point, &polygon) {
            // Swap keys, the normal is from `polygon`, meaning
            // `other_polygon` (key1) is colliding with `polygon` (key2)
            collision.key1 = collision.key2;
            collision.key2 = ShapeCollision2Key::PolygonCollision2Key(PolygonCollision2Key {
                start: j,
                is_edge: false,
            });
            collisions.push(collision);
        }
    }
    collisions
}

// normal is from 2nd object to 1st
pub fn get_shapes_collisions(shape: Shape2, other_shape: Shape2) -> Vec<Collision2> {
    match shape {
        Shape2::ConvexPolygon(polygon) => match other_shape {
            Shape2::ConvexPolygon(other_polygon) => polygons_collisions(polygon, other_polygon),
            Shape2::Circle(other_circle) => collision_option_to_collisions(
                circle_and_polygon_collide(other_circle, polygon, true),
            ),
        },
        Shape2::Circle(circle) => match other_shape {
            Shape2::ConvexPolygon(other_polygon) => collision_option_to_collisions(
                circle_and_polygon_collide(circle, other_polygon, false),
            ),
            Shape2::Circle(other_circle) => {
                collision_option_to_collisions(circles_collide(circle, other_circle))
            }
        },
    }
}

pub fn might_collide(
    collider: &Collider2,
    collider_parent_transform: Transform2,
    other_collider: &Collider2,
    other_collider_parent_transform: Transform2,
) -> bool {
    let collider_min = (collider_parent_transform + collider.transform).applied()
        - collider.bounding_box.scale(0.5);
    let collider_max = collider_min + collider.bounding_box;
    let other_collider_min = (other_collider_parent_transform + other_collider.transform).applied()
        - other_collider.bounding_box.scale(0.5);
    let other_collider_max = other_collider_min + other_collider.bounding_box;

    if collider_max.x() > other_collider_min.x() || other_collider_max.x() > collider_min.x() {
        return false;
    }
    if collider_max.y() > other_collider_min.y() || other_collider_max.y() > collider_min.y() {
        return false;
    }
    true
}

pub fn get_colliders_collisions(
    collider: &Collider2,
    collider_parent_transform: Transform2,
    other_collider: &Collider2,
    other_collider_parent_transform: Transform2,
) -> Vec<Collision2> {
    if !might_collide(
        collider,
        collider_parent_transform,
        other_collider,
        other_collider_parent_transform,
    ) {
        Vec::new()
    } else {
        get_shapes_collisions(
            collider.transformed_shape(collider_parent_transform),
            other_collider.transformed_shape(other_collider_parent_transform),
        )
    }
}
