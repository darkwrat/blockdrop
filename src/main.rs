use std::thread;
use std::time::Duration;

use bitvec::prelude::*;

use tokio::prelude::*;
use tokio::time::{self};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

#[tokio::main]
async fn main() {
    let ctx = sdl2::init().unwrap();
    let vid = ctx.video().unwrap();

    let fw: i32 = 10;
    let fh: i32 = 24;
    let mut f = bitvec![0; (fw*fh) as usize];

    let cell = 32;
    let ww: i32 = cell * fw;
    let wh: i32 = cell * fh;
    let wnd = vid.window("blockdrop", ww as u32, wh as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut cnv = wnd.into_canvas().build().unwrap();
    cnv.set_draw_color(Color::RGB(0, 255, 255));
    cnv.clear();
    cnv.present();

    let mut x = fw / 2;
    let mut y = 0;
    let mut xvel = 0;
    let yvel = 1;

    let mut bg = Color::RGB(0xFF, 0xFF, 0xFF);
    let fg = Color::RGB(0xFF, 0x00, 0x00);
    let og = Color::RGB(0xAA, 0x00, 0x00);

    let mut rekts = Vec::with_capacity((fw * fh) as usize);

    let mut iv = time::interval(Duration::new(0, 1_000_000_000u32 / 30));
    let mut evs = ctx.event_pump().unwrap();
    'running: loop {
        iv.tick().await;

        for ev in evs.poll_iter() {
            match ev {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => { break 'running; }

                Event::KeyDown { keycode: Some(Keycode::A), .. } => { xvel = -1; }
                Event::KeyDown { keycode: Some(Keycode::D), .. } => { xvel = 1; }

                Event::KeyUp { keycode: Some(Keycode::A), .. } => if xvel < 0 { xvel = 0; }
                Event::KeyUp { keycode: Some(Keycode::D), .. } => if xvel > 0 { xvel = 0; }

                _ => {}
            }
        }

        let mut new_x = x + xvel;
        if new_x < 0 { new_x = 0 };
        if new_x + 1 > fw { new_x = fw - 1 }

        let mut new_y = y + yvel;
        if new_y != y {
            if new_y == fh {
                new_y = fh - 1;
            }
            if new_y < 0 {
                new_y = 0;
            }
            if f[(fw * new_y + new_x) as usize] {
                x = fw / 2;
                y = 0;
                if new_y == 1 {
                    f = bitvec![0; (fw*fh) as usize];
                    continue 'running;
                }
            } else {
                f.set((fw * y + x) as usize, false);
                x = new_x;
                y = new_y;
                f.set((fw * y + x) as usize, true);
            }
        } else {
            f.set((fw * y + x) as usize, false);
            x = new_x;
            y = new_y;
            f.set((fw * y + x) as usize, true);
        }

        rekts.clear();

        let mut i = 0;
        for row in f.chunks_exact(fw as usize) {
            let mut j = 0;
            for col in row {
                if *col {
                    rekts.push(Rect::new(cell * j, cell * i, cell as u32, cell as u32));
                }
                j += 1;
            }
            i += 1;
        }

        cnv.set_draw_color(bg);
        cnv.clear();
        cnv.set_draw_color(fg);
        cnv.fill_rects(&rekts);
        cnv.set_draw_color(og);
        cnv.draw_rects(&rekts);

        cnv.present();
    }
}