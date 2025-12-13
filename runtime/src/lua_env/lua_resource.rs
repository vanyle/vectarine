use std::rc::Rc;

use mlua::{FromLua, IntoLua, UserDataMethods, UserDataRegistry};

use crate::game_resource::{ResourceId, ResourceManager};

pub trait ResourceIdWrapper: std::cmp::Eq + FromLua {
    fn to_resource_id(&self) -> ResourceId;
    fn from_id(id: ResourceId) -> Self;
}

pub fn register_resource_id_methods_on_type<T: ResourceIdWrapper>(
    resources: &Rc<ResourceManager>,
    registry: &mut UserDataRegistry<T>,
) {
    registry.add_meta_function(mlua::MetaMethod::ToString, |_lua, (id,): (T,)| {
        Ok(format!("Resource({})", id.to_resource_id().get_id()))
    });
    registry.add_meta_function(mlua::MetaMethod::Eq, |_lua, (id1, id2): (T, T)| {
        Ok(id1 == id2)
    });
    registry.add_method("getStatus", {
        let resources = resources.clone();
        move |_, id: &T, (): ()| {
            let status = resources.get_holder_by_id(id.to_resource_id()).get_status();
            Ok(status.to_string())
        }
    });
    registry.add_method("isReady", {
        let resources = resources.clone();
        move |_, id: &T, (): ()| Ok(resources.get_holder_by_id(id.to_resource_id()).is_loaded())
    });

    registry.add_method("getId", move |_, id: &T, (): ()| {
        Ok(id.to_resource_id().get_id())
    });
}

/// This macro takes a struct like ScriptResourceId and generates the ResourceIdWrapper, IntoLua and FromLua implementations for it.
/// The only condition is that the struct must be a wrapper around a ResourceId like:
/// pub struct ScriptResourceId(ResourceId);
/// The struct needs to implement "Copy" for this to work.
#[macro_export]
macro_rules! make_resource_lua_compatible {
    ($struct_name:ident) => {
        impl ResourceIdWrapper for $struct_name {
            fn to_resource_id(&self) -> ResourceId {
                self.0
            }
            fn from_id(id: ResourceId) -> Self {
                Self(id)
            }
        }

        impl IntoLua for $struct_name {
            fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
                lua.create_any_userdata(self).map(mlua::Value::UserData)
            }
        }

        impl FromLua for $struct_name {
            fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
                match value {
                    mlua::Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
                    _ => Err(mlua::Error::FromLuaConversionError {
                        from: value.type_name(),
                        to: stringify!($struct_name).to_string(),
                        message: Some(format!("Expected {} userdata", stringify!($expression))),
                    }),
                }
            }
        }
    };
}

/// This macro automatically implements IntoLua and FromLua for a given struct.
/// The struct needs to implement clone for this to work.
/// The second parameter of the macro is the name of the struct in conversion error messages
#[macro_export]
macro_rules! auto_impl_lua {
    ($struct_name:ident, $friendly_name:ident) => {
        impl IntoLua for $struct_name {
            fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
                lua.create_any_userdata(self).map(mlua::Value::UserData)
            }
        }

        impl FromLua for $struct_name {
            fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
                match value {
                    mlua::Value::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
                    _ => Err(mlua::Error::FromLuaConversionError {
                        from: value.type_name(),
                        to: stringify!($friendly_name).to_string(),
                        message: Some(format!("Expected {} userdata", stringify!($friendly_name))),
                    }),
                }
            }
        }
    };
}

/// This macro automatically implements IntoLua and FromLua for a given struct.
/// The struct needs to implement copy for this to work.
/// The second parameter of the macro is the name of the struct in conversion error messages
#[macro_export]
macro_rules! auto_impl_lua_copy {
    ($struct_name:ident, $friendly_name:ident) => {
        impl IntoLua for $struct_name {
            #[inline(always)]
            fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
                lua.create_any_userdata(self).map(mlua::Value::UserData)
            }
        }

        impl FromLua for $struct_name {
            #[inline(always)]
            fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
                match value {
                    mlua::Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
                    _ => Err(mlua::Error::FromLuaConversionError {
                        from: value.type_name(),
                        to: stringify!($friendly_name).to_string(),
                        message: Some(format!("Expected {} userdata", stringify!($friendly_name))),
                    }),
                }
            }
        }
    };
}

// MARK: Script Resource
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct ScriptResourceId(ResourceId);
make_resource_lua_compatible!(ScriptResourceId);
