extern crate gif;
extern crate ncurses;
extern crate resize;
extern crate time;

use std::fs::File;
use std::{str, thread};
use gif::{Frame, SetParameter};
use time::PreciseTime;

use ncurses::*;

// TODO: Multithread?
// TODO: Remove extraneous copying
// TODO: Turn into crate(s) and release
// TODO: Looping playback
// TODO: Contrast adjustment?
// TODO: Map chunks into one string

mod ascii_generator {
    extern crate resize;
    extern crate time;
    use gif::{Frame, SetParameter};
    use std::borrow::Cow;
    use time::PreciseTime;

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
            let index = i / 4;
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
        //println!("Scale: {}, w: {}, h: {}", scale, width, height);
        (width, height)
    }

    pub fn to_ascii<'a>(frame: &'a Frame, screen_max_width: usize, screen_max_height: usize) -> Frame<'a> {
        let (width, height) = calc_new_size(screen_max_height,
                                            screen_max_width,
                                            frame.height as usize,
                                            frame.width as usize);
        let mut scaled = vec![0u8;width*height*4];
        let startResize = PreciseTime::now();
        let mut resizer = resize::new(frame.width as usize,
                                      frame.height as usize,
                                      width,
                                      height,
                                      resize::Pixel::RGBA,
                                      //resize::Type::Lanczos3);
                                      resize::Type::Triangle);
        resizer.resize(&frame.buffer, &mut scaled);
        let endResize = PreciseTime::now();
        let startGray = PreciseTime::now();
        let grayscale = frame_to_grayscale(&scaled, width, height);
        let endGray = PreciseTime::now();
        let startAscii = PreciseTime::now();
        let ascii: Vec<u8> = grayscale.into_iter().map(|x| intensity_to_char(x)).collect();
        let endAscii = PreciseTime::now();
        let startClone = PreciseTime::now();
        let mut ret = frame.clone();
        ret.buffer = Cow::Owned(ascii);
        let endClone = PreciseTime::now();
        ret.width = width as u16;
        ret.height = height as u16;
        //println!("Resize: {}, Gray: {}, Ascii: {}, Clone: {}",
                 //startResize.to(endResize),
                 //startGray.to(endGray),
                 //startAscii.to(endAscii),
                 //startClone.to(endClone),
                 //);
        ret
    }
}



fn main() {
    let mut decoder = gif::Decoder::new(File::open("/Users/chris/Desktop/homer.gif").unwrap());
    decoder.set(gif::ColorOutput::RGBA);

    let mut decoder = decoder.read_info().unwrap();

    initscr();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    let mut screen_max_width = 0;
    let mut screen_max_height = 0;
    getmaxyx(stdscr(), &mut screen_max_height, &mut screen_max_width);

    while let Some(frame) = decoder.read_next_frame().unwrap() {
        let startFrame = PreciseTime::now();

        let ascii_frame = ascii_generator::to_ascii(&frame,
                                                    screen_max_width as usize,
                                                    screen_max_height as usize);
        let chunks = ascii_frame.buffer.chunks(ascii_frame.width as usize);

        let startRender = PreciseTime::now();
        clear();
        for c in chunks.into_iter() {
            assert!(c.len() == ascii_frame.width as usize);
            printw(format!("{}\n", str::from_utf8(c).unwrap()).as_str());
        }
        refresh();

        let endFrame = PreciseTime::now();
        //println!("Frame Total: {}s, Render: {}s", startFrame.to(endFrame), startRender.to(endFrame));
        let delay = std::time::Duration::from_millis(frame.delay as u64 * 10);
        // TODO Should we delay?
        thread::sleep(delay);
    }
    endwin();
}
