use math::Vec2;
use std::f32;

const T_THRESH: f32 = 0.0001;

#[derive(Clone, Copy, Debug, Default)]
pub struct LineSegment {
    pub from: Vec2,
    pub to: Vec2,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Circle {
    pub p: Vec2,
    pub r: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Rectangle {
    pub p: Vec2,
    pub w: f32,
    pub h: f32,
}

#[derive(Clone, Copy, Debug)]
pub enum RectangleSide {
    North,
    East,
    South,
    West,
}

impl LineSegment {
    pub fn new(from: Vec2, to: Vec2) -> Self {
        LineSegment { from, to }
    }
}

impl Circle {
    pub fn new(p: Vec2, r: f32) -> Self {
        Circle { p, r }
    }
}

impl Rectangle {
    pub fn new(p: Vec2, w: f32, h: f32) -> Self {
        Rectangle { p, w, h }
    }
}

pub fn check_rect_contains(rect: Rectangle, p: Vec2) -> bool {
    let d = p - rect.p;
    T_THRESH < d.x && d.x < rect.w && T_THRESH <= d.y && d.y < rect.h
}

pub fn check_circle_contains(circle: Circle, p: Vec2) -> bool {
    let d = p - circle.p;
    d.norm() <= circle.r
}

pub fn solve_line_line(l1: LineSegment, l2: LineSegment) -> Option<(f32, f32)> {
    let a = l2.from - l1.from;
    let r = l1.to - l1.from;
    let s = l2.to - l2.from;
    let d = r.cross(s);
    if d == T_THRESH {
        return None;
    }
    let t = a.cross(s) / d;
    let u = a.cross(r) / d;
    if T_THRESH < t && t < 1. - T_THRESH && T_THRESH < u && u < 1. - T_THRESH {
        return Some((t, u));
    }
    None
}

pub fn solve_line_circle(line: LineSegment, circle: Circle) -> Option<f32> {
    let v = line.to - line.from;
    let pc = line.from - circle.p;
    let a = v.dot(v);
    let b = 2. * v.dot(pc);
    let c = pc.dot(pc) - circle.r * circle.r;
    let d = b * b - 4. * a * c;
    if a == T_THRESH || d < T_THRESH {
        return None;
    }
    let sol_1 = (-b - d.sqrt()) / (2. * a);
    if T_THRESH < sol_1 && sol_1 < 1. - T_THRESH {
        return Some(sol_1);
    }
    let sol_2 = (-b + d.sqrt()) / (2. * a);
    if T_THRESH < sol_2 && sol_2 < 1. - T_THRESH {
        return Some(sol_2);
    }
    None
}

pub fn solve_line_rect(line: LineSegment, rect: Rectangle) -> Option<(f32, RectangleSide, f32)> {
    let mut min_t = f32::MAX;
    let mut result = None;
    for &side in [RectangleSide::North, RectangleSide::South, RectangleSide::East, RectangleSide::West].iter() {
        let rect_line = match side {
            RectangleSide::South => LineSegment::new(rect.p, rect.p + Vec2::new(rect.w, 0.)),
            RectangleSide::North => LineSegment::new(rect.p + Vec2::new(0., rect.h), rect.p + Vec2::new(rect.w, rect.h)),
            RectangleSide::West => LineSegment::new(rect.p, rect.p + Vec2::new(0., rect.h)),
            RectangleSide::East => LineSegment::new(rect.p + Vec2::new(rect.w, 0.), rect.p + Vec2::new(rect.w, rect.h)),
        };
        if let Some((tl, tr)) = solve_line_line(line, rect_line) {
            if min_t > tl {
                min_t = tl;
                result = Some((tl, side, tr))
            }
        }
    }
    result
}

pub fn solve_circle_rect_delta(circle: Circle, rect: Rectangle, delta: Vec2) -> Option<(f32, RectangleSide)> {
    let seg = LineSegment::new(circle.p, circle.p + delta);
    if let Some((tl, side, tr)) = solve_line_rect(seg, Rectangle::new(
        rect.p - Vec2::new(circle.r, circle.r),
        rect.w + 2. * circle.r, rect.h + 2. * circle.r
    )) {
        // Simplified to not do narrow phase
        Some((tl, side))
    } else {
        None
    }
}
