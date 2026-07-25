pub fn inside_triange(
    p: (i32, i32),
    v0: (i32, i32),
    v1: (i32, i32),
    v2: (i32, i32),
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
pub fn cross_product(v0: (i32, i32), v1: (i32, i32), v2: (i32, i32)) -> i32 {
    (v1.0 - v0.0) * (v2.1 - v0.1) - (v1.1 - v0.1) * (v2.0 - v0.0)
}

const DITHER_TABLE: [[i8; 4]; 4] = [
    [-4, 0, -3, 1],
    [2, -2, 3, -1],
    [-3, 1, -4, 0],
    [3, -1, 2, -2],
];

// Color is in rgb
pub fn dither(color: (u8, u8, u8), pixel: (i32, i32)) -> (u8, u8, u8) {
    let offset = DITHER_TABLE[(pixel.0 & 0b11) as usize][(pixel.1 & 0b11) as usize];

    (
        color.0.saturating_add_signed(offset),
        color.1.saturating_add_signed(offset),
        color.2.saturating_add_signed(offset),
    )
}
