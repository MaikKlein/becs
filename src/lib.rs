#![feature(test)]
#![feature(specialization)]
extern crate test;
use std::collections::HashMap;
use std::any::TypeId;
use std::mem::transmute;

#[macro_export]
macro_rules! ptr_as_ref{
    ($ptr: expr) => {
        {
            let fat_ptr = std::mem::transmute::<*mut DynamicType, FatPointer>($ptr);
            std::mem::transmute(fat_ptr.obj_ptr)
        }
    }
}

//trait Type<C> {
//    fn has_type() -> bool;
//}
//
//struct Test;
//
//impl<T> Type<Test> for T {
//    default fn has_type() -> bool {
//        false
//    }
//}
//
//impl Type<Test> for i32 {
//    fn has_type() -> bool {
//        true
//    }
//}

pub struct VecTypeStore {
    store: TypeStore,
}

impl VecTypeStore {
    pub fn type_len(&self) -> usize {
        self.store.type_len()
    }

    pub fn new() -> Self {
        VecTypeStore { store: TypeStore::new() }
    }

    pub fn contains_type<T: 'static>(&self) -> bool {
        self.store.contains_type::<Vec<T>>()
    }

    pub fn contains_type_id(&self, type_id: TypeId) -> bool {
        self.store.contains_type_id(type_id)
    }

    pub fn insert<T: 'static>(&mut self, val: Vec<T>) {
        self.store.insert(val);
    }

    pub fn get<T: 'static>(&self) -> Option<&Vec<T>> {
        self.store.get::<Vec<T>>()
    }

    ////pub fn access_mut<T: 'static>(&mut self) -> Option<*mut Vec<T>> {
    ////    self.store.access_mut::<Vec<T>>()
    ////}

    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut Vec<T>> {
        self.store.get_mut::<Vec<T>>()
    }

    pub fn get_mut2<A: 'static, B: 'static>(&mut self) -> Option<(&mut Vec<A>, &mut Vec<B>)> {
        self.store.get_mut2::<Vec<A>, Vec<B>>()
    }
}

#[repr(C)]
pub struct FatPointer {
    pub obj_ptr: u64,
    pub trait_ptr: u64,
}

pub trait DynamicType {}
impl<T> DynamicType for T {}

pub struct TypeStore {
    store: HashMap<std::any::TypeId, *mut DynamicType>,
}
impl std::ops::Drop for TypeStore {
    fn drop(&mut self) {
        for &ptr in self.store.values().into_iter() {
            unsafe {
                Box::from_raw(ptr);
            }
        }
    }
}

impl TypeStore {
    pub fn type_len(&self) -> usize {
        self.store.keys().len()
    }

    pub fn contains_type<T: 'static>(&self) -> bool {
        self.contains_type_id(std::any::TypeId::of::<T>())
    }

    pub fn contains_type_id(&self, type_id: TypeId) -> bool {
        self.store.contains_key(&type_id)
    }

    pub fn insert<T: 'static>(&mut self, val: T) {
        let ptr = Box::into_raw(Box::new(val));
        let t = std::any::TypeId::of::<T>();
        self.store.insert(t, ptr);
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        let t = std::any::TypeId::of::<T>();
        self.store.get(&t).map(|&ptr| unsafe {
            let fat_ptr = std::mem::transmute::<*mut DynamicType, FatPointer>(ptr);
            let f: &T = std::mem::transmute(fat_ptr.obj_ptr);
            f
        })
    }

    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        let t = std::any::TypeId::of::<T>();
        self.store.get(&t).map(|&ptr| unsafe { ptr_as_ref!(ptr) })
    }

    pub fn get_mut2<A: 'static, B: 'static>(&mut self) -> Option<(&mut A, &mut B)> {
        let a = std::any::TypeId::of::<A>();
        let b = std::any::TypeId::of::<B>();
        let r1 = self.store.get(&a);
        let r2 = self.store.get(&b);
        if (r1.is_none() || r2.is_none()) {
            return None;
        }
        Some((unsafe { ptr_as_ref!(*r1.unwrap()) }, unsafe { ptr_as_ref!(*r2.unwrap()) }))
    }

    //pub fn get_mut3<A: 'static, B: 'static, C: 'static>(&mut self)
    //                                                    -> Option<(&mut A, &mut B, &mut C)> {
    //    let a = std::any::TypeId::of::<A>();
    //    let b = std::any::TypeId::of::<B>();
    //    let c = std::any::TypeId::of::<C>();
    //    let r1 = self.store.get(&a);
    //    let r2 = self.store.get(&b);
    //    let r3 = self.store.get(&c);
    //    if (r1.is_none() || r2.is_none()) {
    //        return None;
    //    }
    //    Some((unsafe { ptr_as_mut_ref(*r1.unwrap()) },
    //          unsafe { ptr_as_mut_ref(*r2.unwrap()) },
    //          unsafe { ptr_as_mut_ref(*r3.unwrap()) }))
    //}

    pub fn new() -> Self {
        TypeStore { store: HashMap::new() }
    }
}

pub struct Ecs {
    world: Vec<VecTypeStore>,
}
pub struct AddEntity<'r, A> {
    ecs: &'r mut Ecs,
    index: usize,
    _m: ::std::marker::PhantomData<A>,
}

impl<'r, A: 'static> AddEntity<'r, A> {
    pub fn add_entity(&mut self, a: A) {
        self.ecs.world[self.index].get_mut::<A>().unwrap().push(a);
    }
}
pub struct AddEntity2<'r, A, B> {
    ecs: &'r mut Ecs,
    index: usize,
    _m: ::std::marker::PhantomData<(A, B)>,
}

impl<'r, A: 'static, B: 'static> AddEntity2<'r, A, B> {
    pub fn add_entity2(&mut self, a: A, b: B) {
        let (va, vb) = self.ecs.world[self.index].get_mut2::<A, B>().unwrap();
        va.push(a);
        vb.push(b);
    }
}

impl Ecs {
    pub fn add_entity<A: 'static>(&mut self) -> AddEntity<A> {
        let index: usize = {
            let p = {
                self.world
                    .iter()
                    .position(|store| store.type_len() == 1 && store.contains_type::<A>())
            };
            let index = p.unwrap_or(self.world.len());
            if p.is_none() {
                let mut store = VecTypeStore::new();
                store.insert(Vec::<A>::new());
                self.world.push(store);
            }
            index
        };
        AddEntity {
            ecs: self,
            index: index,
            _m: ::std::marker::PhantomData,
        }
    }

    pub fn add_entity2<'r, A: 'static, B: 'static>(&mut self) -> AddEntity2<A, B> {
        let index: usize = {
            let p = {
                self.world
                    .iter()
                    .position(|store| {
                        store.type_len() == 2 &&
                        (store.contains_type::<A>() || store.contains_type::<B>())
                    })
            };
            let index = p.unwrap_or(self.world.len());
            if p.is_none() {
                let mut store = VecTypeStore::new();
                store.insert(Vec::<A>::new());
                store.insert(Vec::<B>::new());
                self.world.push(store);
            }
            index
        };
        AddEntity2 {
            ecs: self,
            index: index,
            _m: ::std::marker::PhantomData,
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

    pub fn new() -> Ecs {
        Ecs { world: Vec::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[derive(Debug)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Debug, Copy, Clone)]
    struct Velocity {
        x: f32,
        y: f32,
    }
    static N_POS_VEL: usize = 1000;
    static N_POS: usize = 9000;
    pub fn build() -> Ecs {
        let mut ecs = Ecs::new();
        {
            let mut entity_add_pv = ecs.add_entity2::<Position, Velocity>();
            for i in (0..N_POS_VEL) {
                let pos = Position {
                    x: 0.0,
                    y: i as f32,
                };
                let vel = Velocity { x: 1.0, y: 0.0 };
                entity_add_pv.add_entity2(pos, vel);
            }
        }
        {
            let mut entity_add_p = ecs.add_entity::<Position>();
            for i in (0..N_POS) {
                let pos = Position {
                    x: 9999.0 as f32,
                    y: 0.0,
                };
                entity_add_p.add_entity(pos);
            }
        }
        ecs
    }


    #[test]
    fn t1() {
        let mut t = TypeStore::new();
        //t.insert(Vec::<u32>::new());
        //{
        //    let v = {
        //        t.get_mut::<Vec<u32>>().unwrap().iter_mut()
        //    };
        //}
    }
    #[test]
    fn t() {
        //    let mut v = VecTypeStore::<PosVelDrop>::new();
        //    v.insert(Vec::<Position>::new());
        //    v.insert(Vec::<Velocity>::new());
        //    let mut a = v.access_mut::<Position>();
        //    let mut b = v.access_mut::<Velocity>();
        //    let i = a.iter_mut();
        //    let i2 = b.iter_mut();
    }
    //#[test]
    //fn test_type(){
    //    println!("{}", <i32 as Type<Test>>::has_type());
    //    println!("{}", <String as Type<Test>>::has_type());
    //}
    //#[test]
    //fn test_update() {
    //    let mut ecs = build();
    //    for _ in 0..10 {
    //        ecs.update2(|p: &mut Position, v: &mut Velocity| {
    //            p.x += v.x;
    //            p.y += v.y;
    //            //println!("{:?}", p);
    //        });
    //    }
    //    ecs.update(|p: &mut Position| {
    //        //println!("{:?}", p);
    //    });
    //}
    #[bench]
    fn bench_build(b: &mut Bencher) {
        b.iter(|| build());
    }
    #[bench]
    fn bench_update(b: &mut Bencher) {
        let mut ecs = build();
        b.iter(|| {
            ecs.update2(|p: &mut Position, v: &mut Velocity| {
                p.x += v.x;
                p.y += v.y;
            });
            ecs.update(|p: &mut Position| {
                //println!("{:?}", p);
            });
        });
    }
}
