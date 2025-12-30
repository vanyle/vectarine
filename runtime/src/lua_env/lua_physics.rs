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
        CCDSolver, Collider, ColliderBuilder, ColliderSet, DefaultBroadPhase, ImpulseJointSet,
        IntegrationParameters, IslandManager, MultibodyJointSet, NarrowPhase, PhysicsPipeline,
        QueryFilter, RigidBody, RigidBodyBuilder, RigidBodyHandle, RigidBodySet,
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

// MARK: Object2

struct Object2 {
    rigid_body_handle: RigidBodyHandle,
    world: Weak<RefCell<PhysicsWorld2>>, // maybe weak is better?
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

    add_fn_to_table(lua, &physics_module, "getObjectsAtPoint", {
        move |_, (lua_world, point): (LuaPhysicsWorld2, Vec2)| {
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
                &mut world.multibody_joint_set,
                &mut world.ccd_solver,
                &physics_hooks,
                &event_handler,
            );
            Ok(())
        });

        // MARK: Object2 fn
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
    })?;

    // MARK: Collider2 fn
    add_fn_to_table(lua, &physics_module, "newRectangleCollider", {
        move |_, size: Vec2| {
            let collider = ColliderBuilder::cuboid(size.x(), size.y()).build();
            Ok(Collider2 { collider })
        }
    });

    lua.register_userdata_type::<Object2>(|registry| {
        registry.add_field_method_get("position", |_, object| {
            let translation: Vector<f32> =
                access_rigid_body(object, |_, rigid_body| *rigid_body.translation())?;
            let result = Vec2::new(translation.x, translation.y);
            Ok(result)
        });
        registry.add_field_method_set("position", |_, object, position: Vec2| {
            access_rigid_body(object, |_, rigid_body| {
                rigid_body.set_translation(nalgebra::vector![position.x(), position.y()], true);
            })?;
            Ok(())
        });
        registry.add_field_method_get("speed", |_, object| {
            let speed = access_rigid_body(object, |_, rigid_body| *rigid_body.linvel())?;
            let result = Vec2::new(speed.x, speed.y);
            Ok(result)
        });
        registry.add_field_method_set("speed", |_, object, speed: Vec2| {
            access_rigid_body(object, |_, rigid_body| {
                rigid_body.set_linvel(nalgebra::vector![speed.x(), speed.y()], true);
            })?;
            Ok(())
        });
        registry.add_field_method_get("rotation", |_, object| {
            let rotation =
                access_rigid_body(object, |_, rigid_body| rigid_body.rotation().angle())?;
            Ok(rotation)
        });
        registry.add_field_method_set("rotation", |_, object, rotation: f32| {
            access_rigid_body(object, |_, rigid_body| {
                rigid_body.set_rotation(rapier2d::math::Rotation::new(rotation), true);
            })
        });
        registry.add_field_method_get("rotationSpeed", |_, object| {
            let rotation_speed = access_rigid_body(object, |_, rigid_body| rigid_body.angvel())?;
            Ok(rotation_speed)
        });
        registry.add_field_method_set("rotationSpeed", |_, object, rotation_speed: f32| {
            access_rigid_body(object, |_, rigid_body| {
                rigid_body.set_angvel(rotation_speed, true);
            })?;
            Ok(())
        });
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
            let points = access_rigid_body(object, |collider_set, rigid_body| {
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
            .to_polyline(20)
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

fn access_rigid_body<F, T>(object: &Object2, f: F) -> mlua::Result<T>
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
