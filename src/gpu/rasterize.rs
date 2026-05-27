// pub struct Vert {
//     pub x: u32,
//     pub y: u32,
// }

pub fn inside_triange(
    p: (u32, u32),
    v0: (u32, u32),
    v1: (u32, u32),
    v2: (u32, u32),
) -> Option<[f32; 3]> {
    let mut barycentric_coords = [0.0; 3];

    let denominator = cross_product(v0, v1, v2) as f32;
    if denominator == 0.0 {
        return Some([1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0]);
    }

    for (i, (a, b)) in [(v1, v2), (v2, v0), (v0, v1)].iter().enumerate() {
        let cross_product = cross_product(*a, *b, p);
        barycentric_coords[i] = (cross_product as f32) / denominator;

        if cross_product < 0 {
            return None;
        }

        if cross_product == 0 {
            if b.1 > a.1 {
                return None;
            }

            if b.1 == a.1 && b.0 < a.0 {
                return None;
            }
        }
    }

    Some(barycentric_coords)
}

// Cross product of (v1 - v0) and (v2 - v0)
pub fn cross_product(v0: (u32, u32), v1: (u32, u32), v2: (u32, u32)) -> i32 {
    (v1.0 as i32 - v0.0 as i32) * (v2.1 as i32 - v0.1 as i32)
        - (v1.1 as i32 - v0.1 as i32) * (v2.0 as i32 - v0.0 as i32)
}
