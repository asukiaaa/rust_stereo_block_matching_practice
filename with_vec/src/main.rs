extern crate image;
extern crate time;
use image::{GenericImage, RgbImage};
use time::PreciseTime;

fn get_gray_pixels(file_name: &str) -> (Vec<u8>, usize, usize) {
    let img = image::open(file_name).unwrap().grayscale();
    (img.raw_pixels(), img.width() as usize, img.height() as usize)
}

fn get_min_diff_index(left_pixels: &Vec<u8>, right_pixels: &Vec<u8>, w: usize, block_w: usize, block_h: usize, block_x: usize, block_y: usize, diff_len: usize) -> f32 {
    let mut min_diff_point = std::f32::MAX;
    let mut min_diff_index = diff_len;
    for diff_index in 0..diff_len {
        let mut diff_point:f32 = 0.0;
        for y in (block_h * block_y)..(block_h * (block_y + 1)) {
            for x in (block_w * block_x)..(block_w * (block_x + 1)) {
                diff_point += (left_pixels[y * w + x + diff_index] as f32 - right_pixels[y * w + x] as f32).abs();
            }
            // Using iterator is slower than direct reading of array
            // let pixel_index = y * w + block_w * block_x;
            // let mut left_iter = left_pixels.iter().skip(pixel_index + diff_index);
            // let mut right_iter = right_pixels.iter().skip(pixel_index);
            // for lr in left_iter.zip(right_iter).take(block_w) {
            //     let (&l, &r) = lr;
            //     diff_point += (l as f32 - r as f32).abs();
            // }
            // for pixel_index in (y * w + (block_w * block_x))..(y * w + (block_w * (block_x + 1))) {
            //     diff_point += (left_pixels[pixel_index + diff_index] as f32 - right_pixels[pixel_index] as f32).abs();
            // }
        }
        if diff_point < min_diff_point {
            min_diff_point = diff_point;
            min_diff_index = diff_index;
        }
    }
    min_diff_index as f32
}

fn block_match(left_pixels: &Vec<u8>, right_pixels: &Vec<u8>, w: usize, h: usize, block_w: usize, block_h: usize, diff_len: usize) -> Vec<f32> {
    let result_w = w / block_w;
    let result_h = h / block_h;
    let mut diff_vec = vec![];
    for y in 0..result_h {
        for x in 0..result_w {
            diff_vec.push(get_min_diff_index(&left_pixels, &right_pixels, w, block_w, block_h, x, y, diff_len));
        }
    }
    diff_vec
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
    let start_time = PreciseTime::now();
    // let left_image_file_name = "../data/aloeL.jpg";
    // let right_image_file_name = "../data/aloeR.jpg";
    let left_image_file_name = "../data/left.png";
    let right_image_file_name = "../data/right.png";
    let (left_pixels, width, height) = get_gray_pixels(&left_image_file_name);
    let (right_pixels, _, _) = get_gray_pixels(&right_image_file_name);
    let block_w = 11;
    let block_h = 11;
    let diff_len = width / 4;
    let loaded_image_time = PreciseTime::now();
    let result_pixels = block_match(&left_pixels, &right_pixels, width, height, block_w, block_h, diff_len);
    let got_result_time = PreciseTime::now();
    let mut pixels = vec![];
    let diff_len_f32 = diff_len as f32;
    for p in result_pixels {
        let h = ((diff_len_f32 - p) / diff_len_f32) * 200.0;
        pixels.extend(hsv_to_rgb(h as u8, 255, 255));
    }
    let result_image = RgbImage::from_raw((width / block_w) as u32, (height / block_h) as u32, pixels).unwrap();
    let _saved = result_image.save("result.png");
    let created_result_image_time = PreciseTime::now();

    println!("Load image {} sec", start_time.to(loaded_image_time));
    println!("Get result {} sec", loaded_image_time.to(got_result_time));
    println!("Create resutl image {} sec", got_result_time.to(created_result_image_time));
    println!("Total {} sec", start_time.to(created_result_image_time));
}
