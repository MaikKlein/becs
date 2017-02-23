//#![feature(test)]
//extern crate test;
use std::collections::HashMap;
use std::any::TypeId;

pub struct VecTypeStore<C: DropType> {
    store: TypeStore,
    _m: ::std::marker::PhantomData<C>,
}

impl<D: DropType> std::ops::Drop for VecTypeStore<D> {
    fn drop(&mut self) {
        for (&type_id, &ptr) in self.store.store.iter() {
            D::drop_type(type_id, ptr);
        }
    }
}

impl<D: DropType> VecTypeStore<D> {
    pub fn types(&self) -> std::collections::hash_map::Keys<std::any::TypeId, *mut ()> {
        self.store.types()
    }

    pub fn new() -> Self {
        VecTypeStore {
            store: TypeStore::new(),
            _m: ::std::marker::PhantomData,
        }
    }

    pub fn contains_type<T: 'static>(&self) -> bool {
        self.store.contains_type::<Vec<T>>()
    }

    pub fn insert<T: 'static>(&mut self, val: Vec<T>) {
        self.store.insert(val);
    }

    pub fn get<T: 'static>(&self) -> Option<&Vec<T>> {
        self.store.get::<Vec<T>>()
    }

    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut Vec<T>> {
        self.store.get_mut::<Vec<T>>()
    }

    pub fn get_mut2<A: 'static, B: 'static>(&mut self) -> Option<(&mut Vec<A>, &mut Vec<B>)> {
        self.store.get_mut2::<Vec<A>, Vec<B>>()
    }
}

pub trait DropType {
    fn drop_type(type_id: TypeId, ptr: *mut ());
}

pub struct TypeStore {
    store: HashMap<std::any::TypeId, *mut ()>,
}


impl TypeStore {
    pub fn types(&self) -> std::collections::hash_map::Keys<std::any::TypeId, *mut ()> {
        self.store.keys()
    }

    pub fn contains_type<T: 'static>(&self) -> bool {
        self.store.contains_key(&std::any::TypeId::of::<T>())
    }

    pub fn insert<T: 'static>(&mut self, val: T) {
        let ptr = Box::into_raw(Box::new(val)) as *mut ();
        let t = std::any::TypeId::of::<T>();
        self.store.insert(t, ptr);
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        let t = std::any::TypeId::of::<T>();
        self.store.get(&t).map(|&ptr| unsafe { std::mem::transmute(ptr) })
    }

    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        let t = std::any::TypeId::of::<T>();
        self.store.get(&t).map(|&ptr| unsafe { std::mem::transmute(ptr) })
    }

    pub fn get_mut2<A: 'static, B: 'static>(&mut self) -> Option<(&mut A, &mut B)> {
        let a = std::any::TypeId::of::<A>();
        let b = std::any::TypeId::of::<B>();
        let r1 = self.store.get(&a);
        let r2 = self.store.get(&b);
        if (r1.is_none() || r2.is_none()) {
            return None;
        }
        Some((unsafe { std::mem::transmute(*r1.unwrap()) },
              unsafe { std::mem::transmute(*r2.unwrap()) }))
    }

    pub fn new() -> Self {
        TypeStore { store: HashMap::new() }
    }
}

pub struct Ecs<D: DropType> {
    world: Vec<VecTypeStore<D>>,
}
pub struct AddEntity<'r, Ecs: 'r, A> {
    ecs: &'r mut Ecs,
    index: usize,
    _a: ::std::marker::PhantomData<A>,
}

impl<'r, D: DropType, A: 'static> AddEntity<'r, Ecs<D>, A> {
    pub fn add_entity(&mut self, a: A) {
        self.ecs.world[self.index].get_mut::<A>().unwrap().push(a);
    }
}

pub struct AddEntity2<'r, Ecs: 'r, A, B> {
    ecs: &'r mut Ecs,
    index: usize,
    _a: ::std::marker::PhantomData<A>,
    _b: ::std::marker::PhantomData<B>,
}
impl<'r, D: DropType, A: 'static, B: 'static> AddEntity2<'r, Ecs<D>, A, B> {
    pub fn add_entity2(&mut self, a: A, b: B) {
        self.ecs.world[self.index].get_mut::<A>().unwrap().push(a);
        self.ecs.world[self.index].get_mut::<B>().unwrap().push(b);
    }
}
impl<D: DropType> Ecs<D> {
    pub fn add_entity<'r, A: 'static>(&'r mut self) -> AddEntity<'r, Self, A> {
        let p = self.world
            .iter()
            .position(|store| {
                store.types().len() == 1 && store.types().all(|ty| *ty == TypeId::of::<A>())
            });
        let index = p.unwrap_or(self.world.len());
        if p.is_none() {
            let mut store = VecTypeStore::new();
            store.insert(Vec::<A>::new());
            self.world.push(store);
        }
        AddEntity {
            ecs: self,
            index: index,
            _a: ::std::marker::PhantomData,
        }
    }

    pub fn add_entity2<'r, A: 'static, B: 'static>(&'r mut self) -> AddEntity2<'r, Self, A, B> {
        let p = self.world
            .iter()
            .position(|store| {
                store.types().len() == 2 &&
                store.types().all(|ty| *ty == TypeId::of::<A>() || *ty == TypeId::of::<B>())
            });
        let index = p.unwrap_or(self.world.len());
        if p.is_none() {
            let mut store = VecTypeStore::new();
            store.insert(Vec::<A>::new());
            store.insert(Vec::<B>::new());
            self.world.push(store);
        }

        AddEntity2 {
            ecs: self,
            index: index,
            _a: ::std::marker::PhantomData,
            _b: ::std::marker::PhantomData,
        }
    }

    pub fn update<T: 'static, F>(&mut self, f: F)
        where F: Fn(&mut T)
    {
        for store in self.world.iter_mut() {
            if let Some(i1) = store.get_mut::<T>() {
                for val in i1.iter_mut() {
                    f(val);
                }
            }
        }
    }

    pub fn update2<A: 'static, B: 'static, F>(&mut self, f: F)
        where F: Fn(&mut A, &mut B)
    {
        for store in self.world.iter_mut() {
            if let Some((i1, i2)) = store.get_mut2::<A, B>() {
                for (a, b) in i1.iter_mut().zip(i2.iter_mut()) {
                    f(a, b);
                }
            }
        }
    }

    pub fn new() -> Ecs<D> {
        Ecs { world: Vec::new() }
    }
}

pub fn drop_vec<T>(ptr: *mut ()) {
    let tptr: *mut Vec<T> = unsafe { ::std::mem::transmute(ptr) };
    unsafe { Box::from_raw(tptr) };
}

//#[cfg(test)]
//mod tests {
//    use super::*;
//    use test::Bencher;
//
//    pub struct PosVelDrop;
//    impl DropType for PosVelDrop {
//        fn drop_type(type_id: TypeId, ptr: *mut ()) {
//            if TypeId::of::<Vec<Position>>() == type_id {
//                drop_vec::<Position>(ptr);
//            }
//            if TypeId::of::<Vec<Velocity>>() == type_id {
//                drop_vec::<Velocity>(ptr);
//            }
//        }
//    }
//
//    #[derive(Debug)]
//    struct Position {
//        x: f32,
//        y: f32,
//    }
//
//    #[derive(Debug, Copy, Clone)]
//    struct Velocity {
//        x: f32,
//        y: f32,
//    }
//    static N_POS_VEL: usize = 1000;
//    static N_POS: usize = 9000;
//    pub fn build() -> Ecs<PosVelDrop> {
//        let mut ecs = Ecs::new();
//        {
//            let mut entity_add_pv = ecs.add_entity2::<Position, Velocity>();
//            for _ in (0..N_POS_VEL) {
//                let pos = Position { x: 0.0, y: 0.0 };
//                let vel = Velocity { x: 0.0, y: 0.0 };
//                entity_add_pv.add_entity2(pos, vel);
//            }
//        }
//        {
//            let mut entity_add_p = ecs.add_entity::<Position>();
//            for _ in (0..N_POS) {
//                let pos = Position { x: 0.0, y: 0.0 };
//                entity_add_p.add_entity(pos);
//            }
//        }
//        ecs
//    }
//
//    #[bench]
//    fn bench_build(b: &mut Bencher) {
//        b.iter(|| build());
//    }
//    #[bench]
//    fn bench_update(b: &mut Bencher) {
//        let mut ecs = build();
//        b.iter(|| {
//            ecs.update2(|p: &mut Position, v: &mut Velocity| {
//                //println!("{:?}", p);
//                p.x += v.x;
//                p.y += v.y;
//            });
//        });
//    }
//}
