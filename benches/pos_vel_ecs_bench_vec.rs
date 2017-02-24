#![feature(test)]

extern crate test;
use test::Bencher;
#[derive(Debug)]
struct Position {
    x: f32,
    y: f32,
}

static N_POS_VEL: usize = 1000;
static N_POS: usize = 9000;
#[derive(Debug, Copy, Clone)]
struct Velocity {
    dx: f32,
    dy: f32,
}
pub struct PosVel {
    p: Vec<Position>,
    v: Vec<Velocity>,
}
pub struct Pos {
    p: Vec<Position>,
}
pub struct Ecs {
    e1: PosVel,
    e2: Pos,
}

pub fn build() -> Ecs {
    let mut ecs = Ecs {
        e1: PosVel {
            p: Vec::with_capacity(N_POS_VEL),
            v: Vec::with_capacity(N_POS_VEL),
        },
        e2: Pos { p: Vec::with_capacity(N_POS) },
    };
    for _ in (0..N_POS_VEL) {
        let pos = Position { x: 0.0, y: 0.0 };
        let vel = Velocity { dx: 0.0, dy: 0.0 };
        ecs.e1.p.push(pos);
        ecs.e1.v.push(vel);
    }
    for _ in (0..N_POS) {
        let pos = Position { x: 0.0, y: 0.0 };
        ecs.e2.p.push(pos);
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
        {
            let p1 = ecs.e1.p.iter_mut();
            let v1 = ecs.e1.v.iter_mut();
            for (p, v) in p1.zip(v1) {
                p.x += v.dx;
                p.y += v.dy;
            }
        }
        {
            let p1 = ecs.e1.p.iter_mut();
            let p2 = ecs.e2.p.iter_mut();
            for p in p1 {}
            for p in p2 {}
            // Chaining here is 10 times slower. :x
            //for p in p1.chain(p2) {}
        }
    });
}
