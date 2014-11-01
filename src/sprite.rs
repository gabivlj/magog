use std::slice::Items;
use calx::color;
use calx::{Context, V2};
use world::{Location, Unchart};
use viewutil::{FX_Z, chart_to_screen};

trait WorldSprite {
    fn update(&mut self);
    fn is_alive(&self) -> bool;

    fn footprint<'a>(&'a self) -> Items<'a, Location>;
    // XXX: Locked to the type of iterator Vecs return for now. It's assumed
    // that implementers use a Vec to cache the footprint points internally.

    fn draw(&self, chart: &Location, ctx: &mut Context);
    // XXX: Can't parametrize to Unchart since trait objects can't have
    // parameterized methods.
}

pub struct WorldSprites {
    sprites: Vec<Box<WorldSprite + 'static>>,
}

impl WorldSprites {
    pub fn new() -> WorldSprites {
        WorldSprites {
            sprites: vec!(),
        }
    }

    pub fn add(&mut self, spr: Box<WorldSprite + 'static>) {
        self.sprites.push(spr);
    }

    pub fn draw(&self, is_visible: |V2<int>| -> bool, chart: &Location, ctx: &mut Context) {
        // XXX: Ineffective if there are many sprites outside the visible
        // area.
        for s in self.sprites.iter() {
            for &loc in s.footprint() {
                if chart.chart_pos(loc).map_or(false, |p| is_visible(p)) {
                    s.draw(chart, ctx);
                    break;
                }
            }
        }
    }

    pub fn update(&mut self) {
        for spr in self.sprites.iter_mut() { spr.update(); }
        self.sprites.retain(|spr| spr.is_alive());
    }
}

pub struct BeamSprite {
    p1: Location,
    p2: Location,
    life: int,
    footprint: Vec<Location>,
}

impl BeamSprite {
    pub fn new(p1: Location, p2: Location, life: int) -> BeamSprite {
        BeamSprite {
            p1: p1,
            p2: p2,
            life: life,
            // TODO: Generate intervening points into the footprint. With this
            // footprint you can't see the beam unless either the start or the
            // end point is visible.
            footprint: vec![p1, p2],
        }
    }
}

impl WorldSprite for BeamSprite {
    fn update(&mut self) { self.life -= 1; }
    fn is_alive(&self) -> bool { self.life >= 0 }
    fn footprint<'a>(&'a self) -> Items<'a, Location> {
        self.footprint.iter()
    }
    fn draw(&self, chart: &Location, ctx: &mut Context) {
        if let (Some(p1), Some(p2)) = (chart.chart_pos(self.p1), chart.chart_pos(self.p2)) {
            ctx.draw_line(
                chart_to_screen(p1), chart_to_screen(p2), FX_Z, &color::LIME);
        }
    }
}
