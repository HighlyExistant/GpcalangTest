use std::{cell::RefCell, rc::Rc};

use boolmap::BooleanMap;

use crate::entity::GPCAEntity;
pub mod boolmap;
type WorldUserFunction = fn(&mut GPCAEntity, &World);

pub struct World {
    pub(crate) functions: Vec<WorldUserFunction>,
    entities: Rc<RefCell<Vec<GPCAEntity>>>,
    pub(crate) map: Rc<RefCell<BooleanMap>>,
    pseudo: rand_pcg::Pcg64,
}

impl World {
    pub fn new(functions: Vec<WorldUserFunction>, entity_capacity: usize, width: u32, height: u32) -> World {
        Self { functions, entities: Rc::new(RefCell::new(Vec::with_capacity(entity_capacity))), map: Rc::new(RefCell::new(BooleanMap::new(width, height))), pseudo: rand_pcg::Pcg64::new(0xcafef00dd15ea5e5, 0xa02bdbf7bb3c0a7ac28fa16a64abf96) }
    }
    pub fn surrounding_square_count(&self, x: u32, y: u32) -> usize {
        let map = self.map.borrow();
        map.surrounding_square_count(x, y)
    }
    pub fn push_entity(&self, entity: GPCAEntity) {
        let mut entities = self.entities.borrow_mut();
        let mut map = self.map.borrow_mut();
        map.set(entity.x(), entity.y(), true);
        entities.push(entity);
    }
    pub fn step<F, H>(&mut self, mut clear: F, mut place: H) 
        where F: FnMut(&GPCAEntity),
            H: FnMut(&GPCAEntity) {
        let mut entities = self.entities.borrow_mut();
        for i in 0..entities.len() {
            // let next = entities[i].next().unwrap_or_else(||{entities[i].next().unwrap()});
            // if entities[i].handle_event(next.event, &self.map) {
            //     if next.response.is_move_step() {
            //         clear(&entities[i]);
            //         entities[i].handle_response(next.response, self);
            //         place(&entities[i]);
            //     } else {
            //         entities[i].handle_response(next.response, self);
            //     }
            // }
            if entities[i].step(self, &mut clear, &mut place) {
                println!("DELETE");
            }
        }
    }
    pub fn get(&self, x: u32, y: u32) -> bool {
        let map = self.map.borrow();
        map.get(x, y)
    }
    pub fn set(&mut self, x: u32, y: u32, active: bool) {
        let mut map = self.map.borrow_mut();
        map.set(x, y, active);
    }
    pub fn push(&mut self, entity: GPCAEntity) {
        let mut entities = self.entities.borrow_mut();
        entities.push(entity);
    }
    pub fn width(&self) -> u32 {
        let map = self.map.borrow();
        map.width
    }
    pub fn height(&self) -> u32 {
        let map = self.map.borrow();
        map.height
    }
}