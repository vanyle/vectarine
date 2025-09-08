use std::{fs, path::Path, sync::mpsc::channel, thread, time::Duration};

use notify_debouncer_full::{DebounceEventResult, new_debouncer, notify::RecursiveMode};
use runtime::helpers::{
    game::Game,
    lua_env::{LuaEnvironment, run_file_and_display_error},
};
use runtime::init_sdl;

fn main() {
    gui_main();
}

fn gui_main() {
    let (canvas, event_pump) = init_sdl();

    let (debounce_event_sender, debounce_receiver) = channel();

    let mut debouncer = new_debouncer(
        Duration::from_millis(10),
        None,
        move |result: DebounceEventResult| match result {
            Ok(events) => events.iter().for_each(|event| {
                let _ = debounce_event_sender.send(event.clone());
            }),
            Err(errors) => errors.iter().for_each(|error| println!("{error:?}")),
        },
    )
    .unwrap();

    let lua_env = LuaEnvironment::new();

    let lua_for_reload = lua_env.clone();

    let path = Path::new("game.lua");
    let content = fs::read(path);
    if let Ok(content) = content {
        run_file_and_display_error(&lua_for_reload, &content, path);
    }

    thread::spawn(move || {
        loop {
            let event = debounce_receiver.recv();
            if let Ok(event) = event {
                for path in event.event.paths {
                    if path.extension().is_some() && path.extension().unwrap() == "lua" {
                        // println!("Reloading script: {}", path.to_string_lossy());
                        let content = fs::read(&path);
                        let Ok(content) = content else {
                            println!("Failed to read file: {}", path.to_string_lossy());
                            continue;
                        };
                        run_file_and_display_error(&lua_for_reload, &content, &path);
                    }
                }
            }
        }
    });

    debouncer.watch(".", RecursiveMode::NonRecursive).unwrap();

    let mut game = Game::new(canvas, event_pump, lua_env);
    loop {
        game.main_loop();
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}
