use std::{collections::HashMap, ops};

use crate::{lua_env::lua_vec2::Vec2, space2::transform2::Transform2};

const EPSILON: f32 = 1e-9;
const EPSILON2: f32 = EPSILON * EPSILON;

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct PolygonCollision2Key {
    start: Option<usize>,
    is_edge: Option<bool>,
}

#[derive(Clone)]
pub struct Collision2 {
    normal: Vec2,
    depth: f32,
    location: Vec2,
    key1: Option<PolygonCollision2Key>,
    key2: Option<PolygonCollision2Key>,
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

    pub fn get_key1(&self) -> Option<PolygonCollision2Key> {
        self.key1.clone()
    }

    pub fn get_key2(&self) -> Option<PolygonCollision2Key> {
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

#[derive(Clone, Eq, Hash, PartialEq)]
struct Collision2DetailsKey {
    key1: Option<PolygonCollision2Key>,
    key2: Option<PolygonCollision2Key>,
}

#[derive(Clone)]
pub struct Collision2Details {
    collision_map: HashMap<Collision2DetailsKey, Collision2>,
}

impl Collision2Details {
    pub fn new() -> Self {
        Self {
            collision_map: HashMap::new(),
        }
    }

    pub fn update_collision_details(&mut self, collisions: Vec<Collision2>) {
        let mut collision_keys_to_keep: HashMap<Collision2DetailsKey, bool> = HashMap::new();
        for collision in collisions {
            let key1 = collision.get_key1();
            let key2 = collision.get_key2();
            let collision2_details_key = Collision2DetailsKey { key1, key2 };
            collision_keys_to_keep.insert(collision2_details_key.clone(), true);
            self.collision_map.insert(collision2_details_key, collision);
        }
        self.collision_map
            .retain(|key, _| collision_keys_to_keep.contains_key(key));
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
    let mut max_dist2 = 0.0;
    let bounding_box_size: f32;
    match shape {
        Shape2::ConvexPolygon(polygon) => {
            for ele in polygon.vertices {
                let dist2 = ele.length_sq();
                if dist2 > max_dist2 {
                    max_dist2 = dist2
                }
            }
            bounding_box_size = 2.0 * max_dist2.sqrt();
        }
        Shape2::Circle(circle) => {
            bounding_box_size = circle.radius;
        }
    }
    Vec2::new(bounding_box_size, bounding_box_size)
}

pub fn point_and_polygon_with_thickness_collide(
    point: Vec2,
    polygon: &ConvexPolygon,
    thickness: f32,
) -> Option<Collision2> {
    let mut depth = f32::MAX;
    let mut key1_start = 0;
    for (i, pv_a) in polygon.vertices.iter().enumerate() {
        let v_a = *pv_a;
        let v_b = polygon.vertices[(i + 1) % (polygon.vertices.len())];
        let signed_dist_to_polygon = (v_b - v_a).cross(point - v_a);
        if signed_dist_to_polygon < -thickness {
            return None;
        }
        if signed_dist_to_polygon < depth {
            depth = signed_dist_to_polygon;
            key1_start = i;
        }
    }
    let v_a = polygon.vertices[key1_start % (polygon.vertices.len())];
    let v_b = polygon.vertices[(key1_start + 1) % (polygon.vertices.len())];
    let normal = (v_a - v_b).normalized();
    Some(Collision2 {
        normal,
        depth,
        location: point,
        key1: None,
        key2: None,
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
            key1: None,
            key2: None,
        })
    } else {
        let dist = dist2.sqrt();
        let mut depth = max_dist - dist;
        if depth < 0.0 {
            depth = 0.0;
        }
        Some(Collision2 {
            normal: (circle.center - other_circle.center).scale(1.0 / dist),
            depth,
            location: (circle.center.scale(other_circle.radius)
                + other_circle.center.scale(circle.radius))
            .scale(1.0 / (max_dist)),
            key1: None,
            key2: None,
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
        Some(-collision)
    } else {
        Some(collision)
    }
}

pub fn collision_option_to_collisions(collision: Option<Collision2>) -> Vec<Collision2> {
    let mut collisions = Vec::new();
    if let Some(collision) = collision {
        collisions.push(collision);
    }
    collisions
}

// Cannot handle weird collisions with only edge/edge intersections
pub fn polygons_collisions(
    polygon: ConvexPolygon,
    other_polygon: ConvexPolygon,
) -> Vec<Collision2> {
    let mut collisions = Vec::new();
    for (i, _) in polygon.vertices.iter().enumerate() {
        let point = polygon.vertices[i % (polygon.vertices.len())];
        if let Some(collision) = point_and_polygon_collide(point, &other_polygon) {
            collisions.push(collision);
        }
    }
    for (j, _) in polygon.vertices.iter().enumerate() {
        let point = polygon.vertices[j % (polygon.vertices.len())];
        if let Some(collision) = point_and_polygon_collide(point, &polygon) {
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
