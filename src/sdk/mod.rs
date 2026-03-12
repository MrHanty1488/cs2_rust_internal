pub mod animationsystem_dll;
pub mod buttons;
pub mod client_dll;
pub mod engine2_dll;
pub mod host_dll;
pub mod interfaces;
pub mod materialsystem2_dll;
pub mod networksystem_dll;
pub mod offsets;
pub mod schemas;
pub mod panorama_dll;
pub mod particles_dll;
pub mod pulse_system_dll;
pub mod rendersystemdx11_dll;
pub mod resourcesystem_dll;
pub mod scenesystem_dll;
pub mod schemasystem_dll;
pub mod soundsystem_dll;
pub mod steamaudio_dll;
pub mod vphysics2_dll;
pub mod worldrenderer_dll;

#[allow(overflowing_literals)]
#[allow(dead_code)]
#[allow(non_upper_case_globals)]
pub mod server_dll {
#![allow(overflowing_literals)]
#![allow(dead_code)]
#![allow(non_upper_case_globals)]

include!(concat!(env!("OUT_DIR"), "/server_dll.rs"));
}
