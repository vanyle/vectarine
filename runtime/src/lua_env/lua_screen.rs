use std::{cell::RefCell, rc::Rc, sync::Arc};

use crate::{
    auto_impl_lua_clone,
    game_resource::{self, ResourceManager},
    graphics::{batchdraw, glstencil::draw_with_mask},
    io,
    lua_env::{add_fn_to_table, get_internals},
};
use mlua::IntoLua;
use mlua::{FromLua, UserDataMethods};

/// Represents a screen in the game (menu, gameplay, pause, etc.)
#[derive(Clone, PartialEq)]
pub struct Screen {
    name: String,
    draw_fn: mlua::Function,
}
auto_impl_lua_clone!(Screen, Screen);

#[derive(Default)]
pub struct ScreenState {
    pub current_screen: Option<Screen>,
    pub previous_screen: Option<Screen>,
    pub transition: Option<TransitionState>,
}

#[derive(Clone, Debug)]
pub struct TransitionState {
    pub duration: f32,
    pub elapsed: f32,
    pub style: TransitionStyle,
}

#[derive(Clone, Debug)]
pub enum TransitionStyle {
    SlideUp,
    SlideDown,
    Toon,
    Custom(mlua::Function),
}

pub struct RcScreenState(pub Rc<RefCell<ScreenState>>);
impl mlua::UserData for RcScreenState {}

const SCREEN_STATE_KEY: &str = "__screen_state";
pub fn get_screen_state(lua: &mlua::Lua) -> Option<Rc<RefCell<ScreenState>>> {
    let internals = get_internals(lua);
    let value: mlua::Value = internals.raw_get(SCREEN_STATE_KEY).ok()?;
    let rc_screen_state = value.as_userdata()?;
    let rc_screen_state = rc_screen_state.borrow::<RcScreenState>().ok()?;
    Some(rc_screen_state.0.clone())
}

/// Updates the screen transition state. Should be called each frame with delta_time.
pub fn update_screen_transition(lua: &mlua::Lua, delta_time: f32) {
    let Some(screen_state) = get_screen_state(lua) else {
        // Indicates that the Lua code tampered with the internals.
        return;
    };

    let mut state = screen_state.borrow_mut();

    if let Some(ref mut transition) = state.transition {
        transition.elapsed += delta_time;
    }
}

pub fn setup_screen_api(
    lua: &Rc<mlua::Lua>,
    batch: &Rc<RefCell<batchdraw::BatchDraw2d>>,
    _env_state: &Rc<RefCell<io::IoEnvState>>,
    resources: &Rc<game_resource::ResourceManager>,
) -> mlua::Result<mlua::Table> {
    let screen_module = lua.create_table()?;

    let screen_state = Rc::new(RefCell::new(ScreenState::default()));
    get_internals(lua)
        .raw_set(SCREEN_STATE_KEY, RcScreenState(screen_state.clone()))
        .expect("Failed to set screen state");

    lua.register_userdata_type::<Screen>(|registry| {
        registry.add_method("name", |_, this, ()| Ok(this.name.clone()));
        registry.add_method("draw", |_, this, ()| this.draw_fn.call::<()>(()));
        registry.add_meta_function(
            mlua::MetaMethod::Eq,
            |_lua, (id1, id2): (Screen, Screen)| Ok(id1 == id2),
        );
    })?;

    add_fn_to_table(lua, &screen_module, "newScreen", {
        move |_, (name, draw_fn): (String, mlua::Function)| Ok(Screen { name, draw_fn })
    });

    add_fn_to_table(lua, &screen_module, "setCurrentScreen", {
        let screen_state = screen_state.clone();
        move |_, (maybe_screen, transition): (mlua::AnyUserData, Option<mlua::Table>)| {
            let screen = maybe_screen.borrow::<Screen>()?;
            let mut state = screen_state.borrow_mut();

            let transition_state = if let Some(trans_table) = transition {
                let duration = trans_table.get::<f32>("duration")?;
                let style_value: mlua::Value = trans_table.get("transition_style")?;

                let style = match style_value {
                    mlua::Value::String(s) => {
                        let style_str = s.to_str()?;
                        if style_str == "slide_up" {
                            TransitionStyle::SlideUp
                        } else if style_str == "slide_down" {
                            TransitionStyle::SlideDown
                        } else if style_str == "toon" {
                            TransitionStyle::Toon
                        } else {
                            return Err(mlua::Error::RuntimeError(format!(
                                "Unknown transition style: {}",
                                style_str
                            )));
                        }
                    }
                    mlua::Value::Function(f) => TransitionStyle::Custom(f),
                    _ => {
                        return Err(mlua::Error::RuntimeError(
                            "transition_style must be a string or function".to_string(),
                        ));
                    }
                };

                Some(TransitionState {
                    duration,
                    elapsed: 0.0,
                    style,
                })
            } else {
                None
            };

            if transition_state.is_some() {
                state.previous_screen = state.current_screen.clone();
            }

            state.current_screen = Some(screen.clone());
            state.transition = transition_state;

            Ok(())
        }
    });

    add_fn_to_table(lua, &screen_module, "getCurrentScreen", {
        let screen_state = screen_state.clone();
        move |_, ()| {
            let state = screen_state.borrow();
            Ok(state.current_screen.clone())
        }
    });

    add_fn_to_table(lua, &screen_module, "drawCurrentScreen", {
        let gl = batch.borrow().drawing_target.gl().clone();
        let screen_state = screen_state;
        let batch = batch.clone();
        let resources = resources.clone();
        move |_, ()| draw_current_screen_impl(&gl, &screen_state, &batch, &resources)
    });

    Ok(screen_module)
}

fn draw_current_screen_impl(
    gl: &Arc<glow::Context>,
    screen_state: &Rc<RefCell<ScreenState>>,
    batch: &Rc<RefCell<batchdraw::BatchDraw2d>>,
    resources: &Rc<ResourceManager>,
) -> mlua::Result<()> {
    // Update progress and determine if in transition
    let (has_transition, progress, old_screen, new_screen, style) = (|| {
        let mut state = screen_state.borrow_mut();

        let new = state.current_screen.clone();

        if let Some(transition) = state.transition.as_ref() {
            let progress = (transition.elapsed / transition.duration).min(1.0);
            if progress < 1.0 {
                // Transition in progress
                let old = state.previous_screen.clone();
                let style = Some(transition.style.clone());
                return (true, progress, old, new, style);
            } else {
                state.previous_screen = None;
                state.transition = None;
            }
        }
        (false, 0.0, None, new, None)
    })();

    if has_transition {
        match style {
            Some(TransitionStyle::SlideUp) => {
                draw_slide_transition(gl, old_screen, new_screen, progress, 1.0, batch, resources);
            }
            Some(TransitionStyle::SlideDown) => {
                draw_slide_transition(gl, old_screen, new_screen, progress, -1.0, batch, resources);
            }
            Some(TransitionStyle::Toon) => {
                draw_toon_transition(gl, old_screen, new_screen, progress, batch, resources);
            }
            Some(TransitionStyle::Custom(custom_fn)) => {
                custom_fn.call::<()>((old_screen, new_screen, progress))?;
            }
            None => {}
        }
    } else if let Some(ref screen) = new_screen {
        // Just draw current screen
        screen.draw_fn.call::<()>(())?;
    }
    Ok(())
}

fn draw_slide_transition(
    gl: &Arc<glow::Context>,
    old_screen: Option<Screen>,
    new_screen: Option<Screen>,
    progress: f32,
    direction: f32,
    batch: &Rc<RefCell<batchdraw::BatchDraw2d>>,
    resources: &Rc<ResourceManager>,
) {
    let height = progress * 2.0;
    let slide_up = direction > 0.0;

    if let Some(ref screen) = old_screen {
        draw_with_mask(
            gl,
            || {
                batch.borrow_mut().draw_rect(
                    -1.0,
                    -1.0 + height * slide_up as i32 as f32,
                    2.0,
                    2.0 - height,
                    [1.0, 1.0, 1.0, 1.0],
                );
                batch.borrow_mut().draw(resources, true);
            },
            || {
                let _ = screen.draw_fn.call::<()>(());
                batch.borrow_mut().draw(resources, true);
            },
        );
    }

    // Now draw new screen with stencil mask for slide effect
    if let Some(ref screen) = new_screen {
        let y = if slide_up { -1.0 } else { 1.0 - height };
        draw_with_mask(
            gl,
            || {
                batch
                    .borrow_mut()
                    .draw_rect(-1.0, y, 2.0, height, [1.0, 1.0, 1.0, 1.0]);
                batch.borrow_mut().draw(resources, true);
            },
            || {
                let _ = screen.draw_fn.call::<()>(());
                batch.borrow_mut().draw(resources, true);
            },
        );
    }
}

fn draw_toon_transition(
    gl: &Arc<glow::Context>,
    old_screen: Option<Screen>,
    new_screen: Option<Screen>,
    progress: f32,
    batch: &Rc<RefCell<batchdraw::BatchDraw2d>>,
    resources: &Rc<ResourceManager>,
) {
    // Circle wipe effect
    let radius = progress * 2.0;

    if let Some(ref screen) = old_screen {
        let _ = screen.draw_fn.call::<()>(());
        batch.borrow_mut().draw(resources, true);
    }

    if let Some(ref screen) = new_screen {
        draw_with_mask(
            gl,
            || {
                batch
                    .borrow_mut()
                    .draw_circle(0.0, 0.0, radius, [1.0, 1.0, 1.0, 1.0]);
                batch.borrow_mut().draw(resources, true);
            },
            || {
                let _ = screen.draw_fn.call::<()>(());
                batch.borrow_mut().draw(resources, true);
            },
        );
    }
}
