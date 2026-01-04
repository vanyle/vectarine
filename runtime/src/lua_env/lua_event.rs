use std::hash::Hash;
use std::rc::Weak;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{auto_impl_lua_clone, lua_env::add_fn_to_table};
use mlua::FromLua;
use mlua::IntoLua;
use mlua::UserDataFields;
use mlua::UserDataMethods;

#[derive(Clone, Debug)]
pub struct EventType(usize, Weak<RefCell<EventManager>>);
auto_impl_lua_clone!(EventType, EventType);

impl Hash for EventType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
impl PartialEq for EventType {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl Eq for EventType {}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SubscriptionId(usize, EventType);
auto_impl_lua_clone!(SubscriptionId, SubscriptionId);

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

#[derive(Clone)]
pub struct EventManagerRc(Rc<RefCell<EventManager>>);

auto_impl_lua_clone!(EventManagerRc, EventManager);

impl Default for EventManagerRc {
    fn default() -> Self {
        Self(Rc::new(RefCell::new(EventManager {
            registered_events: HashMap::new(),
            event_map: Vec::new(),
        })))
    }
}

impl EventType {
    pub fn trigger(&self, data: mlua::Value) -> mlua::Result<()> {
        // Maybe no-op instead of panic?
        let event_manager = self.1.upgrade().expect("Event manager should exist");
        let event_manager = event_manager.borrow();
        let subscription = event_manager.event_map.get(self.0);
        let Some(subscription) = subscription else {
            return Ok(());
        };
        for callback in subscription.subscriptions.values() {
            let _ = callback.call::<mlua::Value>(data.clone());
        }
        Ok(())
    }
    pub fn clear_subscription(&self) {
        let event_manager = self.1.upgrade().expect("Event manager should exist");
        let mut event_manager = event_manager.borrow_mut();
        let subscriptions = &mut event_manager.event_map;
        let entry = subscriptions
            .get_mut(self.0)
            .expect("Event type should exist");
        entry.subscriptions.clear();
        entry.next_id = 0;
    }
}

pub fn create_event(
    event_manager: &EventManagerRc,
    _lua: &mlua::Lua,
    name: String,
) -> mlua::Result<EventType> {
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
    let event_type = EventType(em.event_map.len(), Rc::downgrade(&event_manager.0));
    em.registered_events
        .insert(name.clone(), event_type.clone());
    em.event_map.push(EventSubscriptions {
        next_id: 0,
        name,
        subscriptions: HashMap::new(),
    });
    Ok(event_type)
}

pub fn create_event_constant_in_event_module(
    event_manager: &EventManagerRc,
    lua: &mlua::Lua,
    name: &str,
    event_module_table: &mlua::Table,
) -> mlua::Result<EventType> {
    let name_with_uppercase_first = format!(
        "{}{}",
        name.chars().next().unwrap_or_default().to_uppercase(),
        &name[1..]
    );
    let constant_name = format!("get{name_with_uppercase_first}Event");
    let name = format!("@vectarine/{name}");
    let event_type = create_event(event_manager, lua, name)?;
    event_module_table.raw_set(
        constant_name,
        lua.create_function({
            let event_type = event_type.clone();
            move |_lua, ()| {
                event_type.clear_subscription();
                Ok(event_type.clone())
            }
        })?,
    )?;
    Ok(event_type)
}

#[derive(Debug, Clone)]
pub struct DefaultEvents {
    pub keydown_event: EventType,
    pub keyup_event: EventType,
    pub text_input_event: EventType,

    pub mouse_down_event: EventType,
    pub mouse_up_event: EventType,
    pub mouse_click_event: EventType,

    pub resource_loaded_event: EventType,
    pub console_command_event: EventType,
}

pub fn setup_event_api(
    lua: &Rc<mlua::Lua>,
) -> mlua::Result<(mlua::Table, DefaultEvents, EventManagerRc)> {
    let event_module = lua.create_table()?;
    let event_manager = EventManagerRc::default();

    lua.register_userdata_type::<EventType>(|registry| {
        registry.add_field_method_get("name", {
            let event_manager = event_manager.clone();
            move |_lua, etype: &EventType| {
                let events = &event_manager.0.borrow().event_map;
                Ok(events
                    .get(etype.0)
                    .map(|e| e.name.clone())
                    .unwrap_or_default())
            }
        });
        registry.add_method("dispatch", {
            move |_lua, event_type, data: mlua::Value| event_type.trigger(data)
        });
        registry.add_method("clear", {
            move |_lua, event_type, ()| {
                event_type.clear_subscription();
                Ok(())
            }
        });
        registry.add_method("on", {
            let event_manager = event_manager.clone();
            move |_lua, event_type, callback: mlua::Function| {
                // We can access the outside using lua.globals()
                let mut subscriptions = event_manager.0.borrow_mut();
                let subscriptions = &mut subscriptions.event_map;
                let entry = subscriptions
                    .get_mut(event_type.0)
                    .expect("Event type should exist");
                let id = SubscriptionId(entry.next_id, event_type.clone());
                entry.next_id += 1;
                entry.subscriptions.insert(id.clone(), callback);
                Ok(id)
            }
        });
    })?;

    lua.register_userdata_type::<SubscriptionId>(|registry| {
        let event_manager = event_manager.clone();
        registry.add_method("unsubscribe", move |_lua, id, ()| {
            let mut em = event_manager.0.borrow_mut();
            let subscriptions = &mut em.event_map;
            let entry = subscriptions
                .get_mut(id.1.0)
                .expect("Event type should exist");
            entry.subscriptions.remove(id);
            Ok(())
        })
    })?;

    add_fn_to_table(lua, &event_module, "newEvent", {
        let event_manager = event_manager.clone();
        move |lua, name: String| create_event(&event_manager, lua, name)
    });

    let keydown_event =
        create_event_constant_in_event_module(&event_manager, lua, "keyDown", &event_module)?;
    let keyup_event =
        create_event_constant_in_event_module(&event_manager, lua, "keyUp", &event_module)?;
    let text_input_event =
        create_event_constant_in_event_module(&event_manager, lua, "textInput", &event_module)?;

    let mouse_down_event =
        create_event_constant_in_event_module(&event_manager, lua, "mouseDown", &event_module)?;
    let mouse_up_event =
        create_event_constant_in_event_module(&event_manager, lua, "mouseUp", &event_module)?;
    let mouse_click_event =
        create_event_constant_in_event_module(&event_manager, lua, "mouseClick", &event_module)?;
    let resource_loaded_event = create_event_constant_in_event_module(
        &event_manager,
        lua,
        "resourceLoaded",
        &event_module,
    )?;
    let console_command_event = create_event_constant_in_event_module(
        &event_manager,
        lua,
        "consoleCommand",
        &event_module,
    )?;

    let default_events = DefaultEvents {
        keydown_event,
        keyup_event,
        mouse_down_event,
        mouse_up_event,
        mouse_click_event,
        resource_loaded_event,
        console_command_event,
        text_input_event,
    };

    Ok((event_module, default_events, event_manager))
}
