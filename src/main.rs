#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]
use std::{
    process::exit,
    thread::{sleep, spawn},
    time::Duration,
};

use raylib::prelude::*;
use rdev::{listen, simulate, EventType, Key};
use translate_rlib_rdev_events::map_raylib_key_to_rdev_key;

static mut KEYBOARD_KEY: Option<Key> = None;
static mut MOUSE_BUTTON: Option<rdev::Button> = Some(rdev::Button::Left);
fn mouse_button(button: i32) {
    unsafe {
        MOUSE_BUTTON = [
            rdev::Button::Left,
            rdev::Button::Right,
            rdev::Button::Middle,
            rdev::Button::Unknown(2), // Forward
            rdev::Button::Unknown(1), // Back
        ]
        .get(button as usize)
        .copied()
    }
}

static mut MOUSE_BIND: Option<Key> = None;
static mut KEYBOARD_BIND: Option<Key> = None;

enum CaptureBindMode {
    None,
    KbBind,
    KbButton,
    MouseBind,
}

fn set_key(bind: *mut Option<Key>, d: &mut RaylibDrawHandle, bind_mode: &mut CaptureBindMode) {
    // Press a key mode

    d.gui_window_box(rrect(8, 0, 248, 168), "ALERT!");
    d.gui_dummy_rec(rrect(16, 32, 232, 128), "PRESS A KEY TO CONTINUE");
    if let Some(key) = d.get_key_pressed() {
        *bind_mode = CaptureBindMode::None;
        unsafe {
            *bind = Some(map_raylib_key_to_rdev_key(key));
        }
    }
}

fn send(event_type: &EventType) {
    if let Err(e) = simulate(event_type) {
        eprintln!("{e}");
    }
}

mod translate_rlib_rdev_events;

static mut IS_WINDOW_FOCUSED: bool = false;
fn is_window_focused() -> bool {
    // Safety: None, I'm a maniac
    unsafe { IS_WINDOW_FOCUSED }
}

static mut KEYBOARD_TOGGLE: bool = false;
static mut MOUSE_TOGGLE: bool = false;

fn main() {
    spawn(|| {
        init_gui();
    });

    spawn(|| unsafe {
        kb_event_simulator();
    });

    spawn(|| unsafe {
        mouse_event_simulator();
    });

    loop {
        listen(|e| {
            if let EventType::KeyPress(key) = e.event_type {
                let kb_bind = unsafe { KEYBOARD_BIND };
                let mouse_bind = unsafe { MOUSE_BIND };

                #[allow(static_mut_refs)] // Reading only after mutating
                match (kb_bind, mouse_bind) {
                    (Some(kb_bind), _) if key == kb_bind => unsafe {
                        KEYBOARD_TOGGLE = !KEYBOARD_TOGGLE;
                    },
                    (_, Some(mouse_bind)) if key == mouse_bind => unsafe {
                        MOUSE_TOGGLE = !MOUSE_TOGGLE;
                    },
                    _ => (),
                }
            }
        })
        .ok();
    }
}

// Microseconds
static mut KB_DELAY: u64 = 100_000;
unsafe fn kb_event_simulator() {
    loop {
        if KEYBOARD_TOGGLE && !is_window_focused() {
            if let Some(key) = KEYBOARD_KEY {
                send(&EventType::KeyPress(key));
                send(&EventType::KeyRelease(key));
                sleep(Duration::from_micros(KB_DELAY))
            } else {
                sleep(Duration::from_millis(1));
            }
        } else {
            sleep(Duration::from_millis(1));
        }
    }
}

// Microseconds
static mut MOUSE_DELAY: u64 = 100_000;
unsafe fn mouse_event_simulator() {
    loop {
        if MOUSE_TOGGLE && !is_window_focused() {
            if let Some(button) = MOUSE_BUTTON {
                send(&EventType::ButtonPress(button));
                send(&EventType::ButtonRelease(button));
                sleep(Duration::from_micros(MOUSE_DELAY));
            } else {
                sleep(Duration::from_millis(1));
            }
        } else {
            sleep(Duration::from_millis(1));
        }
    }
}

fn init_gui() {
    let screen_width = 264;
    let screen_height = 200;

    let height2 = 280;

    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .title("rMultiClicker")
        .build();

    let mouse_delay_text = "Mouse Delay";
    let keybinds_text = "Keybinds";
    let kb_delay_text = "Keyboard Delay";
    let buttons_text = "Buttons";
    let mouse_button_dropdown_text = "Mouse1;Mouse2;Mouse3;Forward;Back";
    let mut kb_button_text = String::from("KB: Press to set");
    let mut mouse_bind_text = String::from("M: Press to set");
    let mut kb_bind_text = String::from("KB: Press to set");
    let mut state_label_text;

    let main = Vector2::new(0.0, 0.0);

    let mut mouse_minutes_edit_mode = false;
    let mut mouse_minutes_value = 0;
    let mut mouse_seconds_edit_mode = false;
    let mut mouse_seconds_value = 0;
    let mut mouse_milliseconds_edit_mode = false;
    let mut mouse_milliseconds_value = 100;
    let mut mouse_microseconds_edit_mode = false;
    let mut mouse_microseconds_value = 0;
    let mut kb_minutes_edit_mode = false;
    let mut kb_minutes_value = 0;
    let mut kb_seconds_edit_mode = false;
    let mut kb_seconds_value = 0;
    let mut kb_milliseconds_edit_mode = false;
    let mut kb_milliseconds_value = 100;
    let mut kb_microseconds_edit_mode = false;
    let mut kb_microseconds_value = 0;
    let mut mouse_button_dropdown_edit_mode = false;
    let mut mouse_button_dropdown_active = 0;

    rl.set_target_fps(60);
    rl.gui_load_style("style.rgs");

    let mut capture_bind_mode = CaptureBindMode::None;

    // Main game loop
    while !rl.window_should_close() {
        // Safety: Concurrency isn't an issue here.
        unsafe {
            IS_WINDOW_FOCUSED = rl.is_window_focused();
        }

        state_label_text = unsafe {
            match (KEYBOARD_TOGGLE, MOUSE_TOGGLE) {
                (true, true) => "KB: On  | Mouse: On ",
                (true, false) => "KB: On  | Mouse: Off",
                (false, true) => "KB: Off | Mouse: On ",
                (false, false) => "KB: Off | Mouse: Off",
            }
        };

        unsafe {
            KB_DELAY = (kb_microseconds_value
                + kb_milliseconds_value * 1000
                + kb_seconds_value * 1_000_000
                + kb_minutes_value * 60_000_000) as u64;
            MOUSE_DELAY = (mouse_microseconds_value
                + mouse_milliseconds_value * 1000
                + mouse_seconds_value * 1_000_000
                + mouse_minutes_value * 60_000_000) as u64;
        }

        unsafe {
            if let Some(key) = KEYBOARD_KEY {
                kb_button_text = format!("KB: {:?}", key);
            }
            if let Some(bind) = KEYBOARD_BIND {
                kb_bind_text = format!("KB: {:?}", bind);
            }
            if let Some(bind) = MOUSE_BIND {
                mouse_bind_text = format!(" M: {:?}", bind);
            }
        }

        // Draw
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::get_color(
            d.gui_get_style(GuiControl::DEFAULT, GuiDefaultProperty::BACKGROUND_COLOR) as u32,
        ));

        match capture_bind_mode {
            CaptureBindMode::None => (),
            CaptureBindMode::KbBind => {
                set_key(&raw mut KEYBOARD_BIND, &mut d, &mut capture_bind_mode);
                continue;
            }
            CaptureBindMode::KbButton => {
                set_key(&raw mut KEYBOARD_KEY, &mut d, &mut capture_bind_mode);
                continue;
            }
            CaptureBindMode::MouseBind => {
                set_key(&raw mut MOUSE_BIND, &mut d, &mut capture_bind_mode);
                continue;
            }
        }

        if mouse_button_dropdown_edit_mode {
            d.gui_lock();
            d.set_window_size(screen_width, height2);
        } else {
            d.set_window_size(screen_width, screen_height);
        }

        d.gui_group_box(
            Rectangle::new(main.x + 8.0, main.y + 8.0, 120.0, 80.0),
            mouse_delay_text,
        );
        if d.gui_value_box(
            Rectangle::new(main.x + 96.0, main.y + 16.0, 24.0, 16.0),
            "Minutes",
            &mut mouse_minutes_value,
            0,
            60,
            mouse_minutes_edit_mode,
        ) {
            mouse_minutes_edit_mode = !mouse_minutes_edit_mode;
        }
        if d.gui_value_box(
            Rectangle::new(main.x + 96.0, main.y + 32.0, 24.0, 16.0),
            "Seconds",
            &mut mouse_seconds_value,
            0,
            60,
            mouse_seconds_edit_mode,
        ) {
            mouse_seconds_edit_mode = !mouse_seconds_edit_mode;
        }
        if d.gui_value_box(
            Rectangle::new(main.x + 96.0, main.y + 48.0, 24.0, 16.0),
            "Milliseconds",
            &mut mouse_milliseconds_value,
            0,
            999,
            mouse_milliseconds_edit_mode,
        ) {
            mouse_milliseconds_edit_mode = !mouse_milliseconds_edit_mode;
        }
        if d.gui_value_box(
            Rectangle::new(main.x + 96.0, main.y + 64.0, 24.0, 16.0),
            "Microseconds",
            &mut mouse_microseconds_value,
            if mouse_minutes_value == 0 && mouse_seconds_value == 0 && mouse_milliseconds_value == 0
            {
                100
            } else {
                0
            },
            999,
            mouse_microseconds_edit_mode,
        ) {
            mouse_microseconds_edit_mode = !mouse_microseconds_edit_mode;
        }

        d.gui_group_box(
            Rectangle::new(main.x + 8.0, main.y + 96.0, 120.0, 80.0),
            keybinds_text,
        );
        if d.gui_button(
            Rectangle::new(main.x + 16.0, main.y + 112.0, 104.0, 24.0),
            &mouse_bind_text,
        ) {
            capture_bind_mode = CaptureBindMode::MouseBind;
        }
        if d.gui_button(
            Rectangle::new(main.x + 16.0, main.y + 144.0, 104.0, 24.0),
            &kb_bind_text,
        ) {
            capture_bind_mode = CaptureBindMode::KbBind;
        }

        d.gui_group_box(
            Rectangle::new(main.x + 136.0, main.y + 8.0, 120.0, 80.0),
            kb_delay_text,
        );
        if d.gui_value_box(
            Rectangle::new(main.x + 224.0, main.y + 16.0, 24.0, 16.0),
            "Minutes",
            &mut kb_minutes_value,
            0,
            60,
            kb_minutes_edit_mode,
        ) {
            kb_minutes_edit_mode = !kb_minutes_edit_mode;
        }
        if d.gui_value_box(
            Rectangle::new(main.x + 224.0, main.y + 32.0, 24.0, 16.0),
            "Seconds",
            &mut kb_seconds_value,
            0,
            60,
            kb_seconds_edit_mode,
        ) {
            kb_seconds_edit_mode = !kb_seconds_edit_mode;
        }
        if d.gui_value_box(
            Rectangle::new(main.x + 224.0, main.y + 48.0, 24.0, 16.0),
            "Milliseconds",
            &mut kb_milliseconds_value,
            0,
            999,
            kb_milliseconds_edit_mode,
        ) {
            kb_milliseconds_edit_mode = !kb_milliseconds_edit_mode;
        }
        if d.gui_value_box(
            Rectangle::new(main.x + 224.0, main.y + 64.0, 24.0, 16.0),
            "Microseconds",
            &mut kb_microseconds_value,
            if kb_minutes_value == 0 && kb_seconds_value == 0 && kb_milliseconds_value == 0 {
                100
            } else {
                0
            },
            999,
            kb_microseconds_edit_mode,
        ) {
            kb_microseconds_edit_mode = !kb_microseconds_edit_mode;
        }

        d.gui_group_box(
            Rectangle::new(main.x + 136.0, main.y + 96.0, 120.0, 80.0),
            buttons_text,
        );
        if d.gui_button(
            Rectangle::new(main.x + 144.0, main.y + 144.0, 104.0, 24.0),
            &kb_button_text,
        ) {
            capture_bind_mode = CaptureBindMode::KbButton;
        }
        if d.gui_dropdown_box(
            Rectangle::new(main.x + 144.0, main.y + 112.0, 104.0, 24.0),
            mouse_button_dropdown_text,
            &mut mouse_button_dropdown_active,
            mouse_button_dropdown_edit_mode,
        ) {
            mouse_button_dropdown_edit_mode = !mouse_button_dropdown_edit_mode;
            mouse_button(mouse_button_dropdown_active);
        }

        d.gui_label(
            rrect(main.x + 8.0, main.y + 176.0, 200, 24),
            state_label_text,
        );

        d.gui_unlock();
    }
    exit(0)
}
