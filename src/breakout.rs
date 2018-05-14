use gl;
use glutin::{
    ElementState,
    MouseButton,
    VirtualKeyCode,
};
use std::path::{Path, PathBuf};
use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::cmp;

use collision::{
    check_rect_contains,
    solve_circle_rect_delta,
    Circle,
    Rectangle,
    RectangleSide,
};
use fonts::{FontLibrary, FontHandle};
use math::Vec2;
use renderer::{Color, Renderer, Viewport};


struct LevelState {
    is_launching_ball: bool,
    w: f32,
    h: f32,
    screen_p: Vec2,
    screen_v: Vec2,
    delay: f32,
    time_scale: f32,
    paddle: Paddle,
    ball_proto: Ball,
    active_balls: Vec<Ball>,
    blocks: Vec<Block>,
    invalid_block_start: usize,
}

enum Scene {
    Start,
    Level(LevelState),
    Finish,
}

fn load_level<P: AsRef<Path>>(path: P) -> Scene {
    const DEFAULT_PADDLE_RADIUS: f32 = 48.;
    const DEFAULT_PADDLE_HEIGHT: f32 = 16.;
    const BLOCK_W: f32 = 24.;
    const BLOCK_H: f32 = 16.;
    const BOTTOM_TO_PADDLE_BOTTOM: f32 = 32.;
    const BOTTOM_TO_BLOCK_BOTTOM: f32 = 256.;

    let file = File::open(Path::new("res/levels/").join(path)).expect("Cannot open file");
    let mut file = BufReader::new(file);
    let mut read_line = || {
        let mut line = String::new();
        if file.read_line(&mut line).unwrap_or(0) > 0 {
            let len = line.len();
            line.truncate(len - 1);
            Some(line)
        } else {
            None
        }
    };
    let mut blocks = Vec::new();
    let mut grid_w = 0;
    let mut lines = Vec::new();
    while let Some(line) = read_line() {
        grid_w = cmp::max(line.len(), grid_w);
        lines.push(line);
    }
    let grid_h = lines.len();
    for (j, line) in lines.iter().enumerate() {
        for (i, c) in line.chars().enumerate() {
            if c == '#' {
                blocks.push(Block {
                    r: Rectangle::new(
                        Vec2::new(
                            i as f32 * BLOCK_W,
                            ((grid_h - j - 1) as f32 * BLOCK_H) + BOTTOM_TO_BLOCK_BOTTOM,
                        ),
                        BLOCK_W,
                        BLOCK_H,
                    ),
                    hits: 0,
                });
            }
        }
    }

    let w = grid_w as f32 * BLOCK_W;
    let h = grid_h as f32 * BLOCK_H + BOTTOM_TO_BLOCK_BOTTOM;
    let blocks_len = blocks.len();

    Scene::Level(LevelState {
        is_launching_ball: true,
        w,
        h,
        screen_p: Vec2::default(),
        screen_v: Vec2::default(),
        delay: 0.,
        time_scale: 1.,
        paddle: Paddle {
            r: Rectangle::new(
               Vec2::new(
                   w / 2. - DEFAULT_PADDLE_RADIUS,
                   BOTTOM_TO_PADDLE_BOTTOM,
               ),
               DEFAULT_PADDLE_RADIUS * 2.,
               DEFAULT_PADDLE_HEIGHT,
            ),
            dx: 0.,
        },
        ball_proto: Ball {
            c: Circle::new(Vec2::default(), 8.),
            v: Vec2::new(0., 300.),
        },
        active_balls: Vec::new(),
        blocks,
        invalid_block_start: blocks_len,
    })
}

struct Paddle {
    r: Rectangle,
    dx: f32,
}

#[derive(Clone)]
struct Ball {
    c: Circle,
    v: Vec2,
}

struct Block {
    r: Rectangle,
    hits: i32,
}

struct State {
    score: i32,
    scene: Scene,
    balls_left: i32,
    current_level: usize,
    levels: Vec<PathBuf>,
    key_l_state: bool,
    key_r_state: bool,
}

struct Assets {
    default_font: FontHandle,
    primary_font: FontHandle,
    secondary_font: FontHandle,
}

pub struct Game {
    renderer: Renderer,
    font_lib: FontLibrary,
    viewport: Viewport,
    state: State,
    assets: Assets,
}

const PADDLE_ADJ_FACTOR: f32 = 0.4;
const POINTS_PER_BLOCK: i32 = 10;
const BALL_MASS: f32 = 3.;
const BOUNCE_DELAY: f32 = 0.01;
const BOUNCE_SPEED_SCALE: f32 = 1.05;
const LEVEL_MASS: f32 = 5.;
const LEVEL_DAMP: f32 = 0.3;
const LEVEL_YOUNGS_MODULUS: f32 = 3.;
const BALL_ANGLE_CLAMP: f32 = 0.5;

fn adjust_velocity(mut v: Vec2, dx: f32) -> Vec2 {
    let mag = v.norm();
    v.x += dx;
    mag * v.unit()
}

fn clamp_angle_x(v: Vec2, angle: f32) -> Vec2 {
    let mag = v.norm();
    let unit =  (1. / mag) * v;
    let cx = angle.cos();
    let cy = angle.sin();
    if unit.x.abs() >= cx {
        Vec2::new(
            cx * v.x.signum() * mag,
            cy * v.y.signum() * mag,
        )
    } else {
        v
    }
}

impl Game {
    pub fn new(screen_w: u32, screen_h: u32) -> Self {
        let mut font_lib = FontLibrary::new();
        let state = State {
            score: 0,
            scene: Scene::Start,
            balls_left: 0,
            current_level: 0,
            levels: vec![
                "0.level".into(),
                "1.level".into(),
                "2.level".into(),
            ],
            key_l_state: false,
            key_r_state: false,
        };
        let assets = Assets {
            default_font: font_lib.load_from_file("res/fonts/yoster.ttf", 20),
            primary_font: font_lib.load_from_file("res/fonts/yoster.ttf", 64),
            secondary_font: font_lib.load_from_file("res/fonts/yoster.ttf", 36),
        };
        Game {
            renderer: Renderer::new(),
            font_lib,
            viewport: Viewport {
                p: Vec2::default(),
                w: screen_w as _,
                h: screen_h as _,
            },
            state,
            assets,
        }
    }

    pub fn on_mouse_button(&mut self, button: MouseButton, state: ElementState) { }

    pub fn on_mouse_motion(&mut self, x: f32, y: f32) { }

    pub fn on_key(&mut self, keycode: VirtualKeyCode, key_state: ElementState) {
        if keycode == VirtualKeyCode::Left {
            self.state.key_l_state = key_state == ElementState::Pressed;
        }
        if keycode == VirtualKeyCode::Right {
            self.state.key_r_state = key_state == ElementState::Pressed;
        }
        match self.state.scene {
            Scene::Start => {
                if (key_state == ElementState::Pressed) {
                    self.state.score = 0;
                    self.state.balls_left = 3;
                    self.state.current_level = 0;
                    self.state.scene = load_level(&self.state.levels[0]);
                }
            },
            Scene::Level(ref mut level) => {
                if keycode == VirtualKeyCode::Space && key_state == ElementState::Pressed && level.is_launching_ball {
                    level.is_launching_ball = false;
                    let p = level.paddle.r.p + Vec2::new(
                        level.paddle.r.w / 2., 
                        level.paddle.r.h + level.ball_proto.c.r
                        );
                    let v = clamp_angle_x(
                        adjust_velocity(level.ball_proto.v, level.paddle.dx * PADDLE_ADJ_FACTOR),
                        BALL_ANGLE_CLAMP,
                    );
                    let c = Circle::new(p, level.ball_proto.c.r);
                    level.active_balls.push(Ball {c, v});
                }
            },
            Scene::Finish => {
                if (key_state == ElementState::Pressed) {
                    self.state.scene = Scene::Start;
                }
            },
        }
    }

    pub fn on_viewport_change(&mut self, w: u32, h: u32) {
        /* We probably want to ignore resize
        self.viewport.w = w as _;
        self.viewport.h = h as _;
        unsafe {
            gl::Viewport(0, 0, w as _, h as _);
        }
        */
    }

    pub fn step(&mut self, dt: f32) {
        let mut next_scene = None;
        if let Scene::Level(ref mut level) = self.state.scene {
            if level.invalid_block_start == 0 {
                self.state.current_level += 1;
                if self.state.current_level < self.state.levels.len() {
                    next_scene = Some(load_level(&self.state.levels[self.state.current_level]));
                } else {
                    next_scene = Some(Scene::Finish);
                }
            } else if self.state.balls_left == 0 {
                next_scene = Some(Scene::Finish);
            }
            let mut dt = level.time_scale * dt;
            let reduce = if dt > level.delay { level.delay } else { dt };
            dt -= reduce;
            level.delay -= reduce;

            level.screen_p = level.screen_p + dt * level.screen_v;
            level.screen_v = (1. - LEVEL_DAMP) * level.screen_v + (-LEVEL_YOUNGS_MODULUS) * level.screen_p;

            let mut ddx = 0.;
            if self.state.key_l_state {
                ddx += -96.;
            }
            if self.state.key_r_state {
                ddx += 96.;
            }
            level.paddle.dx += ddx;
            level.paddle.dx *= 0.9;
            level.paddle.r.p.x += level.paddle.dx * dt;
            if level.paddle.r.p.x < 0. {
                level.paddle.r.p.x = 0.;
                if level.paddle.dx < 0. {
                    level.paddle.dx = 0.;
                }
            }
            if level.paddle.r.p.x + level.paddle.r.w > level.w {
                level.paddle.r.p.x = level.w - level.paddle.r.w ;
                if level.paddle.dx > 0. {
                    level.paddle.dx = 0.;
                }
            }
            let boundary_rects = [
                Rectangle::new(Vec2::new(-24., 0.), 24., level.h),
                Rectangle::new(Vec2::new(level.w, 0.), 24., level.h),
                Rectangle::new(Vec2::new(-24., level.h), level.w + 48., 24.),
            ];
            'ball_loop: for ball in level.active_balls.iter_mut() {
                let reduce = if dt > level.delay { level.delay } else { dt };
                dt -= reduce;
                level.delay -= reduce;
                while dt > 0. {
                    let dcp = dt * ball.v;
                    {

                        enum What {
                            Block(usize),
                            Boundary,
                        }

                        let mut collision = None;
                        for (i, block) in level.blocks[..level.invalid_block_start].iter().enumerate() {
                            if let Some((t, side)) = solve_circle_rect_delta(ball.c, block.r, dcp) {
                                if let Some((tt, _, _)) = collision {
                                    if tt > t {
                                        collision = Some((t, side, What::Block(i)));
                                    }
                                } else {
                                    collision = Some((t, side, What::Block(i)));
                                }
                            }
                        }
                        for rect in &boundary_rects {
                            if let Some((t, side)) = solve_circle_rect_delta(ball.c, *rect, dcp) {
                                if let Some((tt, _, _)) = collision {
                                    if tt > t {
                                        collision = Some((t, side, What::Boundary));
                                    }
                                } else {
                                    collision = Some((t, side, What::Boundary));
                                }
                            }
                        }
                        if let Some((t, side, what)) = collision {
                            ball.c.p = ball.c.p + t * dcp;
                            let original_v = ball.v.clone();
                            match side {
                                RectangleSide::North => ball.v.y = ball.v.y.abs(),
                                RectangleSide::South => ball.v.y = -1. * ball.v.y.abs(),
                                RectangleSide::East => ball.v.x = ball.v.x.abs(),
                                RectangleSide::West => ball.v.x = -1. * ball.v.x.abs(),
                            }
                            level.screen_v = level.screen_v + (BALL_MASS / LEVEL_MASS) * (original_v - ball.v);
                            level.delay = BOUNCE_DELAY;
                            if let What::Block(i) = what {
                                level.blocks[i].hits += 1;
                                level.invalid_block_start -= 1;
                                level.blocks.swap(i, level.invalid_block_start);
                                self.state.score += POINTS_PER_BLOCK;
                            }
                            continue 'ball_loop;
                        }
                    }
                    if let Some((t, RectangleSide::North)) = solve_circle_rect_delta(ball.c, level.paddle.r, dcp) {
                        let original_v = ball.v.clone();
                        ball.c.p = ball.c.p + t * dcp;
                        ball.v.y = ball.v.y.abs();
                        ball.v = clamp_angle_x(
                            adjust_velocity(ball.v, level.paddle.dx * PADDLE_ADJ_FACTOR),
                            BALL_ANGLE_CLAMP,
                            );
                        dt -= t;
                        let v = original_v - ball.v;
                        level.screen_v = level.screen_v + (BALL_MASS / LEVEL_MASS) * v;
                        level.delay = BOUNCE_DELAY;
                        ball.v = BOUNCE_SPEED_SCALE * ball.v;
                        continue 'ball_loop;
                    }
                    ball.c.p = ball.c.p + dcp;
                    dt = 0.;
                }
            }
            let level_rect = Rectangle::new(Vec2::default(), level.w, level.h);
            level.active_balls = level.active_balls
                .iter()
                .filter(|ball| check_rect_contains(level_rect, ball.c.p + Vec2::new(0., -ball.c.r - 12.)))
                .cloned()
                .collect();
            if level.active_balls.len() == 0 && !level.is_launching_ball {
                self.state.balls_left -= 1;
                level.is_launching_ball = true;
            }
        }
        if let Some(scene) = next_scene {
            self.state.scene = scene;
        }
    }

    pub fn render(&mut self) {
        unsafe {
            gl::ClearColor(0.1, 0.1, 0.1, 0.);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        match self.state.scene {
            Scene::Start => {
                self.renderer.draw_text(
                    &self.viewport,
                    "B R E A K O U T ! !",
                    Vec2::new(24., 256.),
                    Color::new(1., 0., 0., 1.), 
                    self.font_lib.get(self.assets.primary_font)
                );
                self.renderer.draw_text(
                    &self.viewport,
                    "A small game written by Eugene Che ~~",
                    Vec2::new(24., 212.),
                    Color::new(1., 1., 0., 1.), 
                    self.font_lib.get(self.assets.secondary_font)
                );
                self.renderer.draw_text(
                    &self.viewport,
                    "Press any key to continue...",
                    Vec2::new(24., 64.),
                    Color::new(0., 0., 1., 1.), 
                    self.font_lib.get(self.assets.default_font)
                );
            },
            Scene::Finish => {
                self.renderer.draw_text(
                    &self.viewport,
                    &format!("Your final score is {}", self.state.score),
                    Vec2::new(24., 256.),
                    Color::new(1., 0., 0., 1.), 
                    self.font_lib.get(self.assets.primary_font)
                );
                self.renderer.draw_text(
                    &self.viewport,
                    "N I C E ! !",
                    Vec2::new(24., 212.),
                    Color::new(1., 1., 0., 1.), 
                    self.font_lib.get(self.assets.secondary_font)
                );
                self.renderer.draw_text(
                    &self.viewport,
                    "Press any key to continue...",
                    Vec2::new(24., 64.),
                    Color::new(0., 0., 1., 1.), 
                    self.font_lib.get(self.assets.default_font)
                );
            },
            Scene::Level(ref level) => {
                let vp_x = (self.viewport.w as f32 - level.w) / 2.;
                let vp_y = (self.viewport.h as f32 - level.h) / 2.;
                let vp = Vec2::new(vp_x, vp_y) + level.screen_p;
                self.renderer.begin_batch();
                self.renderer.draw_rectangle(vp, level.w, level.h, Color::new(0.01, 0.01, 0.01, 1.));
                for block in level.blocks[..level.invalid_block_start].iter() {
                    self.renderer.draw_rectangle(vp + block.r.p, block.r.w, block.r.h, Color::new(1., 1., 0., 1.));
                }
                self.renderer.draw_rectangle(
                    vp + level.paddle.r.p,
                    level.paddle.r.w,
                    level.paddle.r.h,
                    Color::new(1., 0., 0., 1.),
                );
                if level.is_launching_ball {
                    let p = level.paddle.r.p + Vec2::new(
                        level.paddle.r.w / 2., 
                        level.paddle.r.h + level.ball_proto.c.r
                    );
                    self.renderer.draw_circle(vp + p, level.ball_proto.c.r, Color::new(0., 0., 1., 1.));
                }
                for ball in level.active_balls.iter() {
                    self.renderer.draw_circle(vp + ball.c.p, ball.c.r, Color::new(0., 0., 1., 1.));
                }
                self.renderer.end_batch(&self.viewport);

                self.renderer.draw_text(
                    &self.viewport,
                    &format!("Score: {}", self.state.score),
                    Vec2::new(24., 64.),
                    Color::new(0., 1., 0., 1.), 
                    self.font_lib.get(self.assets.default_font)
                );
                self.renderer.draw_text(
                    &self.viewport,
                    &format!("Lives: {}", self.state.balls_left),
                    Vec2::new(24., 32.),
                    Color::new(0., 1., 0., 1.), 
                    self.font_lib.get(self.assets.default_font)
                );
            }
        }
    }
}
