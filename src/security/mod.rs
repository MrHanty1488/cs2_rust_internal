use core::arch::asm;
use once_cell::sync::Lazy;
use obfstr::obfstr;
use std::sync::Mutex;
use windows::Win32::Foundation::{FARPROC, HMODULE};

pub use obfstr::obfstr as encrypt_str;

const HASH_SEED: u32 = 0x811C9DC5;

#[inline(always)]
pub const fn hash_api(name: &[u8]) -> u32 {
    let mut hash = HASH_SEED;
    let mut i = 0;
    while i < name.len() {
        hash ^= name[i] as u32;
        hash = hash.wrapping_mul(0x01000193);
        i += 1;
    }
    hash
}

#[macro_export]
macro_rules! hash_api {
    ($name:literal) => {
        $crate::security::hash_api($name.as_bytes())
    };
}

pub struct ApiResolver {
    user32: HMODULE,
    kernel32: HMODULE,
}

fn load_library_a(name: &str) -> HMODULE {
    let name_wide: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
    unsafe {
        windows::Win32::System::LibraryLoader::LoadLibraryW(
            windows::core::PCWSTR(name_wide.as_ptr()),
        )
        .unwrap_or(HMODULE(0))
    }
}

static API_RESOLVER: Lazy<Mutex<ApiResolver>> = Lazy::new(|| {
    let user32 = load_library_a(obfstr!("user32.dll"));
    let kernel32 = load_library_a(obfstr!("kernel32.dll"));
    Mutex::new(ApiResolver { user32, kernel32 })
});

impl ApiResolver {
    #[inline(always)]
    pub fn resolve(&self, module: HMODULE, hash: u32) -> Option<FARPROC> {
        if module.0 == 0 {
            return None;
        }
        let base = module.0 as usize;

        let dos_header = unsafe { *(base as *const u16) };
        if dos_header != 0x5A4D {
            return None;
        }

        let nt_headers_offset = unsafe { *((base + 0x3C) as *const u32) } as usize;
        let nt_headers = (base + nt_headers_offset) as *const u32;
        let export_dir_rva = unsafe { *nt_headers.add(6) } as usize;
        let export_dir = (base + export_dir_rva) as *const u8;

        let num_names = unsafe { *(export_dir.add(24) as *const u32) } as usize;
        let functions_rva = unsafe { *(export_dir.add(28) as *const u32) } as usize;
        let names_rva = unsafe { *(export_dir.add(32) as *const u32) } as usize;
        let ordinals_rva = unsafe { *(export_dir.add(36) as *const u32) } as usize;

        for i in 0..num_names {
            let name_rva = unsafe { *((base + names_rva + i * 4) as *const u32) } as usize;
            let name_ptr = (base + name_rva) as *const u8;

            let mut name_len = 0;
            while unsafe { *name_ptr.add(name_len) } != 0 {
                name_len += 1;
            }

            let name_slice = unsafe { core::slice::from_raw_parts(name_ptr, name_len) };

            if hash_api(name_slice) == hash {
                let ordinal = unsafe {
                    *((base + ordinals_rva + i * 2) as *const u16)
                } as usize;
                let func_rva = unsafe {
                    *((base + functions_rva + ordinal * 4) as *const u32)
                } as usize;
                let func_addr = base + func_rva;
                let func_ptr: unsafe extern "system" fn() -> isize =
                    unsafe { core::mem::transmute(func_addr) };
                return Some(Some(func_ptr));
            }
        }

        None
    }
}

pub type GetAsyncKeyStateFn = unsafe extern "system" fn(i32) -> i16;
pub type GetModuleHandleWFn = unsafe extern "system" fn(*const u16) -> HMODULE;
pub type FreeLibraryAndExitThreadFn = unsafe extern "system" fn(HMODULE, u32);
pub type DisableThreadLibraryCallsFn = unsafe extern "system" fn(HMODULE) -> i32;

const HASH_GETASYNCKEYSTATE: u32 = hash_api(b"GetAsyncKeyState");
const HASH_GETMODULEHANDLEW: u32 = hash_api(b"GetModuleHandleW");
const HASH_FREELIBRARYANDEXITTHREAD: u32 = hash_api(b"FreeLibraryAndExitThread");
const HASH_DISABLETHREADLIBRARYCALLS: u32 = hash_api(b"DisableThreadLibraryCalls");

static GAKS_CACHE: Lazy<Mutex<Option<GetAsyncKeyStateFn>>> = Lazy::new(|| Mutex::new(None));
static GMH_CACHE: Lazy<Mutex<Option<GetModuleHandleWFn>>> = Lazy::new(|| Mutex::new(None));
static FLET_CACHE: Lazy<Mutex<Option<FreeLibraryAndExitThreadFn>>> = Lazy::new(|| Mutex::new(None));
static DTLC_CACHE: Lazy<Mutex<Option<DisableThreadLibraryCallsFn>>> =
    Lazy::new(|| Mutex::new(None));

#[inline(always)]
pub fn get_async_key_state(vkey: i32) -> i16 {
    let mut cache = GAKS_CACHE.lock().unwrap();

    if cache.is_none() {
        let resolver = API_RESOLVER.lock().unwrap();
        if let Some(proc) = resolver.resolve(resolver.user32, HASH_GETASYNCKEYSTATE) {
            *cache = Some(unsafe { core::mem::transmute(proc) });
        }
    }

    match *cache {
        Some(func) => unsafe { func(vkey) },
        None => 0,
    }
}

#[inline(always)]
pub fn get_module_handle_w(module_name: *const u16) -> HMODULE {
    let mut cache = GMH_CACHE.lock().unwrap();

    if cache.is_none() {
        let resolver = API_RESOLVER.lock().unwrap();
        if let Some(proc) = resolver.resolve(resolver.kernel32, HASH_GETMODULEHANDLEW) {
            *cache = Some(unsafe { core::mem::transmute(proc) });
        }
    }

    match *cache {
        Some(func) => unsafe { func(module_name) },
        None => HMODULE(0),
    }
}

#[inline(always)]
pub fn free_library_and_exit_thread(module: HMODULE, exit_code: u32) {
    let mut cache = FLET_CACHE.lock().unwrap();

    if cache.is_none() {
        let resolver = API_RESOLVER.lock().unwrap();
        if let Some(proc) = resolver.resolve(resolver.kernel32, HASH_FREELIBRARYANDEXITTHREAD) {
            *cache = Some(unsafe { core::mem::transmute(proc) });
        }
    }

    if let Some(func) = *cache {
        unsafe { func(module, exit_code) };
    }
}

#[inline(always)]
pub fn disable_thread_library_calls(module: HMODULE) -> bool {
    let mut cache = DTLC_CACHE.lock().unwrap();

    if cache.is_none() {
        let resolver = API_RESOLVER.lock().unwrap();
        if let Some(proc) = resolver.resolve(resolver.kernel32, HASH_DISABLETHREADLIBRARYCALLS) {
            *cache = Some(unsafe { core::mem::transmute(proc) });
        }
    }

    match *cache {
        Some(func) => unsafe { func(module) != 0 },
        None => false,
    }
}

#[inline(always)]
pub fn opaque_predicate() -> bool {
    let mut result: u32 = 0;
    unsafe {
        asm!(
            "mov {0}, 0x12345678",
            "xor {0}, 0x12345678",
            out(reg) result,
        );
    }
    result == 0
}

#[inline(always)]
fn sanitize_sdk_value(value: usize) -> usize {
    if value == 0 {
        0
    } else {
        value
    }
}

pub mod encrypted_offsets {
    use super::sanitize_sdk_value;
    use crate::sdk::offsets::cs2_dumper::offsets::client_dll;

    #[inline(always)]
    pub fn dw_local_player_pawn() -> usize {
        sanitize_sdk_value(client_dll::dwLocalPlayerPawn)
    }

    #[inline(always)]
    pub fn dw_view_matrix() -> usize {
        sanitize_sdk_value(client_dll::dwViewMatrix)
    }

    #[inline(always)]
    pub fn dw_game_entity_system() -> usize {
        sanitize_sdk_value(client_dll::dwGameEntitySystem)
    }

    #[inline(always)]
    pub fn dw_entity_list() -> usize {
        sanitize_sdk_value(client_dll::dwEntityList)
    }

    #[inline(always)]
    pub fn dw_planted_c4() -> usize {
        sanitize_sdk_value(client_dll::dwPlantedC4)
    }

    #[inline(always)]
    pub fn dw_game_rules() -> usize {
        sanitize_sdk_value(client_dll::dwGameRules)
    }

    #[inline(always)]
    pub fn dw_glow_manager() -> usize {
        sanitize_sdk_value(client_dll::dwGlowManager)
    }

    #[inline(always)]
    pub fn dw_global_vars() -> usize {
        sanitize_sdk_value(client_dll::dwGlobalVars)
    }

    #[inline(always)]
    pub fn dw_weapon_c4() -> usize {
        sanitize_sdk_value(client_dll::dwWeaponC4)
    }

    #[inline(always)]
    pub fn dw_local_player_controller() -> usize {
        sanitize_sdk_value(client_dll::dwLocalPlayerController)
    }
}

pub mod encrypted_schemas {
    use super::sanitize_sdk_value;
    use crate::sdk::schemas::cs2_dumper::schemas::{
        base_entity, base_player_pawn, player_controller,
    };

    #[inline(always)]
    pub fn m_i_team_num() -> usize {
        sanitize_sdk_value(base_entity::m_i_team_num)
    }

    #[inline(always)]
    pub fn m_i_health() -> usize {
        sanitize_sdk_value(base_entity::m_i_health)
    }

    #[inline(always)]
    pub fn m_v_old_origin() -> usize {
        sanitize_sdk_value(base_player_pawn::m_v_old_origin)
    }

    #[inline(always)]
    pub fn m_h_player_pawn() -> usize {
        sanitize_sdk_value(player_controller::m_h_player_pawn)
    }

    #[inline(always)]
    pub fn m_p_clipping_weapon() -> usize {
        sanitize_sdk_value(0)
    }

    #[inline(always)]
    pub fn m_ang_eye_angles() -> usize {
        sanitize_sdk_value(0)
    }

    #[inline(always)]
    pub fn m_i_shots_fired() -> usize {
        sanitize_sdk_value(0)
    }

    #[inline(always)]
    pub fn m_aim_punch_angle() -> usize {
        sanitize_sdk_value(0)
    }

    #[inline(always)]
    pub fn m_vec_view_offset() -> usize {
        sanitize_sdk_value(0)
    }
}
