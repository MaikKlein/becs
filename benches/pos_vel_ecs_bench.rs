#![feature(test)]

extern crate test;
use test::Bencher;
use std::any::TypeId;

extern crate becs;

use becs::*;
#[derive(Debug)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Debug, Copy, Clone)]
struct Velocity {
    dx: f32,
    dy: f32,
}

static N_POS_VEL: usize = 1000;
static N_POS: usize = 9000;
pub fn build() -> Ecs {
    let mut ecs = Ecs::new();
    {
        let mut entity_add_pv = ecs.add_entity2::<Position, Velocity>();
        for _ in (0..N_POS_VEL) {
            let pos = Position { x: 0.0, y: 0.0 };
            let vel = Velocity { dx: 0.0, dy: 0.0 };
            entity_add_pv.add_entity2(pos, vel);
        }
    }
    {
        let mut entity_add_p = ecs.add_entity::<Position>();
        for _ in (0..N_POS) {
            let pos = Position { x: 0.0, y: 0.0 };
            entity_add_p.add_entity(pos);
        }
    }
    ecs
}
#[bench]
fn bench_build(b: &mut Bencher) {
    b.iter(|| build());
}

#[bench]
fn bench_update(b: &mut Bencher) {
    let mut ecs = build();

    b.iter(|| {
        ecs.update2(|p: &mut Position, v: &mut Velocity| {
            //println!("{:?}", p);
            p.x += v.dx;
            p.y += v.dy;
        });
        ecs.update(|_: &mut Position| {});
    });
}
