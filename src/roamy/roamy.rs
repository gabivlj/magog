use std::rand;
use std::mem;

use cgmath::point::{Point, Point2};
use cgmath::vector::{Vec2};
use cgmath::aabb::{Aabb};
use color::rgb::consts::*;
use area::{Location, Area, uphill};
use area;
use areaview;
use glutil::app::App;
use fov::Fov;
use fov;
use mapgen::MapGen;

pub struct Roamy {
    area: ~Area,
    pos: Location,
    seen: ~Fov,
    remembered: ~Fov,
    rng: rand::StdRng,
}

impl Roamy {
    pub fn new() -> Roamy {
        let mut ret = Roamy {
            area: ~Area::new(),
            pos: Location(Point2::new(0i8, 0i8)),
            seen: ~Fov::new(),
            remembered: ~Fov::new(),
            rng: rand::rng(),
        };
        ret.next_level();
        ret
    }

    pub fn next_level(&mut self) {
        self.area = ~Area::new();
        self.area.gen_cave(&mut self.rng);

        self.pos = Location(Point2::new(0i8, 0i8));
        self.seen = ~Fov::new();
        self.remembered = ~Fov::new();
    }

    pub fn draw(&mut self, app: &mut App) {
        let origin = Vec2::new(320.0f32, 180.0f32);
        let mouse = app.get_mouse();
        let mut cursor_chart_pos = areaview::screen_to_chart(
            &mouse.pos.add_v(&origin.neg()).add_v(&Vec2::new(8.0f32, 0.0f32)));
        let Location(offset) = self.pos;
        cursor_chart_pos.x += offset.x;
        cursor_chart_pos.y += offset.y;

        let mut tmp_seen = ~fov::fov(self.area, &self.pos, 12);
        mem::swap(self.seen, tmp_seen);
        // Move old fov to map memory.
        self.remembered.add(tmp_seen);

        if app.screen_area().contains(&mouse.pos) {
            if mouse.left {
                self.area.dig(&Location(cursor_chart_pos));
            }

            if mouse.right {
                self.area.fill(&Location(cursor_chart_pos));
            }
        }

        areaview::draw_area(self.area, app, &self.pos, self.seen, self.remembered);

        if !self.area.fully_explored(self.remembered) {
            let map = self.area.explore_map(self.remembered);
            match uphill(&map, &self.pos) {
                Some(p) => { if self.area.is_walkable(&p) { self.pos = p; } },
                None => (),
            }
        } else {
            app.set_color(&FIREBRICK);
            app.draw_string(&Vec2::new(32f32, 32f32), "Done exploring");
        }

        if self.area.get(&self.pos) == area::Downstairs {
            self.next_level();
        }
    }
}
