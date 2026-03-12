use crate::cast;
use crate::is_valid;
use crate::junk_code_midpoint;
use crate::junk_code_prologue;
use crate::security::{encrypted_offsets, encrypted_schemas};
use glam::{Vec2, Vec3};

/// Данная структура описывает данные для отрисовки игрока.
#[derive(Clone, Copy, Debug)]
pub struct PlayerDrawData {
    pub screen_pos: Vec2,
    pub health: i32,
    pub box_width: f32,
    pub box_height: f32,
    pub distance: f32,
}

/// Возвращает список игроков, которых стоит отрисовать на экране.
///
/// Функция собирает информацию о живых игроках противника, рассчитывает положение
/// на экране и группыibox'ов для HUD.
pub unsafe fn run_esp(
    base: usize,
    matrix: &[[f32; 4]; 4],
    screen_width: f32,
    screen_height: f32,
    local_pawn: usize,
    my_team: i32,
) -> Vec<PlayerDrawData> {
    junk_code_prologue!();
    let mut render_list = Vec::new();

    let entity_system_off = encrypted_offsets::dw_game_entity_system();
    let entity_system = cast!((base + entity_system_off), *const usize).read_unaligned();

    if !is_valid(entity_system) {
        return render_list;
    }

    for i in 1..64 {
        junk_code_midpoint!();

        let list_entry = cast!(
            (entity_system + (8 * (i as usize & 0x7FFF) >> 9) + 16),
            *const usize
        )
        .read_unaligned();
        if !is_valid(list_entry) {
            continue;
        }

        let controller = cast!((list_entry + 120 * (i as usize & 0x1FF)), *const usize)
            .read_unaligned();
        if !is_valid(controller) {
            continue;
        }

        let pawn_handle_off = encrypted_schemas::m_h_player_pawn();
        let pawn_handle = cast!((controller + pawn_handle_off), *const u32).read_unaligned();
        if pawn_handle == 0 || pawn_handle == 0xFFFFFFFF {
            continue;
        }

        let p_list_entry = cast!(
            (entity_system + 8 * ((pawn_handle as usize & 0x7FFF) >> 9) + 16),
            *const usize
        )
        .read_unaligned();
        if !is_valid(p_list_entry) {
            continue;
        }

        let p_pawn =
            cast!((p_list_entry + 120 * (pawn_handle as usize & 0x1FF)), *const usize)
                .read_unaligned();
        if !is_valid(p_pawn) || p_pawn == local_pawn {
            continue;
        }

        let health_off = encrypted_schemas::m_i_health();
        let team_off = encrypted_schemas::m_i_team_num();
        let health_ptr = cast!((p_pawn + health_off), *const i32);
        if !is_valid(health_ptr as usize) {
            continue;
        }
        let health = health_ptr.read_unaligned();
        let team_ptr = cast!((p_pawn + team_off), *const i32);
        if !is_valid(team_ptr as usize) {
            continue;
        }
        let team = team_ptr.read_unaligned();

        if health > 0 && health <= 100 && team != my_team {
            let origin_off = encrypted_schemas::m_v_old_origin();
            let origin_ptr = cast!((p_pawn + origin_off), *const [f32; 3]);
            if !is_valid(origin_ptr as usize) {
                continue;
            }
            let pos_arr = origin_ptr.read_unaligned();
            let world_pos = Vec3::new(pos_arr[0], pos_arr[1], pos_arr[2]);

            let local_pawn_off = encrypted_offsets::dw_local_player_pawn();
            let local_pawn_ptr = cast!((base + local_pawn_off), *const usize);
            let local_pos_arr = if is_valid(local_pawn_ptr as usize) {
                cast!(((*local_pawn_ptr) + origin_off), *const [f32; 3]).read_unaligned()
            } else {
                [0.0; 3]
            };
            let local_world_pos = Vec3::new(local_pos_arr[0], local_pos_arr[1], local_pos_arr[2]);
            let distance = crate::utils::distance_3d(world_pos, local_world_pos);

            if let Some(screen_pos) = crate::utils::world_to_screen(
                world_pos,
                matrix,
                screen_width,
                screen_height,
            ) {
                let scale = 1000.0 / distance;
                let box_h = 2.0 * scale;
                let box_w = box_h / 2.0;

                let draw_data = PlayerDrawData {
                    screen_pos,
                    health,
                    box_width: box_w,
                    box_height: box_h,
                    distance,
                };
                render_list.push(draw_data);
            }
        }
    }

    render_list
}
