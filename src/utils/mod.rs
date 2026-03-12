use glam::{Vec2, Vec3};

pub mod common;
pub mod mutation;

pub use common::{is_valid_ptr, ptr_add, ptr_add_mut, read_unaligned};

// Преобразует 3d координаты из мира в 2d координаты экрана
//
//Параметры
// - pos: позиция в мировом пространстве
// - matrix: матрица вид-проекции (4x4)
// - width: ширина экрана в пикселях
// - height: высота экрана в пикселях
//
//Возвращает
//Some(Vec2) если объект видим. none если за спиной камеры
pub fn world_to_screen(pos: Vec3, matrix: &[[f32; 4]; 4], width: f32, height: f32) -> Option<Vec2> {
    //применяем матрицу к позиции
    let w = matrix[3][0] * pos.x + matrix[3][1] * pos.y + matrix[3][2] * pos.z + matrix[3][3];

    //объект за спиной или слишком близко
    if w < 0.001 {
        return None;
    }

    //вычисляем экранные координаты (нормализованные)
    let x = (matrix[0][0] * pos.x + matrix[0][1] * pos.y + matrix[0][2] * pos.z + matrix[0][3]) / w;
    let y = (matrix[1][0] * pos.x + matrix[1][1] * pos.y + matrix[1][2] * pos.z + matrix[1][3]) / w;

    //преобразуем в пиксельные координаты экрана
    let nx = (width / 2.0) + (x * width / 2.0);
    let ny = (height / 2.0) - (y * height / 2.0);

    //проверяем что точка в пределах экрана
    if nx < 0.0 || nx > width || ny < 0.0 || ny > height {
        return None;
    }

    Some(Vec2::new(nx, ny))
}

//рассчет расстояния между двумя 3d точками
pub fn distance_3d(p1: Vec3, p2: Vec3) -> f32 {
    (p1 - p2).length()
}

// получить угол между двумя 3d векторами в градусах
pub fn angle_between(from: Vec3, to: Vec3) -> f32 {
    let dot = from.normalize().dot(to.normalize()).clamp(-1.0, 1.0);
    dot.acos().to_degrees()
}
