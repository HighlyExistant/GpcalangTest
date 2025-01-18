use std::{cell::{RefCell, UnsafeCell}, collections::HashMap, rc::Rc, sync::Arc};

use affogato::spatial::morton::MortonU64;

use super::entity::{Direction, GPCAEntity, GPCAEntityInternal};

type WorldUserFunction = fn(&Arc<GPCAEntity>, &World);

pub struct World {
    pub(crate) functions: Vec<WorldUserFunction>,
    entities: Rc<UnsafeCell<Vec<Arc<GPCAEntity>>>>,
    // pub(crate) map: Rc<RefCell<BooleanMap>>,
    pub(crate) map: Rc<RefCell<HashMap<MortonU64, Arc<GPCAEntity>>>>,
    pseudo: UnsafeCell<rand_pcg::Pcg64>,
    width: u32, 
    height: u32,
    use_energy: bool
}

impl World {
    pub fn new(functions: Vec<WorldUserFunction>, entity_capacity: usize, width: u32, height: u32, use_energy: bool) -> World {
        Self { functions, entities: Rc::new(UnsafeCell::new(Vec::with_capacity(entity_capacity))), map: Rc::new(RefCell::new(HashMap::with_capacity(entity_capacity))), pseudo: UnsafeCell::new(rand_pcg::Pcg64::new(0xcafef00dd15ea5e5, 0xa02bdbf7bb3c0a7ac28fa16a64abf96)), width, height, use_energy }
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
                    continue;
                }
                entities[i].decrement_energy();
            }
            if entities[i].step(self, &mut clear, &mut place) {
                if i == (entities.len()-1) {
                    let _ = entities.pop().unwrap();
                } else {
                    entities[i] = entities.pop().unwrap();
                }
                continue;
            }
            i += 1;
        }
        // for i in 0..entities.len() {
        // }
    }
    pub fn pseudo(&self) -> &mut rand_pcg::Lcg128Xsl64 {
        unsafe { self.pseudo.get().as_mut().unwrap() }
    }
    pub fn get(&self, x: u32, y: u32) -> bool {
        let map = self.map.borrow();
        if x >= self.width || y >= self.height {
            return true;
        }
        map.get(&MortonU64::encode_xy(x, y)).is_some()
    }
    pub fn get_entity_at_direction(&self, entity: &GPCAEntityInternal, dir: Direction) -> Option<Arc<GPCAEntity>> {
        let map = self.map.borrow();
        let mut pos = entity.pos;
        dir.perform_direction(&mut pos, self.width, self.height);
        map.get(&MortonU64::encode_xy(pos[0], pos[1])).cloned()
    }
    pub fn set(&self, entity: &Arc<GPCAEntity>) {
        assert!(entity.inner().pos[0] < self.width && entity.inner().pos[0] < self.height, "x and y can not exceed width and height respectively");
        // let mut map = self.map.borrow_mut();
        // map.set(x, y, active);
        let mut map = self.map.borrow_mut();
        map.insert(MortonU64::encode_xy(entity.inner().pos[0], entity.inner().pos[1]), entity.clone());
    }
    pub fn remove(&self, x: u32, y: u32) {
        // let mut map = self.map.borrow_mut();
        // map.set(x, y, active);
        let mut map = self.map.borrow_mut();
        map.remove(&MortonU64::encode_xy(x, y));
    }
    pub fn push(&self, entity: GPCAEntity) {
        let mut entities = self.get_entites_mut();
        entities.push(entity.into());
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
}