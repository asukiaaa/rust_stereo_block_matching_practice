extern crate image;
#[macro_use(s)]
extern crate ndarray;
use image::{GenericImage, RgbImage};
use ndarray::{Array, Array2};
use std::ops::Sub;

fn get_gray_mat(file_name: &str) -> Array2<f32> {
    let img = image::open(file_name).unwrap().grayscale();
    let w = img.width(); // / 2;
    let h = img.height(); // / 2;
    // let img = img.resize(w, h, FilterType::Gaussian);
    let pixels = img.raw_pixels().into_iter().map(|p| p as f32).collect();
    Array::from_vec(pixels)
        .into_shape((h as usize, w as usize))
        .unwrap()
}

fn mat_wh(mat: &Array2<f32>) -> (usize, usize) {
    let shape = mat.shape();
    (shape[1], shape[0])
}

fn fill_vec(target_vec: &mut Vec<f32>, value: f32, start_x: usize, start_y: usize, fill_w: usize, fill_h: usize, line_w: usize, line_h: usize) {
    for i in start_y.. start_y + fill_h {
        if i >= line_h { continue }
        for j in start_x.. start_x + fill_w {
            if j >= line_w { continue }
            let index = i*line_w + j;
            target_vec[index] = value;
        }
    }
}

fn get_diff_point(left_mat: &Array2<f32>, right_mat: &Array2<f32>, block_w: usize, block_h: usize, left_x: usize, left_y: usize, right_x: usize, right_y: usize) -> f32 {
    let (w, h) = mat_wh(&left_mat);
    if left_x + block_w >= w || right_x + block_w >= w ||
    left_y + block_h >= h || right_y + block_h >= h {
        return std::f32::MAX;
    }
    let left_block = left_mat.slice(s![left_y .. left_y + block_h, left_x .. left_x + block_w]);
    let right_block = right_mat.slice(s![right_y .. right_y + block_h, right_x .. right_x + block_w]);
    let point = left_block.sub(&right_block).mapv(f32::abs).scalar_sum();
    point
}

fn block_match(left_mat: &Array2<f32>, right_mat: &Array2<f32>, block_w: usize, block_h: usize, max_diff: usize) -> Array2<f32> {
    let (w, h) = mat_wh(&left_mat);
    let mut diff_vec = vec![max_diff as f32; w * h];
    for i in 0..h {
        if i % block_h != 0 { continue } // For step_by
        for j in 0..w {
            if j % block_w != 0 { continue } // For step_by
            let mut min_diff_point = std::f32::MAX;
            let mut min_diff_index = max_diff;
            for k in 0..max_diff {
                let diff_point = get_diff_point(&left_mat, &right_mat, block_w, block_h, j+k, i, j, i);
                if diff_point < min_diff_point {
                    min_diff_point = diff_point;
                    min_diff_index = k;
                }
            }
            fill_vec(&mut diff_vec, min_diff_index as f32, j, i, block_w, block_h, w, h);
        }
    }
    Array::from_vec(diff_vec).into_shape((h, w)).unwrap()
}

fn hsv_to_rgb(h: u8, s: u8, v: u8) -> Vec<u8> {
    let hf = (h as f32 * 360. / std::u8::MAX as f32) / 60.;
    let sf = s as f32 / std::u8::MAX as f32;
    let vf = v as f32;
    let h_floor = hf.floor();
    let ff = hf - h_floor;
    let p = (vf * (1. - sf)) as u8;
    let q = (vf * (1. - sf * ff)) as u8;
    let t = (vf * (1. - sf * (1. - ff))) as u8;

    match h_floor as u8 {
        0 => vec![v, t, p],
        1 => vec![q, v, p],
        2 => vec![p, v, t],
        3 => vec![p, q, v],
        4 => vec![t, p, v],
        5 => vec![v, p, q],
        6 => vec![v, t, p],
        _ => vec![0, 0, 0],
    }
}

fn main() {
    // let left_image_file_name = "data/aloeL.jpg";
    // let right_image_file_name = "data/aloeR.jpg";
    let left_image_file_name = "data/left.png";
    let right_image_file_name = "data/right.png";
    let left_mat = get_gray_mat(&left_image_file_name);
    let right_mat = get_gray_mat(&right_image_file_name);
    let (w, h) = mat_wh(&left_mat);
    let block_w = 11;
    let block_h = 11;
    let max_diff = w/4;
    let result_mat = block_match(&left_mat, &right_mat, block_w, block_h, max_diff);
    let result_mat = (max_diff - result_mat) / max_diff * 200;
    let mut pixels = vec![];
    for p in result_mat.into_shape(w * h).unwrap().to_vec() {
        pixels.extend(hsv_to_rgb(p as u8, 255, 255));
    }
    let result_image = RgbImage::from_raw(w as u32, h as u32, pixels).unwrap();
    let _saved = result_image.save("result.png");
}
