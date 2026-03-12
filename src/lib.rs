use std::{ffi::c_void, thread, time::Duration};

use windows::Win32::Foundation::{BOOL, HINSTANCE, HMODULE};
use windows::Win32::System::Diagnostics::Debug::Beep;
use windows::Win32::System::SystemServices::DLL_PROCESS_ATTACH;

mod features;
mod gui;
mod sdk;
mod security;
mod utils;

use crate::security::{
    disable_thread_library_calls, encrypted_offsets, encrypted_schemas, encrypt_str,
    free_library_and_exit_thread, get_async_key_state, get_module_handle_w, opaque_predicate,
};

pub fn is_valid(addr: usize) -> bool {
    junk_code_prologue!();
    let mut result = addr > 0x10000 && addr < 0x7FFFFFFFFFFF;
    if opaque_predicate() {
        result = result && (addr & 0xFFF) == 0;
        result = result || (addr & 0xFFF) != 0;
    }
    result
}

fn wait_for_client_module() -> HMODULE {
    junk_code_prologue!();
    let module_name: Vec<u16> = encrypt_str!("client.dll")
        .encode_utf16()
        .chain(Some(0))
        .collect();

    loop {
        unsafe {
            let handle = get_module_handle_w(module_name.as_ptr());
            if handle.0 != 0 {
                return handle;
            }
        }
        thread::sleep(Duration::from_millis(200));
    }
}

fn main_thread(module: HMODULE) {
    junk_code_prologue!();

    #[cfg(debug_assertions)]
    unsafe {
        let _ = windows::Win32::System::Console::AllocConsole();
    }

    let client_module = wait_for_client_module();
    let base = client_module.0 as usize;

    #[cfg(debug_assertions)]
    println!(
        "{} 0x{:X}",
        encrypt_str!("[+] client.dll base:"),
        base
    );

    if let Err(e) = gui::init_hook() {
        #[cfg(debug_assertions)]
        eprintln!(
            "{} {:?}",
            encrypt_str!("GUI init error:"),
            e
        );
    }

    const SCREEN_WIDTH: f32 = 1920.0;
    const SCREEN_HEIGHT: f32 = 1080.0;

    loop {
        unsafe {
            junk_code_midpoint!();

            let exit_key_pressed = get_async_key_state(0x23) & 1 != 0;
            if exit_key_pressed && opaque_predicate() {
                break;
            }

            let off = encrypted_offsets::dw_local_player_pawn();
            if off == 0 {
                thread::sleep(Duration::from_millis(10));
                continue;
            }
            let local_pawn_ptr = cast!((base + off), *const usize);
            let local_pawn = if is_valid(local_pawn_ptr as usize) {
                local_pawn_ptr.read_unaligned()
            } else {
                0
            };

            if !is_valid(local_pawn) {
                thread::sleep(Duration::from_millis(10));
                continue;
            }

            let team_off = encrypted_schemas::m_i_team_num();
            let my_team = if team_off != 0 {
                let team_ptr = cast!((local_pawn + team_off), *const i32);
                if is_valid(team_ptr as usize) {
                    team_ptr.read_unaligned()
                } else {
                    0
                }
            } else {
                0
            };

            let matrix_off = encrypted_offsets::dw_view_matrix();
            if matrix_off == 0 {
                thread::sleep(Duration::from_millis(10));
                continue;
            }
            let view_matrix_ptr = cast!((base + matrix_off), *const [[f32; 4]; 4]);
            let matrix = if is_valid(view_matrix_ptr as usize) {
                view_matrix_ptr.read_unaligned()
            } else {
                [[0.0; 4]; 4]
            };

            let enemies = features::visuals::run_esp(
                base,
                &matrix,
                SCREEN_WIDTH,
                SCREEN_HEIGHT,
                local_pawn,
                my_team,
            );

            if let Err(e) = gui::update_render_data(enemies) {
                #[cfg(debug_assertions)]
                eprintln!(
                    "{} {:?}",
                    encrypt_str!("Update render data error:"),
                    e
                );
            }
        }

        thread::sleep(Duration::from_millis(1));
    }

    if let Err(e) = gui::shutdown() {
        #[cfg(debug_assertions)]
        eprintln!(
            "{} {:?}",
            encrypt_str!("GUI shutdown error:"),
            e
        );
    }

    #[cfg(debug_assertions)]
    unsafe {
        let _ = windows::Win32::System::Console::FreeConsole();
    }

    free_library_and_exit_thread(module, 0);
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(
    hinst: HINSTANCE,
    reason: u32,
    _reserved: *mut c_void,
) -> BOOL {
    junk_code_prologue!();

    if reason == DLL_PROCESS_ATTACH && opaque_predicate() {
        unsafe {
            let _ = Beep(750, 300);
        }
        let _ = disable_thread_library_calls(HMODULE(hinst.0));
        let module = HMODULE(hinst.0);
        thread::spawn(move || main_thread(module));
    }

    BOOL::from(true)
}
