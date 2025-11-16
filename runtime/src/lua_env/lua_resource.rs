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
    registry.add_meta_function(mlua::MetaMethod::ToString, |_lua, (id,): (ResourceId,)| {
        Ok(format!("{}", id))
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
}

// MARK: Script Resource
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct ScriptResourceId(ResourceId);

impl ResourceIdWrapper for ScriptResourceId {
    fn to_resource_id(&self) -> ResourceId {
        self.0
    }
    fn from_id(id: ResourceId) -> Self {
        Self(id)
    }
}

impl IntoLua for ScriptResourceId {
    fn into_lua(self, lua: &mlua::Lua) -> mlua::Result<mlua::Value> {
        lua.create_any_userdata(self).map(mlua::Value::UserData)
    }
}

impl FromLua for ScriptResourceId {
    fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
            _ => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "ScriptResource".to_string(),
                message: Some("Expected ScriptResource userdata".to_string()),
            }),
        }
    }
}
