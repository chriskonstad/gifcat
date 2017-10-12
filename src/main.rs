extern crate gif;
extern crate ncurses;
extern crate resize;

use std::fs::File;
use std::{str, thread};
use std::sync::mpsc;
use gif::SetParameter;

use ncurses::*;

// TODO: Multithread?
// TODO: Remove extraneous copying
// TODO: Turn into crate(s) and release
// TODO: Looping playback
// TODO: Contrast adjustment?
// TODO: Map chunks into one string

mod ascii_generator {
    extern crate resize;
    use gif::Frame;

    pub struct Renderable {
        pub delay: u64,
        pub width: usize,
        pub height: usize,
        pub buffer: Vec<u8>,
    }

    impl Renderable {
        pub fn new(delay: u64, width: usize, height: usize, buffer: Vec<u8>) -> Renderable {
            Renderable {
                delay: delay,
                width: width,
                height: height,
                buffer: buffer.to_vec(),
            }
        }

        pub fn from_frame(frame: &Frame) -> Renderable {
            Renderable::new(
                frame.delay as u64,
                frame.width as usize,
                frame.height as usize,
                frame.buffer.to_vec(),
                )
        }
    }

    fn rgba_to_gray(r: u8, g: u8, b: u8, _: u8) -> u8 {
        let max = 255.0;
        let r = r as f32 / max;
        let g = g as f32 / max;
        let b = b as f32 / max;

        let y = 0.299 * r + 0.587 * g + 0.114 * b;

        (y * max).round() as u8
    }

    fn frame_to_grayscale(buffer: &Vec<u8>, width: usize, height: usize) -> Vec<u8> {
        let mut grayscale: Vec<u8> = Vec::with_capacity(width * height);
        for (i, _) in buffer.iter().enumerate() {
            if 0 == i % 4 {
                grayscale.push(rgba_to_gray(buffer[i + 0],
                                            buffer[i + 1],
                                            buffer[i + 2],
                                            buffer[i + 3]));
            }
        }
        assert!(grayscale.len() == buffer.len() / 4);
        grayscale
    }

    fn intensity_to_char(p: u8) -> u8 {
        // stolen from https://github.com/rfotino/imgcat
        let map = b"@B%8&WM#*oahkbdpqwmZO0QLCJUYXzcvunxrjft/|()1{}[]?-_+~<>i!lI;:,\"`'. ";
        let offset = (p as f32 / 255.0 * (map.len() - 1) as f32).round() as usize;
        map[offset]
    }

    fn calc_new_size(vh: usize, vw: usize, gh: usize, gw: usize) -> (usize, usize) {
        let scale = (vw as f64 / gw as f64).min(vh as f64 / gh as f64);
        let width = (gw as f64 * scale * 10.0/8.0).round() as usize;
        let height = (gh as f64 * scale * 10.0/10.0 ).round() as usize;
        (width, height)
    }

    pub fn to_ascii(frame: &Renderable, screen_max_width: usize, screen_max_height: usize) -> Renderable {
        let (width, height) = calc_new_size(screen_max_height,
                                            screen_max_width,
                                            frame.height as usize,
                                            frame.width as usize);
        let mut scaled = vec![0u8;width*height*4];
        let mut resizer = resize::new(frame.width as usize,
                                      frame.height as usize,
                                      width,
                                      height,
                                      resize::Pixel::RGBA,
                                      resize::Type::Triangle);
        resizer.resize(&frame.buffer, &mut scaled);
        let grayscale = frame_to_grayscale(&scaled, width, height);
        Renderable::new(
            frame.delay,
            width,
            height,
            grayscale.into_iter().map(|x| intensity_to_char(x)).collect(),
            )
    }
}

use ascii_generator::{Renderable, to_ascii};


fn main() {
    let file = "/Users/chris/Desktop/cam.gif";

    initscr();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    let mut screen_max_width = 0;
    let mut screen_max_height = 0;
    getmaxyx(stdscr(), &mut screen_max_height, &mut screen_max_width);

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let mut decoder = gif::Decoder::new(File::open(file).unwrap());
        decoder.set(gif::ColorOutput::RGBA);

        let mut decoder = decoder.read_info().unwrap();

        while let Some(frame) = decoder.read_next_frame().unwrap() {
            let ascii_frame = to_ascii(&Renderable::from_frame(frame),
                                       screen_max_width as usize,
                                       screen_max_height as usize);
            tx.send(ascii_frame).unwrap();
        }
    }
    );

    for frame in rx {
        clear();
        for c in frame.buffer.chunks(frame.width as usize).into_iter() {
            printw(format!("{}\n", str::from_utf8(c).unwrap()).as_str());
        }
        refresh();
        let delay = std::time::Duration::from_millis(frame.delay as u64 * 10);
        thread::sleep(delay);
    }

    endwin();
}
