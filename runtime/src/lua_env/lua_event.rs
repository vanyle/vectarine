use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::lua_env::{add_fn_to_table, get_internals};

#[derive(Clone, Debug, PartialEq, Copy, Hash, Eq)]
pub struct EventType(usize);

#[derive(Clone, Debug, PartialEq, Copy, Hash, Eq)]
pub struct SubscriptionId(usize, EventType);

impl mlua::UserData for SubscriptionId {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("unsubscribe", |lua, id, ()| {
            let event_manager = get_event_manager(lua);
            let mut em = event_manager.0.borrow_mut();
            let subscriptions = &mut em.event_map;
            let entry = subscriptions
                .get_mut(id.1.0)
                .expect("Event type should exist");
            entry.subscriptions.remove(id);
            Ok(())
        });
    }
}
impl mlua::FromLua for SubscriptionId {
    fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
            _ => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "SubscriptionId".to_string(),
                message: Some("Expected SubscriptionId userdata".to_string()),
            }),
        }
    }
}

#[derive(Default)]
pub struct EventSubscriptions {
    next_id: usize,
    name: String,
    subscriptions: HashMap<SubscriptionId, mlua::Function>,
}

struct EventManager {
    registered_events: HashMap<String, EventType>,
    event_map: Vec<EventSubscriptions>,
}
pub struct EventManagerRc(Rc<RefCell<EventManager>>);

const EVENT_MANAGER_KEY: &str = "__event_manager";

impl mlua::UserData for EventManagerRc {}

impl mlua::FromLua for EventManagerRc {
    fn from_lua(value: mlua::Value, _lua: &mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(EventManagerRc(ud.borrow::<Self>()?.0.clone())),
            _ => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "EventManager".to_string(),
                message: Some("expected EventManager userdata".to_string()),
            }),
        }
    }
}

impl Default for EventManagerRc {
    fn default() -> Self {
        Self(Rc::new(RefCell::new(EventManager {
            registered_events: HashMap::new(),
            event_map: Vec::new(),
        })))
    }
}

pub fn get_event_manager(lua: &mlua::Lua) -> EventManagerRc {
    let internals = get_internals(lua);
    internals.get(EVENT_MANAGER_KEY).unwrap()
}

impl mlua::UserData for EventType {
    fn add_fields<F: mlua::UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("name", |lua, etype: &EventType| {
            let event_manager = get_event_manager(lua);
            let events = &event_manager.0.borrow().event_map;
            Ok(events.get(etype.0).unwrap().name.clone())
        });
    }
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("dispatch", |lua, event_type, data: mlua::Value| {
            let event_manager = get_event_manager(lua);
            let subscriptions = event_manager.0.borrow();
            let subscriptions = subscriptions.event_map.get(event_type.0);
            let Some(subscriptions) = subscriptions else {
                return Ok(());
            };
            for callback in subscriptions.subscriptions.values() {
                let _ = callback.call::<mlua::Value>(data.clone());
            }
            Ok(())
        });

        methods.add_method("clear", |lua, event_type, ()| {
            let event_manager = get_event_manager(lua);
            let mut em = event_manager.0.borrow_mut();
            let subscriptions = &mut em.event_map;
            let entry = subscriptions
                .get_mut(event_type.0)
                .expect("Event type should exist");
            entry.subscriptions.clear();
            entry.next_id = 0;
            Ok(())
        });

        methods.add_method("on", |lua, event_type, callback: mlua::Function| {
            // We can access the outside using lua.globals()
            let event_manager = get_event_manager(lua);
            let mut subscriptions = event_manager.0.borrow_mut();
            let subscriptions = &mut subscriptions.event_map;
            let entry = subscriptions
                .get_mut(event_type.0)
                .expect("Event type should exist");
            let id = SubscriptionId(entry.next_id, *event_type);
            entry.next_id += 1;
            entry.subscriptions.insert(id, callback);
            Ok(id)
        });
    }
}

impl mlua::FromLua for EventType {
    fn from_lua(value: mlua::Value, _: &mlua::Lua) -> mlua::Result<Self> {
        match value {
            mlua::Value::UserData(ud) => Ok(*ud.borrow::<Self>()?),
            _ => Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "Event".to_string(),
                message: Some("expected Event userdata".to_string()),
            }),
        }
    }
}

pub fn setup_event_api(lua: &Rc<mlua::Lua>) -> mlua::Result<mlua::Table> {
    let internals = get_internals(lua);

    internals.set(EVENT_MANAGER_KEY, EventManagerRc::default())?;

    let event_module = lua.create_table()?;
    add_fn_to_table(lua, &event_module, "newEvent", |lua, name: String| {
        let event_manager = get_event_manager(lua);
        let mut em = event_manager.0.borrow_mut();
        {
            let entry = em.registered_events.get(&name).cloned();
            if let Some(event_type) = entry {
                if let Some(subs) = em.event_map.get_mut(event_type.0) {
                    subs.subscriptions.clear();
                    subs.next_id = 0;
                }
                return Ok(event_type);
            }
        }
        let event_type = EventType(em.event_map.len());
        em.registered_events.insert(name.clone(), event_type);
        em.event_map.push(EventSubscriptions {
            next_id: 0,
            name: name.clone(),
            subscriptions: HashMap::new(),
        });
        Ok(event_type)
    });

    Ok(event_module)
}
