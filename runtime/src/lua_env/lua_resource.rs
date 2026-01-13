use std::rc::Rc;

use vectarine_plugin_sdk::mlua::{FromLua, IntoLua, UserDataMethods, UserDataRegistry};

use crate::game_resource::{ResourceId, ResourceManager};

pub trait ResourceIdWrapper: std::cmp::Eq + FromLua {
    fn to_resource_id(&self) -> ResourceId;
    fn from_id(id: ResourceId) -> Self;
}

pub fn register_resource_id_methods_on_type<T: ResourceIdWrapper>(
    resources: &Rc<ResourceManager>,
    registry: &mut UserDataRegistry<T>,
) {
    registry.add_meta_function(vectarine_plugin_sdk::mlua::MetaMethod::ToString, |_lua, (id,): (T,)| {
        Ok(format!("Resource({})", id.to_resource_id().get_id()))
    });
    registry.add_meta_function(vectarine_plugin_sdk::mlua::MetaMethod::Eq, |_lua, (id1, id2): (T, T)| {
        Ok(id1.to_resource_id().get_id() == id2.to_resource_id().get_id())
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
            fn into_lua(self, lua: &vectarine_plugin_sdk::mlua::Lua) -> vectarine_plugin_sdk::mlua::Result<vectarine_plugin_sdk::mlua::Value> {
                lua.create_any_userdata(self).map(vectarine_plugin_sdk::mlua::Value::UserData)
            }
        }

        impl FromLua for $struct_name {
            fn from_lua(value: vectarine_plugin_sdk::mlua::Value, _: &vectarine_plugin_sdk::mlua::Lua) -> vectarine_plugin_sdk::mlua::Result<Self> {
                match value {
                    vectarine_plugin_sdk::mlua::Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
                    _ => Err(vectarine_plugin_sdk::mlua::Error::FromLuaConversionError {
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
/// The second parameter of the macro is the name of the struct in conversion error messages
/// It does so by taking ownership from Lua when passing the data to Rust.
/// This is unsafe. If you use this, never us this data as an direct argument to Lua function, only methods.
/// ```lua
/// taking_object:my_method("hello", 1) -- This is OK
/// my_function(taking_object, "hello", 1) -- You need to be careful with this! You'll need to manually borrow userdata.
/// ```
#[macro_export]
macro_rules! auto_impl_lua_take {
    ($struct_name:ident, $friendly_name:ident) => {
        impl IntoLua for $struct_name {
            fn into_lua(self, lua: &vectarine_plugin_sdk::mlua::Lua) -> vectarine_plugin_sdk::mlua::Result<vectarine_plugin_sdk::mlua::Value> {
                lua.create_any_userdata(self).map(vectarine_plugin_sdk::mlua::Value::UserData)
            }
        }

        impl FromLua for $struct_name {
            fn from_lua(value: vectarine_plugin_sdk::mlua::Value, _: &vectarine_plugin_sdk::mlua::Lua) -> vectarine_plugin_sdk::mlua::Result<Self> {
                match value {
                    // this is probably buggy, take can cause issues.
                    vectarine_plugin_sdk::mlua::Value::UserData(ud) => Ok(ud.take::<Self>()?),
                    _ => Err(vectarine_plugin_sdk::mlua::Error::FromLuaConversionError {
                        from: value.type_name(),
                        to: stringify!($friendly_name).to_string(),
                        message: Some(format!("Expected {} userdata", stringify!($friendly_name))),
                    }),
                }
            }
        }
    };
}

#[macro_export]
macro_rules! auto_impl_lua_clone {
    ($struct_name:ident, $friendly_name:ident) => {
        impl IntoLua for $struct_name {
            fn into_lua(self, lua: &vectarine_plugin_sdk::mlua::Lua) -> vectarine_plugin_sdk::mlua::Result<vectarine_plugin_sdk::mlua::Value> {
                lua.create_any_userdata(self).map(vectarine_plugin_sdk::mlua::Value::UserData)
            }
        }

        impl FromLua for $struct_name {
            fn from_lua(value: vectarine_plugin_sdk::mlua::Value, _: &vectarine_plugin_sdk::mlua::Lua) -> vectarine_plugin_sdk::mlua::Result<Self> {
                match value {
                    // this is probably buggy, take can cause issues.
                    vectarine_plugin_sdk::mlua::Value::UserData(ud) => Ok(ud.borrow::<Self>()?.clone()),
                    _ => Err(vectarine_plugin_sdk::mlua::Error::FromLuaConversionError {
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
            fn into_lua(self, lua: &vectarine_plugin_sdk::mlua::Lua) -> vectarine_plugin_sdk::mlua::Result<vectarine_plugin_sdk::mlua::Value> {
                lua.create_any_userdata(self).map(vectarine_plugin_sdk::mlua::Value::UserData)
            }
        }

        impl FromLua for $struct_name {
            #[inline(always)]
            fn from_lua(value: vectarine_plugin_sdk::mlua::Value, _: &vectarine_plugin_sdk::mlua::Lua) -> vectarine_plugin_sdk::mlua::Result<Self> {
                match value {
                    vectarine_plugin_sdk::mlua::Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
                    _ => Err(vectarine_plugin_sdk::mlua::Error::FromLuaConversionError {
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
