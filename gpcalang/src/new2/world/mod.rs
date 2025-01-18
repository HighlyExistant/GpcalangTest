use std::{cell::{RefCell, UnsafeCell}, collections::HashMap, rc::Rc, sync::Arc};

use affogato::spatial::morton::MortonU64;
use rand::Rng;

use super::entity::{Direction, GPCAEntity, GPCAEntityInternal};

type WorldUserFunction = fn(&Arc<GPCAEntity>, &World);

pub struct World {
    pub(crate) functions: Vec<WorldUserFunction>,
    entities: Rc<UnsafeCell<Vec<Arc<GPCAEntity>>>>,
    // pub(crate) map: Rc<RefCell<BooleanMap>>,
    // pub(crate) map: Rc<RefCell<HashMap<MortonU64, Arc<GPCAEntity>>>>,
    map: Rc<RefCell<Vec<u32>>>,
    pseudo: UnsafeCell<rand_pcg::Pcg64>,
    width: u32, 
    height: u32,
    pub(crate) use_energy: bool,
    mutation_chance: f64,
}

impl World {
    pub fn new(functions: Vec<WorldUserFunction>, entity_capacity: usize, width: u32, height: u32, use_energy: bool, mutation_chance: f64, state: Option<u128>) -> World {
        Self { functions, entities: Rc::new(UnsafeCell::new(Vec::with_capacity(entity_capacity))), map: Rc::new(RefCell::new(vec![0xffffffff; (width*height) as usize])), pseudo: UnsafeCell::new(rand_pcg::Pcg64::new(state.unwrap_or(0xcafef00dd15ea5e5), 0xa02bdbf7bb3c0a7ac28fa16a64abf96)), width, height, use_energy, mutation_chance }
    }
    // pub fn surrounding_square_count(&self, x: u32, y: u32) -> usize {
    //     let map = self.map.borrow();
    //     map.surrounding_square_count(x, y)
    // }
    pub fn surrounding_square_count(&self, x: u32, y: u32) -> usize {
        let mut count = 0;
        for y in (y.checked_sub(1).unwrap_or_default())..((y+1).clamp(0, self.height)) {
            for x in (x.checked_sub(1).unwrap_or_default())..((x+1).clamp(0, self.width)) {
                count += self.get(x, y) as usize;
            }
        }
        count
    }
    pub fn get_entites(&self) -> &Vec<Arc<GPCAEntity>> {
        unsafe { self.entities.get().as_ref().unwrap() }
    }
    pub fn get_entites_mut(&self) -> &mut Vec<Arc<GPCAEntity>> {
        unsafe { self.entities.get().as_mut().unwrap() }
    }
    pub fn push_entity(&self, entity: GPCAEntity) {
        let entities = self.get_entites_mut();
        entity.inner_mut().id = entities.len() as u32;
        let entity = Arc::new(entity);
        self.set(&entity);
        entities.push(entity.clone());
    }
    pub fn create_entity(&self, mut entity: GPCAEntity) {
        let pseudo = unsafe { self.pseudo.get().as_mut().unwrap() };
        if self.mutation_chance != 0.0 {
            if pseudo.gen_bool(self.mutation_chance) {
                let code_to_mutate = pseudo.gen_range(0..entity.code.len());
                let bit_to_mutate = pseudo.gen_range(0..32);
                entity.code[code_to_mutate] ^= 1<<bit_to_mutate;
                println!("MUTATED");
            }
        }
        let entities = self.get_entites_mut();
        entity.inner_mut().id = entities.len() as u32;
        let entity = Arc::new(entity);
        self.set(&entity);
        entities.push(entity.clone());
    }
    pub fn step<F, H>(&self, mut clear: F, mut place: H) 
        where F: FnMut(&GPCAEntity),
            H: FnMut(&GPCAEntity) {
        let entities = self.get_entites_mut();
        let mut i = 0;
        while i < entities.len() {
            if self.use_energy {
                if entities[i].get_energy() == 0 {
                    self.remove(entities[i].x(), entities[i].y());
                } else {
                    entities[i].decrement_energy();
                }
            }
            if entities[i].step(self, &mut clear, &mut place) {
                if i == (entities.len()-1) {
                    let _ = entities.pop().unwrap();
                } else {
                    let id = entities[i].inner().id;
                    // self.remove(entities[i].inner().x(), entities[i].inner().y());
                    entities[i] = entities.pop().unwrap();
                    entities[i].inner_mut().id = id;
                    self.set(&entities[i]);
                }
                continue;
            }
            i += 1;
        }
        // for i in 0..entities.len() {
        // }
    }
    pub fn get(&self, x: u32, y: u32) -> bool {
        let map = self.map.borrow();
        if x >= self.width || y >= self.height {
            return true;
        }
        map[self.linear(x, y)] != 0xffffffff
    }
    pub fn pseudo(&self) -> &mut rand_pcg::Lcg128Xsl64 {
        unsafe { self.pseudo.get().as_mut().unwrap() }
    }
    fn linear(&self, x: u32, y: u32) -> usize {
        (x+y*self.width) as usize
    }
    pub fn get_entity_at_direction(&self, entity: &GPCAEntityInternal, dir: Direction) -> Option<Arc<GPCAEntity>> {
        let map = self.map.borrow();
        let mut pos = entity.pos;
        dir.perform_direction(&mut pos, self.width, self.height);
        
        let idx = map.get(self.linear(pos[0], pos[1])).cloned()?;
        if idx == 0xffffffff {
            None
        } else {
            let entities = self.get_entites();
            Some(entities[idx as usize].clone())
        }
    }
    pub fn set(&self, entity: &Arc<GPCAEntity>) {
        assert!(entity.inner().pos[0] < self.width && entity.inner().pos[0] < self.height, "x and y can not exceed width and height respectively");
        let mut map = self.map.borrow_mut();

        map[self.linear(entity.inner().pos[0], entity.inner().pos[1])] = entity.inner().id;
    }
    pub fn remove(&self, x: u32, y: u32) {
        // let mut map = self.map.borrow_mut();
        // map.set(x, y, active);
        let mut map = self.map.borrow_mut();
        map[self.linear(x, y)] = 0xffffffff;
    }
    pub fn push(&self, entity: GPCAEntity) {
        let entities = self.get_entites_mut();
        entities.push(entity.into());
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
}