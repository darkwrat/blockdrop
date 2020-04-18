use rand::Rng;

use std::thread;
use std::time::Duration;

use bitvec::prelude::*;

use tokio::prelude::*;
use tokio::time::{self};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use bitvec::slice::ChunksExact;

struct Shape {
    x: usize,
    y: usize,
    k: ShapeKind,
    r: i32,
}

impl Shape {
    fn new(x: usize, y: usize, k: ShapeKind) -> Shape {
        Shape { x, y, k, r: 0 }
    }

    fn w(&self) -> usize {
        let mut wm = 0;
        let r = ShapeRotation::from_i32(self.r);
        for row in self.k.layout(r).iter() {
            if row.len() > wm {
                wm = row.len();
            }
        }
        wm
    }

    fn h(&self) -> usize {
        let r = ShapeRotation::from_i32(self.r);
        self.k.layout(r).len()
    }

    fn x_incr(&mut self, well_w: usize) -> usize {
        if self.x + self.w() < well_w {
            self.x += 1
        }
        self.x
    }

    fn x_decr(&mut self) -> usize {
        if self.x > 0 {
            self.x -= 1
        }
        self.x
    }

    fn x_set(&mut self, x: usize) -> usize {
        self.x = x;
        self.x
    }

    fn y_incr(&mut self, well_h: usize) -> usize {
        if self.y + self.h() < well_h {
            self.y += 1
        }
        self.y
    }

    fn y_decr(&mut self) -> usize {
        if self.y > 0 {
            self.y -= 1
        }
        self.y
    }

    fn r_incr(&mut self, well_w: usize, well_h: usize) {
        self.r = (self.r + 1) % 4;
        if self.x + self.w() > well_w {
            self.x = well_w - self.w()
        }
        if self.y + self.h() > well_h {
            self.y = well_h - self.h()
        }
    }

    fn r_decr(&mut self, well_w: usize, well_h: usize) {
        self.r = (self.r - 1) % 4;
        if self.x + self.w() > well_w {
            self.x = well_w - self.w()
        }
        if self.y + self.h() > well_h {
            self.y = well_h - self.h()
        }
    }

    fn layout(&self) -> &[&[i8]] {
        self.k.layout(ShapeRotation::from_i32(self.r))
    }
}

enum ShapeRotation { TWELVE, THREE, SIX, NINE }

impl ShapeRotation {
    fn from_i32(r: i32) -> ShapeRotation {
        match r.abs() % 4 {
            0 => ShapeRotation::TWELVE,
            1 => ShapeRotation::THREE,
            2 => ShapeRotation::SIX,
            _ => ShapeRotation::NINE,
        }
    }
}

struct Well {
    w: usize,
    h: usize,
    v: BitVec<Msb0, u64>,
}

impl Well {
    fn new(w: usize, h: usize) -> Well {
        Well { w, h, v: bitvec![Msb0, u64; 0; w*h] }
    }

    fn gen_shape(&self) -> Shape {
        let mut shape = Shape::new(0, 0, ShapeKind::random());
        let _ = shape.x_set(self.w / 2 - shape.w() / 2);
        shape
    }

    fn consume(&mut self, s: &Shape) {
        let mut v = bitvec![Msb0, u64; 0; self.w*s.y];
        let rows = s.layout();
        for row in rows.iter() {
            v.resize(v.len() + s.x, false);
            v.extend(row.iter().map(|x| (*x != 0)));
            v.resize(v.len() + self.w - s.x - row.len(), false);
        }
        v.resize(self.v.len(), false);
        self.v |= v;
    }

    fn rows(&self) -> ChunksExact<Msb0, u64> {
        self.v.chunks_exact(self.w)
    }

    fn clear(&mut self) {
        self.v.clear();
        self.v.resize(self.w * self.h, false);
    }
}

struct ColoredRect {
    r: Rect,
    c: Color,
}

impl ColoredRect {
    // fn new(c: u8, x: i32, y: i32, w: u32, h: u32) -> ColoredRect {
    // }
}

#[tokio::main]
async fn main() {
    let ctx = sdl2::init().unwrap();
    let vid = ctx.video().unwrap();

    let fw: usize = 10;
    let fh: usize = 22;

    let mut well = Well::new(fw, fh);
    let mut shape = well.gen_shape();

    let cell = 32;
    let ww = cell * well.w;
    let wh = cell * well.h;
    let wnd = vid.window("blockdrop", ww as u32, wh as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut cnv = wnd.into_canvas().build().unwrap();
    cnv.set_draw_color(Color::RGB(0, 255, 255));
    cnv.clear();
    cnv.present();

    let mut rekts = Vec::with_capacity(well.w * well.h);

    let mut iv = time::interval(Duration::new(0, 1_000_000_000u32 / 15));
    let mut evs = ctx.event_pump().unwrap();
    'running: loop {
        iv.tick().await;

        for ev in evs.poll_iter() {
            match ev {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => { break 'running; }

                Event::KeyDown { keycode: Some(Keycode::A), .. } => { let _ = shape.x_decr(); }
                Event::KeyDown { keycode: Some(Keycode::D), .. } => { let _ = shape.x_incr(well.w); }
                Event::KeyDown { keycode: Some(Keycode::Q), .. } => { shape.r_decr(well.w, well.h); }
                Event::KeyDown { keycode: Some(Keycode::E), .. } => { shape.r_incr(well.w, well.h); }

                _ => {}
            }
        }

        if shape.y == shape.y_incr(well.h) {
            well.consume(&shape);
            shape = well.gen_shape();
        }

        rekts.clear();
        cnv.set_draw_color(Color::RGB(0xFF, 0xFF, 0xFF));
        cnv.clear();

        let mut i = 0;
        for row in well.rows() {
            let mut j = 0;
            for col in row {
                if *col {
                    let x: i32 = (cell * j) as i32;
                    let y: i32 = (cell * i) as i32;
                    let rekt = Rect::new(x, y, cell as u32, cell as u32);
                    rekts.push(rekt);
                }
                j += 1;
            }
            i += 1;
        }

        cnv.set_draw_color(Color::RGB(0x77, 0x00, 0x00));
        cnv.fill_rects(&rekts);
        cnv.set_draw_color(Color::RGB(0xFF, 0xFF, 0xFF));
        cnv.draw_rects(&rekts);
        rekts.clear();

        let mut i = 0;
        for row in shape.layout().iter() {
            let mut j = 0;
            for col in row.iter() {
                let c = *col;
                if c != 0 {
                    let x: i32 = (cell * (shape.x + j)) as i32;
                    let y: i32 = (cell * (shape.y + i)) as i32;
                    let rekt = Rect::new(x, y, cell as u32, cell as u32);
                    rekts.push(rekt);
                }
                j += 1;
            }
            i += 1;
        }

        cnv.set_draw_color(Color::RGB(0x00, 0x00, 0x77));
        cnv.fill_rects(&rekts);
        cnv.set_draw_color(Color::RGB(0xFF, 0xFF, 0xFF));
        cnv.draw_rects(&rekts);
        rekts.clear();

        cnv.present();
    }
}

enum ShapeKind { I, J, L, O, S, T, Z }

impl ShapeKind {
    fn random() -> ShapeKind {
        match rand::thread_rng().gen_range(0, 7) {
            0 => ShapeKind::I,
            1 => ShapeKind::J,
            2 => ShapeKind::L,
            3 => ShapeKind::O,
            4 => ShapeKind::S,
            5 => ShapeKind::T,
            _ => ShapeKind::Z,
        }
    }

    // fn color(c: u8) -> Color {
    //     match c {
    //         0 => Color::RGB()
    //     }
    // }

    fn layout(&self, rot: ShapeRotation) -> &[&[i8]] {
        match self {
            ShapeKind::I => match rot {
                ShapeRotation::TWELVE => &[
                    &[1, ],
                    &[1, ],
                    &[1, ],
                    &[1, ],
                ],
                ShapeRotation::THREE => &[
                    &[1, 1, 1, 1],
                ],
                ShapeRotation::SIX => &[
                    &[1, ],
                    &[1, ],
                    &[1, ],
                    &[1, ],
                ],
                ShapeRotation::NINE => &[
                    &[1, 1, 1, 1],
                ],
            },
            ShapeKind::J => match rot {
                ShapeRotation::TWELVE => &[
                    &[0, 2, ],
                    &[0, 2, ],
                    &[2, 2, ],
                ],
                ShapeRotation::THREE => &[
                    &[2, 0, 0, ],
                    &[2, 2, 2, ],
                ],
                ShapeRotation::SIX => &[
                    &[2, 2, ],
                    &[2, 0, ],
                    &[2, 0, ],
                ],
                ShapeRotation::NINE => &[
                    &[2, 2, 2, ],
                    &[0, 0, 2, ],
                ],
            }
            ShapeKind::L => match rot {
                ShapeRotation::TWELVE => &[
                    &[3, 0, ],
                    &[3, 0, ],
                    &[3, 3, ],
                ],
                ShapeRotation::THREE => &[
                    &[3, 3, 3, ],
                    &[3, 0, 0, ],
                ],
                ShapeRotation::SIX => &[
                    &[3, 3, ],
                    &[0, 3, ],
                    &[0, 3, ],
                ],
                ShapeRotation::NINE => &[
                    &[0, 0, 3, ],
                    &[3, 3, 3, ],
                ],
            },
            ShapeKind::O => match rot {
                ShapeRotation::TWELVE => &[
                    &[4, 4, ],
                    &[4, 4, ],
                ],
                ShapeRotation::THREE => &[
                    &[4, 4, ],
                    &[4, 4, ],
                ],
                ShapeRotation::SIX => &[
                    &[4, 4, ],
                    &[4, 4, ],
                ],
                ShapeRotation::NINE => &[
                    &[4, 4, ],
                    &[4, 4, ],
                ],
            },
            ShapeKind::S => match rot {
                ShapeRotation::TWELVE => &[
                    &[0, 5, 5, ],
                    &[5, 5, 0, ],
                ],
                ShapeRotation::THREE => &[
                    &[5, 0, ],
                    &[5, 5, ],
                    &[0, 5, ],
                ],
                ShapeRotation::SIX => &[
                    &[0, 5, 5, ],
                    &[5, 5, 0, ],
                ],
                ShapeRotation::NINE => &[
                    &[5, 0, ],
                    &[5, 5, ],
                    &[0, 5, ],
                ],
            },
            ShapeKind::T => match rot {
                ShapeRotation::TWELVE => &[
                    &[0, 6, 0, ],
                    &[6, 6, 6, ],
                ],
                ShapeRotation::THREE => &[
                    &[6, 0, ],
                    &[6, 6, ],
                    &[6, 0, ],
                ],
                ShapeRotation::SIX => &[
                    &[6, 6, 6, ],
                    &[0, 6, 0, ],
                ],
                ShapeRotation::NINE => &[
                    &[0, 6, ],
                    &[6, 6, ],
                    &[0, 6, ],
                ],
            },
            ShapeKind::Z => match rot {
                ShapeRotation::TWELVE => &[
                    &[7, 7, 0, ],
                    &[0, 7, 7, ],
                ],
                ShapeRotation::THREE => &[
                    &[0, 7, ],
                    &[7, 7, ],
                    &[7, 0, ],
                ],
                ShapeRotation::SIX => &[
                    &[7, 7, 0, ],
                    &[0, 7, 7, ],
                ],
                ShapeRotation::NINE => &[
                    &[0, 7, ],
                    &[7, 7, ],
                    &[7, 0, ],
                ],
            },
        }
    }
}
