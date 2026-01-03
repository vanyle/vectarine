use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

use mlua::{FromLua, IntoLua, UserDataFields, UserDataMethods};
use nalgebra::Isometry2;
use rapier2d::{
    math::Vector,
    prelude::{
        CCDSolver, Collider, ColliderBuilder, ColliderSet, DefaultBroadPhase, ImpulseJointHandle,
        ImpulseJointSet, IntegrationParameters, IslandManager, MultibodyJointSet, NarrowPhase,
        PhysicsPipeline, QueryFilter, RevoluteJointBuilder, RigidBody, RigidBodyBuilder,
        RigidBodyHandle, RigidBodySet,
    },
};

use crate::{
    auto_impl_lua_take,
    lua_env::{add_fn_to_table, is_valid_data_type, lua_camera::Camera2, lua_vec2::Vec2},
};

// MARK: World2

/// Lua wrapper around a rapier physics world
pub struct PhysicsWorld2 {
    physics_pipeline: PhysicsPipeline,
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    gravity: Vec2,
    integration_parameters: IntegrationParameters,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    ccd_solver: CCDSolver,
    camera: Option<mlua::Value>,

    extras: HashMap<RigidBodyHandle, ExtraObjectData>,
}

pub fn ensure_camera_is_valid(maybe_camera: &mlua::Value) -> mlua::Result<()> {
    if !is_valid_data_type::<Camera2>(maybe_camera) {
        return Err(mlua::Error::ToLuaConversionError {
            from: maybe_camera.type_name().to_string(),
            to: "Camera2",
            message: Some("Invalid camera when creating World2".to_string()),
        });
    }
    Ok(())
}

impl PhysicsWorld2 {
    fn new(camera: Option<mlua::Value>, gravity: Vec2) -> mlua::Result<Self> {
        let camera = if let Some(camera) = camera {
            ensure_camera_is_valid(&camera)?;
            Some(camera)
        } else {
            None
        };

        Ok(Self {
            physics_pipeline: PhysicsPipeline::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            gravity,
            integration_parameters: IntegrationParameters::default(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            camera,
            extras: HashMap::new(),
        })
    }
}

#[derive(Clone)]
pub struct LuaPhysicsWorld2(Rc<RefCell<PhysicsWorld2>>);
auto_impl_lua_take!(LuaPhysicsWorld2, LuaPhysicsWorld2);

// MARK: Collider2

struct Collider2 {
    collider: Collider,
}
auto_impl_lua_take!(Collider2, Collider2);

// MARK: Joint2

struct Joint2 {
    joint: ImpulseJointHandle,
    world: Weak<RefCell<PhysicsWorld2>>,
}
auto_impl_lua_take!(Joint2, Joint2);

// MARK: Object2

pub struct Object2 {
    pub rigid_body_handle: RigidBodyHandle,
    pub world: Weak<RefCell<PhysicsWorld2>>,
}

impl Object2 {
    pub fn is_out_of_world(&self) -> bool {
        self.world.upgrade().is_none()
    }
    pub fn position(&self) -> Option<Vec2> {
        let world = self.world.upgrade()?;
        let world = world.borrow();
        let world = &*world;
        let rigid_body = world.rigid_body_set.get(self.rigid_body_handle)?;
        let position = rigid_body.position();
        Some(Vec2::new(position.translation.x, position.translation.y))
    }
    pub fn velocity(&self) -> Option<Vec2> {
        let world = self.world.upgrade()?;
        let world = world.borrow();
        let world = &*world;
        let rigid_body = world.rigid_body_set.get(self.rigid_body_handle)?;
        let velocity = rigid_body.linvel();
        Some(Vec2::new(velocity.x, velocity.y))
    }
    pub fn set_position(&self, position: Vec2) -> Option<()> {
        let world = self.world.upgrade()?;
        let mut world = world.borrow_mut();
        let world = &mut *world;
        let rigid_body = world.rigid_body_set.get_mut(self.rigid_body_handle)?;
        rigid_body.set_translation(nalgebra::vector![position.x(), position.y()], true);
        Some(())
    }
    pub fn set_velocity(&self, velocity: Vec2) -> Option<()> {
        let world = self.world.upgrade()?;
        let mut world = world.borrow_mut();
        let world = &mut *world;
        let rigid_body = world.rigid_body_set.get_mut(self.rigid_body_handle)?;
        rigid_body.set_linvel(nalgebra::vector![velocity.x(), velocity.y()], true);
        Some(())
    }
}

struct ExtraObjectData {
    tags: mlua::Table,
    extra_custom: mlua::Value,
}

auto_impl_lua_take!(Object2, Object2);

pub fn setup_physics_api(lua: &Rc<mlua::Lua>) -> mlua::Result<mlua::Table> {
    let physics_module = lua.create_table()?;

    // MARK: World2 fn
    add_fn_to_table(lua, &physics_module, "newWorld2", {
        move |_, (gravity, camera): (Option<Vec2>, mlua::Value)| {
            let camera = if camera.is_nil() { None } else { Some(camera) };
            let world = PhysicsWorld2::new(camera, gravity.unwrap_or(Vec2::new(0.0, 0.0)))?;
            Ok(LuaPhysicsWorld2(Rc::new(RefCell::new(world))))
        }
    });

    lua.register_userdata_type::<LuaPhysicsWorld2>(|registry| {
        registry.add_field_method_get("camera", |_, world| {
            let cam = world.0.borrow().camera.clone();
            match cam {
                Some(cam) => Ok(cam),
                None => Ok(mlua::Nil),
            }
        });
        registry.add_field_method_set("camera", |_, world, new_camera: mlua::Value| {
            if new_camera.is_nil() {
                world.0.borrow_mut().camera = None;
            } else {
                ensure_camera_is_valid(&new_camera)?;
                world.0.borrow_mut().camera = Some(new_camera);
            }
            Ok(())
        });
        registry.add_field_method_get("gravity", |_, world| Ok(world.0.borrow().gravity));
        registry.add_field_method_set("gravity", |_, world, gravity: Vec2| {
            world.0.borrow_mut().gravity = gravity;
            Ok(())
        });

        registry.add_method_mut("step", |_, world, dt: f32| {
            let mut world = world.0.borrow_mut();
            let world = &mut *world;
            let physics_hooks = ();
            let event_handler = ();

            let rapier_gravity = rapier2d::prelude::vector![world.gravity.x(), world.gravity.y()];
            world.integration_parameters.dt = dt;

            world.physics_pipeline.step(
                &rapier_gravity,
                &world.integration_parameters,
                &mut world.island_manager,
                &mut world.broad_phase,
                &mut world.narrow_phase,
                &mut world.rigid_body_set,
                &mut world.collider_set,
                &mut world.impulse_joint_set,
                &mut world.multibody_joint_set, // unused, impulse joints are better for our use-case.
                &mut world.ccd_solver,
                &physics_hooks,
                &event_handler,
            );
            Ok(())
        });

        registry.add_method_mut("createObject", {
            move |_,
                  lua_world,
                  (position, mass, maybe_collider, tags, body_type): (
                Vec2,
                f32,
                mlua::AnyUserData,
                mlua::Table,
                String,
            )| {
                let collider = maybe_collider.borrow::<Collider2>()?;
                let mut world = lua_world.0.borrow_mut();
                let world = &mut *world;

                let body_builder = match body_type.as_str() {
                    "dynamic" => RigidBodyBuilder::dynamic(),
                    "static" => RigidBodyBuilder::fixed(),
                    "kinematic" => RigidBodyBuilder::kinematic_velocity_based(),
                    _ => {
                        return Err(mlua::Error::FromLuaConversionError {
                            from: "string",
                            to: "RigidBodyType".to_string(),
                            message: Some(
                                "Invalid body type, expected 'dynamic', 'static' or 'kinematic'"
                                    .to_string(),
                            ),
                        });
                    }
                };
                let body = body_builder
                    .pose(Isometry2::translation(position.x(), position.y()))
                    .additional_mass(mass)
                    .build();
                let body_handle = world.rigid_body_set.insert(body);
                let collider = collider.collider.clone();
                world.collider_set.insert_with_parent(
                    collider,
                    body_handle,
                    &mut world.rigid_body_set,
                );

                let object = Object2 {
                    rigid_body_handle: body_handle,
                    world: Rc::downgrade(&lua_world.0),
                };
                world.extras.insert(
                    body_handle,
                    ExtraObjectData {
                        tags,
                        extra_custom: mlua::Nil,
                    },
                );
                Ok(object)
            }
        });

        // We pass object directly here because we WANT to take ownership (the object is invalid afterwards)
        registry.add_method_mut("removeObject", |_, world, object: Object2| {
            let mut world = world.0.borrow_mut();
            let world = &mut *world;
            world.extras.remove(&object.rigid_body_handle);
            world.rigid_body_set.remove(
                object.rigid_body_handle,
                &mut world.island_manager,
                &mut world.collider_set,
                &mut world.impulse_joint_set,
                &mut world.multibody_joint_set,
                true,
            );
            Ok(())
        });

        registry.add_method_mut(
            "getObjects",
            |_, lua_world, tags: Option<Vec<mlua::Value>>| {
                let tags = tags.unwrap_or_default();
                let mut world = lua_world.0.borrow_mut();
                let world = &mut *world;
                let objects = world
                    .extras
                    .iter()
                    .filter(|(_, extra)| {
                        tags.iter().all(|queried_tag| {
                            extra
                                .tags
                                .pairs::<mlua::Value, mlua::Value>()
                                .filter_map(|o| o.ok())
                                .any(|(_, object_tag)| object_tag == *queried_tag)
                        })
                    })
                    .map(|(&handle, _)| Object2 {
                        rigid_body_handle: handle,
                        world: Rc::downgrade(&lua_world.0),
                    })
                    .collect::<Vec<_>>();
                Ok(objects)
            },
        );

        registry.add_method_mut("getObjectsAtPoint", {
            move |_, lua_world, (point,): (Vec2,)| {
                let world = lua_world.0.borrow();
                let world = &*world;
                let filter = QueryFilter::default();
                let query_pipeline = world.broad_phase.as_query_pipeline(
                    world.narrow_phase.query_dispatcher(),
                    &world.rigid_body_set,
                    &world.collider_set,
                    filter,
                );
                let matches =
                    query_pipeline.intersect_point(rapier2d::prelude::point![point.x(), point.y()]);
                Ok(matches
                    .filter_map(|m| m.1.parent())
                    .map(|parent| Object2 {
                        rigid_body_handle: parent,
                        world: Rc::downgrade(&lua_world.0),
                    })
                    .collect::<Vec<_>>())
            }
        });

        registry.add_method_mut("getObjectsInArea", {
            move |_, lua_world, (position, size): (Vec2, Vec2)| {
                let world = lua_world.0.borrow();
                let world = &*world;
                let filter = QueryFilter::default();
                let query_pipeline = world.broad_phase.as_query_pipeline(
                    world.narrow_phase.query_dispatcher(),
                    &world.rigid_body_set,
                    &world.collider_set,
                    filter,
                );
                let mins = rapier2d::prelude::point![position.x(), position.y()];
                let maxs =
                    rapier2d::prelude::point![position.x() + size.x(), position.y() + size.y()];
                let aabb = rapier2d::prelude::Aabb::new(mins, maxs);
                let matches = query_pipeline.intersect_aabb_conservative(aabb);
                Ok(matches
                    .filter_map(|m| m.1.parent())
                    .map(|parent| Object2 {
                        rigid_body_handle: parent,
                        world: Rc::downgrade(&lua_world.0),
                    })
                    .collect::<Vec<_>>())
            }
        });

        registry.add_method_mut("getObjectsIntersectingRay", {
            move |lua, lua_world, (position, direction, max_length): (Vec2, Vec2, Option<f32>)| {
                let world = lua_world.0.borrow();
                let world = &*world;
                let filter = QueryFilter::default();
                let query_pipeline = world.broad_phase.as_query_pipeline(
                    world.narrow_phase.query_dispatcher(),
                    &world.rigid_body_set,
                    &world.collider_set,
                    filter,
                );
                let position = rapier2d::prelude::point![position.x(), position.y()];
                let direction = rapier2d::prelude::vector![direction.x(), direction.y()];
                let ray = rapier2d::prelude::Ray::new(position, direction);
                let matches =
                    query_pipeline.intersect_ray(ray, max_length.unwrap_or(10000.0), true);
                Ok(matches
                    .filter_map(|(_, collider, intersection)| {
                        let parent = collider.parent()?;
                        let o = Object2 {
                            rigid_body_handle: parent,
                            world: Rc::downgrade(&lua_world.0),
                        };
                        let table = lua.create_table().ok()?;
                        table.raw_set("object", o).ok()?;
                        table
                            .raw_set("timeOfImpact", intersection.time_of_impact)
                            .ok()?;
                        Some(table)
                    })
                    .collect::<Vec<_>>())
            }
        });

        registry.add_method_mut("getJoints", {
            move |_, lua_world, (): ()| {
                let world = lua_world.0.borrow();
                let world = &*world;
                let handles = world
                    .impulse_joint_set
                    .iter()
                    .map(|(joint_handle, _)| Joint2 {
                        joint: joint_handle,
                        world: Rc::downgrade(&lua_world.0),
                    })
                    .collect::<Vec<_>>();
                Ok(handles)
            }
        });

        // MARK: Joint2 fn
        registry.add_method_mut("createDistanceJoint", {
            move |_, lua_world, (object1, object2): (Object2, Object2)| {
                let mut world = lua_world.0.borrow_mut();
                let world = &mut *world;
                let joint = RevoluteJointBuilder::new()
                    .local_anchor1(nalgebra::point![0.0, 1.0])
                    .local_anchor2(nalgebra::point![0.0, -3.0])
                    .build();
                let join_handle = world.impulse_joint_set.insert(
                    object1.rigid_body_handle,
                    object2.rigid_body_handle,
                    joint,
                    true,
                );
                Ok(Joint2 {
                    joint: join_handle,
                    world: Rc::downgrade(&lua_world.0),
                })
            }
        });
    })?;

    // MARK: Join2 fn
    lua.register_userdata_type::<Joint2>(|registry| {
        registry.add_method_mut("remove", |_, joint, (): ()| {
            let Some(world) = joint.world.upgrade() else {
                return Ok(());
            };
            let mut world = world.borrow_mut();
            let world = &mut *world;
            world.impulse_joint_set.remove(joint.joint, true);
            Ok(())
        });
        registry.add_method_mut("getObject1", |_, joint, (): ()| {
            let Some(world) = joint.world.upgrade() else {
                return Err(mlua::Error::RuntimeError("Joint is invalid".to_string()));
            };
            let mut world = world.borrow_mut();
            let world = &mut *world;
            let Some(j) = world.impulse_joint_set.get(joint.joint) else {
                return Err(mlua::Error::RuntimeError("Joint is invalid".to_string()));
            };
            Ok(Object2 {
                rigid_body_handle: j.body1,
                world: joint.world.clone(),
            })
        });
        registry.add_method_mut("getObject2", |_, joint, (): ()| {
            let Some(world) = joint.world.upgrade() else {
                return Err(mlua::Error::RuntimeError("Joint is invalid".to_string()));
            };
            let mut world = world.borrow_mut();
            let world = &mut *world;
            let Some(j) = world.impulse_joint_set.get(joint.joint) else {
                return Err(mlua::Error::RuntimeError("Joint is invalid".to_string()));
            };
            Ok(Object2 {
                rigid_body_handle: j.body2,
                world: joint.world.clone(),
            })
        });
    })?;

    // MARK: Collider2 fn
    add_fn_to_table(lua, &physics_module, "newRectangleCollider", {
        move |_, size: Vec2| {
            let collider = ColliderBuilder::cuboid(size.x(), size.y()).build();
            Ok(Collider2 { collider })
        }
    });

    add_fn_to_table(lua, &physics_module, "newCircleCollider", {
        move |_, radius: f32| {
            let collider = ColliderBuilder::ball(radius).build();
            Ok(Collider2 { collider })
        }
    });

    add_fn_to_table(lua, &physics_module, "newPolygonCollider", {
        move |_, points: Vec<Vec2>| {
            let mut converted_points = points // We could probably transmute here, but we won't.
                .iter()
                .map(|p| nalgebra::Point::from(nalgebra::vector![p.x(), p.y()]))
                .collect::<Vec<_>>();
            converted_points.push(converted_points[0]);
            let indices = (0..(points.len() as u32)).map(|i| [i, i + 1]).collect();
            let collider = ColliderBuilder::polyline(converted_points, Some(indices)).build();
            Ok(Collider2 { collider })
        }
    });

    // MARK: Object2 fn
    lua.register_userdata_type::<Object2>(|registry| {
        registry.add_field_method_get("position", |_, object| {
            let translation: Vector<f32> =
                access_rigid_body_mut(object, |_, rigid_body| *rigid_body.translation())?;
            let result = Vec2::new(translation.x, translation.y);
            Ok(result)
        });
        registry.add_field_method_set("position", |_, object, position: Vec2| {
            access_rigid_body_mut(object, |_, rigid_body| {
                rigid_body.set_translation(nalgebra::vector![position.x(), position.y()], true);
            })?;
            Ok(())
        });
        registry.add_field_method_get("speed", |_, object| {
            let speed = access_rigid_body_mut(object, |_, rigid_body| *rigid_body.linvel())?;
            let result = Vec2::new(speed.x, speed.y);
            Ok(result)
        });
        registry.add_field_method_set("speed", |_, object, speed: Vec2| {
            access_rigid_body_mut(object, |_, rigid_body| {
                rigid_body.set_linvel(nalgebra::vector![speed.x(), speed.y()], true);
            })?;
            Ok(())
        });
        registry.add_field_method_get("rotation", |_, object| {
            access_rigid_body_mut(object, |_, rigid_body| rigid_body.rotation().angle())
        });
        registry.add_field_method_set("rotation", |_, object, rotation: f32| {
            access_rigid_body_mut(object, |_, rigid_body| {
                rigid_body.set_rotation(rapier2d::math::Rotation::new(rotation), true);
            })
        });
        registry.add_field_method_get("rotationSpeed", |_, object| {
            access_rigid_body_mut(object, |_, rigid_body| rigid_body.angvel())
        });
        registry.add_field_method_set("rotationSpeed", |_, object, rotation_speed: f32| {
            access_rigid_body_mut(object, |_, rigid_body| {
                rigid_body.set_angvel(rotation_speed, true);
            })?;
            Ok(())
        });
        registry.add_field_method_set("linearDamping", |_, object, damping: f32| {
            access_rigid_body_mut(object, |_, rigid_body| {
                rigid_body.set_linear_damping(damping);
            })?;
            Ok(())
        });
        registry.add_field_method_get("linearDamping", |_, object| {
            access_rigid_body_mut(object, |_, rigid_body| rigid_body.linear_damping())
        });
        registry.add_field_method_set("angularDamping", |_, object, damping: f32| {
            access_rigid_body_mut(object, |_, rigid_body| {
                rigid_body.set_angular_damping(damping);
            })?;
            Ok(())
        });
        registry.add_field_method_get("angularDamping", |_, object| {
            access_rigid_body_mut(object, |_, rigid_body| rigid_body.angular_damping())
        });

        registry.add_method_mut("setRestitution", |_, object, restitution: f32| {
            access_rigid_body_mut(object, |collider_set, rigid_body| {
                rigid_body.colliders().iter().for_each(|collider_handle| {
                    let Some(collider) = collider_set.get_mut(*collider_handle) else {
                        return;
                    };
                    collider.set_restitution(restitution);
                });
            })?;
            Ok(())
        });

        registry.add_method_mut("setMass", |_, object, mass: f32| {
            access_rigid_body_mut(object, |collider_set, rigid_body| {
                rigid_body.colliders().iter().for_each(|collider_handle| {
                    let Some(collider) = collider_set.get_mut(*collider_handle) else {
                        return;
                    };
                    // Under the assumption one collider per body
                    collider.set_mass(mass);
                });
            })?;
            Ok(())
        });

        registry.add_method("setLockRotation", |_, object, lock: bool| {
            access_rigid_body_mut(object, |_, rigid_body| {
                rigid_body.lock_rotations(lock, true)
            })?;
            Ok(())
        });
        registry.add_method("setLockTranslation", |_, object, lock: bool| {
            access_rigid_body_mut(object, |_, rigid_body| {
                rigid_body.lock_translations(lock, true)
            })?;
            Ok(())
        });

        // ---

        registry.add_field_method_get("tags", |_lua, object| {
            access_extras(object, |extra_object_data| {
                Ok(extra_object_data.tags.clone())
            })
            .flatten()
        });
        registry.add_field_method_set("tags", |_, object, tags: mlua::Table| {
            access_extras(object, |extra_object_data| {
                extra_object_data.tags = tags;
            })
        });
        registry.add_field_method_get("extra", |_lua, object| {
            access_extras(object, |extra_object_data| {
                Ok(extra_object_data.extra_custom.clone())
            })
            .flatten()
        });
        registry.add_field_method_set("extra", |_, object, extra: mlua::Value| {
            access_extras(object, |extra_object_data| {
                extra_object_data.extra_custom = extra;
            })
        });
        registry.add_method("getPoints", |lua, object, (): ()| {
            let points = access_rigid_body_mut(object, |collider_set, rigid_body| {
                rigid_body
                    .colliders()
                    .iter()
                    .flat_map(|collider| {
                        let Some(c) = collider_set.get(*collider) else {
                            return Vec::new();
                        };
                        get_points_of_collider(c)
                    })
                    .collect::<Vec<Vec2>>()
            })?;
            let table = lua.create_table()?;
            for point in points {
                table.raw_push(point)?;
            }
            Ok(table)
        });
        registry.add_method("getContacts", |_, object, (): ()| {
            let touching_objects = access_rigid_body(object, |world, rigid_body| {
                let filter = QueryFilter::default();
                let query_pipeline = world.broad_phase.as_query_pipeline(
                    world.narrow_phase.query_dispatcher(),
                    &world.rigid_body_set,
                    &world.collider_set,
                    filter,
                );

                let mut touching_objects = rigid_body
                    .colliders()
                    .iter()
                    .filter_map(|collider| {
                        let c = world.collider_set.get(*collider)?;
                        let intersections =
                            query_pipeline.intersect_shape(*rigid_body.position(), c.shape());
                        Some(intersections.filter_map(|(_, collider)| collider.parent()))
                    })
                    .flatten()
                    .collect::<Vec<_>>();
                touching_objects.sort_by(|a, b| a.0.cmp(&b.0));
                touching_objects.dedup();
                touching_objects
                    .iter()
                    .map(|handle| Object2 {
                        rigid_body_handle: *handle,
                        world: object.world.clone(),
                    })
                    .collect::<Vec<_>>()
            })?;
            Ok(touching_objects)
        });
    })?;

    Ok(physics_module)
}

fn get_points_of_collider(collider: &Collider) -> Vec<Vec2> {
    let shape = collider.shape();
    if let Some(shape) = shape.as_cuboid() {
        shape
            .to_polyline()
            .iter()
            .map(|p| collider.position() * p)
            .map(|p| Vec2::new(p.x, p.y))
            .collect()
    } else if let Some(shape) = shape.as_ball() {
        shape
            .to_polyline(32)
            .iter()
            .map(|p| collider.position() * p)
            .map(|p| Vec2::new(p.x, p.y))
            .collect()
    } else if let Some(shape) = shape.as_polyline() {
        shape
            .vertices()
            .iter()
            .map(|p| collider.position() * p)
            .map(|p| Vec2::new(p.x, p.y))
            .collect()
    } else if let Some(shape) = shape.as_convex_polygon() {
        shape
            .points()
            .iter()
            .map(|p| collider.position() * p)
            .map(|p| Vec2::new(p.x, p.y))
            .collect()
    } else {
        // As a fallback, we use the AABB
        let aabb = shape.compute_aabb(collider.position());
        vec![
            Vec2::new(aabb.maxs.x, aabb.maxs.y),
            Vec2::new(aabb.maxs.x, aabb.mins.y),
            Vec2::new(aabb.mins.x, aabb.mins.y),
            Vec2::new(aabb.mins.x, aabb.maxs.y),
        ]
    }
}

fn access_rigid_body_mut<F, T>(object: &Object2, f: F) -> mlua::Result<T>
where
    F: FnOnce(&mut ColliderSet, &mut RigidBody) -> T,
{
    let maybe_world = object.world.upgrade();
    let Some(world) = maybe_world else {
        return Err(mlua::Error::RuntimeError(
            "Object2 is out of this world".to_string(),
        ));
    };
    let world = &mut *world.borrow_mut();
    let Some(rigid_body) = world.rigid_body_set.get_mut(object.rigid_body_handle) else {
        return Err(mlua::Error::RuntimeError(
            "Object2 is out of this world".to_string(),
        ));
    };
    Ok(f(&mut world.collider_set, rigid_body))
}

fn access_rigid_body<F, T>(object: &Object2, f: F) -> mlua::Result<T>
where
    F: FnOnce(&PhysicsWorld2, &RigidBody) -> T,
{
    let maybe_world = object.world.upgrade();
    let Some(world) = maybe_world else {
        return Err(mlua::Error::RuntimeError(
            "Object2 is out of this world".to_string(),
        ));
    };
    let world = &*world.borrow();
    let Some(rigid_body) = world.rigid_body_set.get(object.rigid_body_handle) else {
        return Err(mlua::Error::RuntimeError(
            "Object2 is out of this world".to_string(),
        ));
    };
    Ok(f(world, rigid_body))
}

fn access_extras<F, T>(object: &Object2, f: F) -> mlua::Result<T>
where
    F: FnOnce(&mut ExtraObjectData) -> T,
{
    let maybe_world = object.world.upgrade();
    let Some(world) = maybe_world else {
        return Err(mlua::Error::RuntimeError(
            "Object2 is out of this world".to_string(),
        ));
    };
    let world = &mut *world.borrow_mut();
    let extras = world.extras.get_mut(&object.rigid_body_handle);
    let Some(extras) = extras else {
        return Err(mlua::Error::RuntimeError(
            "Object2 is out of this world".to_string(),
        ));
    };
    Ok(f(extras))
}
