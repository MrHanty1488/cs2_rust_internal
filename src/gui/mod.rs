use crate::features::visuals::PlayerDrawData;
use crate::security::encrypt_str;
use anyhow::Result;
use hudhook::{hooks::dx11::ImguiDx11Hooks, Hudhook, ImguiRenderLoop};
use imgui::*;
use once_cell::sync::Lazy;
use std::sync::Mutex;

static RENDER_DATA: Lazy<Mutex<Vec<PlayerDrawData>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub struct CheatRenderLoop;

impl ImguiRenderLoop for CheatRenderLoop {
    fn render(&mut self, ui: &mut Ui) {
        crate::junk_code_prologue!();

        let enemies = match RENDER_DATA.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };

        if enemies.is_empty() {
            return;
        }

        ui.window(encrypt_str!("##overlay"))
            .flags(
                WindowFlags::NO_TITLE_BAR
                    | WindowFlags::NO_RESIZE
                    | WindowFlags::NO_MOVE
                    | WindowFlags::NO_SCROLLBAR
                    | WindowFlags::NO_BACKGROUND
                    | WindowFlags::NO_INPUTS,
            )
            .position([0.0, 0.0], Condition::Always)
            .size(ui.io().display_size, Condition::Always)
            .build(|| {
                let draw_list = ui.get_background_draw_list();

                for enemy in enemies.iter() {
                    let x = enemy.screen_pos.x;
                    let y = enemy.screen_pos.y;
                    let w = enemy.box_width;
                    let h = enemy.box_height;

                    if x < -w
                        || x > ui.io().display_size[0] + w
                        || y < -h
                        || y > ui.io().display_size[1] + h
                    {
                        continue;
                    }

                    let rect_min = [x - w / 2.0, y - h];
                    let rect_max = [x + w / 2.0, y];

                    draw_list
                        .add_rect(rect_min, rect_max, [1.0, 0.0, 0.0, 1.0])
                        .thickness(2.0)
                        .build();

                    let health_factor = (enemy.health as f32 / 100.0).clamp(0.0, 1.0);
                    let bar_x_min = x - w / 2.0 - 6.0;
                    let bar_x_max = x - w / 2.0 - 2.0;

                    draw_list
                        .add_rect([bar_x_min, y - h], [bar_x_max, y], [0.0, 0.0, 0.0, 0.8])
                        .filled(true)
                        .build();

                    let health_color = [1.0 - health_factor, health_factor, 0.0, 1.0];
                    draw_list
                        .add_rect(
                            [bar_x_min, y - (h * health_factor)],
                            [bar_x_max, y],
                            health_color,
                        )
                        .filled(true)
                        .build();

                    let dist_text = format!("{:.1}m", enemy.distance / 10.0);
                    ui.set_cursor_pos([x - w / 2.0, y + 2.0]);
                    ui.text(dist_text);
                }
            });
    }
}

pub fn init_hook() -> Result<()> {
    crate::junk_code_prologue!();

    std::thread::spawn(|| {
        if let Err(e) = Hudhook::builder()
            .with::<ImguiDx11Hooks>(CheatRenderLoop)
            .build()
            .apply()
        {
            #[cfg(debug_assertions)]
            eprintln!(
                "{} {:?}",
                encrypt_str!("GUI hook error:"),
                e
            );
        }
    });

    Ok(())
}

pub fn update_render_data(enemies: Vec<PlayerDrawData>) -> Result<()> {
    let mut storage = RENDER_DATA
        .lock()
        .map_err(|e| anyhow::anyhow!(
            "{} {:?}",
            encrypt_str!("Lock poisoned:"),
            e
        ))?;
    *storage = enemies;
    Ok(())
}

pub fn shutdown() -> Result<()> {
    let mut storage = RENDER_DATA
        .lock()
        .map_err(|e| anyhow::anyhow!(
            "{} {:?}",
            encrypt_str!("Lock poisoned:"),
            e
        ))?;
    storage.clear();
    Ok(())
}
