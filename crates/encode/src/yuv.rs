//! Fast RGBA/BGRA to YUV420P conversion for OpenH264 pipeline.

/// Converts BGRA slice to standard YUV420P planar buffers (Y, U, V).
pub fn bgra_to_yuv420p(
    bgra: &[u8],
    width: usize,
    height: usize,
    stride: usize,
) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let half_w = (width + 1) / 2;
    let half_h = (height + 1) / 2;

    let mut y_plane = vec![0u8; width * height];
    let mut u_plane = vec![0u8; half_w * half_h];
    let mut v_plane = vec![0u8; half_w * half_h];

    for y in 0..height {
        let row_start = y * stride;
        for x in 0..width {
            let offset = row_start + x * 4;
            let b = bgra[offset] as i32;
            let g = bgra[offset + 1] as i32;
            let r = bgra[offset + 2] as i32;

            // BT.601 standard RGB to YCbCr conversion
            let y_val = ((66 * r + 129 * g + 25 * b + 128) >> 8) + 16;
            y_plane[y * width + x] = y_val.clamp(0, 255) as u8;

            if y % 2 == 0 && x % 2 == 0 {
                let u_val = ((-38 * r - 74 * g + 112 * b + 128) >> 8) + 128;
                let v_val = ((112 * r - 94 * g - 18 * b + 128) >> 8) + 128;
                let uv_idx = (y / 2) * half_w + (x / 2);
                if uv_idx < u_plane.len() {
                    u_plane[uv_idx] = u_val.clamp(0, 255) as u8;
                    v_plane[uv_idx] = v_val.clamp(0, 255) as u8;
                }
            }
        }
    }

    (y_plane, u_plane, v_plane)
}

/// Converts YUV420P planar buffers (Y, U, V) back to BGRA.
pub fn yuv420p_to_bgra(
    y_plane: &[u8],
    u_plane: &[u8],
    v_plane: &[u8],
    width: usize,
    height: usize,
) -> Vec<u8> {
    let half_w = (width + 1) / 2;
    let stride = width * 4;
    let mut bgra = vec![0u8; stride * height];

    for y in 0..height {
        for x in 0..width {
            let y_val = y_plane[y * width + x] as i32 - 16;
            let uv_idx = (y / 2) * half_w + (x / 2);
            let u_val = u_plane[uv_idx] as i32 - 128;
            let v_val = v_plane[uv_idx] as i32 - 128;

            let r = (298 * y_val + 409 * v_val + 128) >> 8;
            let g = (298 * y_val - 100 * u_val - 208 * v_val + 128) >> 8;
            let b = (298 * y_val + 516 * u_val + 128) >> 8;

            let offset = y * stride + x * 4;
            bgra[offset] = b.clamp(0, 255) as u8;
            bgra[offset + 1] = g.clamp(0, 255) as u8;
            bgra[offset + 2] = r.clamp(0, 255) as u8;
            bgra[offset + 3] = 255;
        }
    }

    bgra
}
