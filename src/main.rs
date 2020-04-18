use rand::Rng;

use std::thread;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

#[derive(Clone)]
struct Shape {
    x: i32,
    y: i32,
    k: ShapeKind,
    r: i32,
}

impl Shape {
    fn new(k: &ShapeKind) -> Shape {
        Shape { x: 0, y: 0, k: k.clone(), r: 0 }
    }

    fn pos(&self, x: i32, y: i32) -> Shape {
        let mut s = self.clone();
        s.x = x;
        s.y = y;
        s
    }

    fn w(&self) -> i32 {
        self.layout()[0].len() as i32
    }

    fn h(&self) -> i32 {
        self.layout().len() as i32
    }

    fn x_mod(&self, xvel: i32) -> Shape {
        let mut s = self.clone();
        s.x = self.x + xvel;
        s
    }

    fn y_mod(&self, yvel: i32) -> Shape {
        let mut s = self.clone();
        s.y = self.y + yvel;
        s
    }

    fn r_mod(&self, rvel: i32) -> Shape {
        let mut s = self.clone();
        s.r = self.r + rvel;
        s
    }

    fn layout(&self) -> &[&[u8]] {
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
    w: i32,
    h: i32,
    n: ShapeKind,
    v: Vec<u8>,
}

impl Well {
    fn new(mut w: i32, mut h: i32) -> Well {
        Well { w, h, n: ShapeKind::random(), v: vec![0; w as usize * h as usize] }
    }

    fn gen_shape(&mut self) -> Shape {
        let s = Shape::new(&self.n);
        self.n = ShapeKind::random();
        s.pos(self.w as i32 / 2 - s.w() as i32 / 2, 0)
    }

    fn consume(&mut self, s: Shape) -> Shape {
        let lo = s.layout();
        let mut y = s.h() - 1;
        while y >= 0 {
            let mut x = s.w() - 1;
            while x >= 0 {
                let s_c = lo[y as usize][x as usize];
                let w_c = &mut self.v[(self.w * (s.y + y) + s.x + x) as usize];
                if s_c != 0 {
                    *w_c = s_c;
                }
                x -= 1;
            }
            y -= 1;
        }
        while self.eliminate() {}
        self.gen_shape()
    }

    fn eliminate(&mut self) -> bool {
        let mut y = self.h - 1;
        while y >= 0 {
            let mut c = true;
            let mut x = self.w - 1;
            while x >= 0 {
                if self.v[(self.w * y + x) as usize] == 0 {
                    c = false;
                    break;
                }
                x -= 1;
            }
            if c {
                let mut tmp = vec![0; self.w as usize];
                tmp.extend_from_slice(&self.v[..(self.w * y) as usize]);
                tmp.extend_from_slice(&self.v[(self.w * y + self.w) as usize..]);
                self.v = tmp;
                return true;
            }
            y -= 1;
        }
        false
    }

    fn collides(&self, s: &Shape) -> bool {
        let lo = s.layout();
        let mut y = s.h() - 1;
        while y >= 0 {
            let mut x = s.w() - 1;
            while x >= 0 {
                let s_c = lo[y as usize][x as usize];
                let w_c = self.v[(self.w * (s.y + y) + s.x + x) as usize];
                if s_c != 0 && w_c != 0 {
                    return true;
                }
                x -= 1;
            }
            y -= 1;
        }
        false
    }

    fn clear(&mut self) {
        self.v = vec![0; self.w as usize * self.h as usize]
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

fn main() {
    let ctx = sdl2::init().unwrap();
    let vid = ctx.video().unwrap();

    let fw: i32 = 10;
    let fh: i32 = 22;
    let fx: i32 = 10;
    let fy: i32 = 10;
    let ff: i32 = 5;

    let mut well = Well::new(fw, fh);
    let mut shape = well.gen_shape();

    let mut rvel = 0;
    let mut xvel = 0;
    // let yvel = 1;
    let yvel = 1;

    let cell: i32 = 32;
    let wnd = vid.window("blockdrop", (fx + ff + (fw * cell) as i32 + ff + fx) as u32, (fy + ff + (fh * cell) as i32 + ff + fy) as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut cnv = wnd.into_canvas().build().unwrap();
    cnv.set_draw_color(Color::RGB(0, 255, 255));
    cnv.clear();
    cnv.present();


    let mut rekts = Vec::with_capacity((well.w * well.h) as usize);

    let mut evs = ctx.event_pump().unwrap();
    'running: loop {
        for ev in evs.poll_iter() {
            match ev {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => { break 'running; }

                Event::KeyDown { keycode: Some(Keycode::Q), .. } => { rvel = -1; }
                Event::KeyDown { keycode: Some(Keycode::E), .. } => { rvel = 1; }

                Event::KeyDown { keycode: Some(Keycode::A), .. } => { xvel = -1; }
                Event::KeyDown { keycode: Some(Keycode::D), .. } => { xvel = 1; }

                Event::KeyUp { keycode: Some(Keycode::A), .. } => if xvel < 0 { xvel = 0; }
                Event::KeyUp { keycode: Some(Keycode::D), .. } => if xvel > 0 { xvel = 0; }

                _ => {}
            }
        }

        let mut r_shape = shape.r_mod(rvel);
        rvel = 0;
        if r_shape.x + r_shape.w() > well.w {
            r_shape.x = well.w - r_shape.w();
        }
        if r_shape.x >= 0 {
            if r_shape.y >= 0 && r_shape.y + r_shape.h() <= well.h {
                if !well.collides(&r_shape) {
                    shape = r_shape;
                }
            }
        }

        let x_shape = shape.x_mod(xvel);
        if x_shape.x >= 0 && x_shape.x + x_shape.w() <= well.w {
            if !well.collides(&x_shape) {
                shape = x_shape;
            }
        }

        let y_shape = shape.y_mod(yvel);
        if y_shape.y >= 0 {
            if shape.y + shape.h() < well.h && !well.collides(&y_shape) {
                shape = y_shape;
            } else {
                shape = well.consume(shape);
                if well.collides(&shape) {
                    well.clear()
                }
            }
        }

        rekts.clear();
        cnv.set_draw_color(Color::RGB(0xFF, 0xFF, 0xFF));
        cnv.clear();

        let fm = Rect::new(fx, fy, (cell * well.w + ff + ff) as u32, (cell * well.h + ff + ff) as u32);
        cnv.set_draw_color(Color::RGB(0x00, 0x00, 0x00));
        cnv.draw_rect(fm);

        let mut i = 0;
        for c in &well.v {
            if *c != 0 {
                let x: i32 = fx + ff + cell * (i % well.w);
                let y: i32 = fy + ff + cell * (i / well.w);
                let rekt = Rect::new(x, y, cell as u32, cell as u32);
                rekts.push(rekt);
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
                    let x: i32 = fx + ff + cell * (shape.x + j);
                    let y: i32 = fy + ff + cell * (shape.y + i);
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
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

#[derive(Clone)]
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

    fn color(c: u8) -> Color {
        match c {
            _ => Color::RGB(0x00, 0x00, 0x77)
        }
    }

    fn layout(&self, rot: ShapeRotation) -> &[&[u8]] {
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
