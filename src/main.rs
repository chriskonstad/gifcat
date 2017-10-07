extern crate gif;
extern crate ncurses;
extern crate resize;

use std::fs::File;
use std::{str, thread, time};
use gif::{SetParameter};

use ncurses::*;

// TODO: Turn into crate(s) and release
// TODO: Looping playback
// TODO: Contrast adjustment?
// TODO: Map chunks into one string
// TODO: Remove extraneous copying
// TODO: Multithread?

fn rgba_to_gray(r: u8, g: u8, b: u8, _: u8) -> u8 {
    let max = 255.0;
    let r = r as f32 / max;
    let g = g as f32 / max;
    let b = b as f32 / max;

    let y = 0.299 * r + 0.587 * g + 0.114 * b;

    (y * max).round() as u8
}

//fn frame_to_grayscale(frame: &Frame) -> Vec<u8> {
fn frame_to_grayscale(buffer: &Vec<u8>, width: usize, height: usize) -> Vec<u8> {
    let mut grayscale: Vec<u8> = Vec::with_capacity(width * height);
    for (i, _) in buffer.iter().enumerate() {
        let index = i / 4;
        if 0 == i % 4 {
            grayscale.push(rgba_to_gray(buffer[i + 0],
                                        buffer[i + 1],
                                        buffer[i + 2],
                                        buffer[i + 3]));
        }
    }
    grayscale
}

fn intensity_to_char(p: u8) -> u8 {
    // stolen from https://github.com/rfotino/imgcat
    let map = "$@B%8&WM#*oahkbdpqwmZO0QLCJUYXzcvunxrjft/\\|()1{}[]?-_+~<>i!lI;:,\"^`'. ";
    let offset = (p as f32 / 255.0 * (map.len() - 1) as f32).round() as usize;
    map.chars().nth(offset).unwrap() as u8
}

fn calc_new_size(vh: usize, vw: usize, gh: usize, gw: usize) -> (usize, usize) {
    let scale = (vw as f64 / gw as f64).min(vh as f64 / gh as f64);
    let width = (gw as f64 * scale).round() as usize;
    let height = (gh as f64 * scale).round() as usize;
    (width, height)
}

fn main() {
    let mut decoder = gif::Decoder::new(File::open("/Users/chris/Desktop/giphy.gif").unwrap());
    decoder.set(gif::ColorOutput::RGBA);

    let mut decoder = decoder.read_info().unwrap();

    initscr();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    let mut screen_max_width = 0;
    let mut screen_max_height = 0;
    getmaxyx(stdscr(), &mut screen_max_height, &mut screen_max_width);

    while let Some(frame) = decoder.read_next_frame().unwrap() {
        let (width, height) = calc_new_size(screen_max_height as usize,
                                            screen_max_width as usize,
                                            frame.height as usize,
                                            frame.width as usize);
        let mut scaled = vec![0u8;width*height*4];
        let mut resizer = resize::new(frame.width as usize,
                                      frame.height as usize,
                                      width,
                                      height,
                                      resize::Pixel::RGBA,
                                      resize::Type::Lanczos3);
        resizer.resize(&frame.buffer, &mut scaled);
        let grayscale = frame_to_grayscale(&scaled, width, height);
        let ascii: Vec<u8> = grayscale.into_iter().map(|x| intensity_to_char(x)).collect();
        let chunks = ascii.chunks(width);

        clear();
        for c in chunks.into_iter() {
            printw(format!("{}\n", str::from_utf8(c).unwrap()).as_str());
        }
        refresh();

        let delay = time::Duration::from_millis(frame.delay as u64 * 10);
        // TODO Should we delay?
        //thread::sleep(delay);
    }
    println!("Hello, world!");
    endwin();
}
