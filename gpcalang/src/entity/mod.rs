use std::{cell::RefCell, fmt::Debug, ops::{Add, AddAssign}, rc::Rc};

use bytecode::{Event, Jump, RegConst, Register, Response};

use crate::{boolmap::BooleanMap, world::World};

mod bytecode;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MoveEntity {
    Right,
    TopRight,
    Top,
    TopLeft,
    Left,
    BottomLeft,
    Bottom,
    BottomRight,
}
impl From<u64> for MoveEntity {
    fn from(value: u64) -> Self {
        match value%8 {
            0 => MoveEntity::Right,
            1 => MoveEntity::TopRight,
            2 => MoveEntity::Top,
            3 => MoveEntity::TopLeft,
            4 => MoveEntity::Left,
            5 => MoveEntity::BottomLeft,
            6 => MoveEntity::Bottom,
            7 => MoveEntity::BottomRight,
            _ => unreachable!()
        }
    }
}
#[derive(Clone, Copy)]
union DataRegister {
    long: u64,
    byte: [u8; 4],
}
impl Debug for DataRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe {
            f.debug_tuple("Registers")
            .field(&self.byte[0])
            .field(&self.byte[1])
            .field(&self.byte[2])
            .field(&self.byte[3])
            .finish()
        }
    }
}
impl Default for DataRegister {
    fn default() -> Self {
        unsafe { Self { long: 0 } }
    }
}
impl DataRegister {
    pub fn set_byte_register(&mut self, val: u8, idx: usize) {
        unsafe {
            self.byte[idx] = val;
        }
    }
    pub fn set_long_register(&mut self, val: u64) {
        unsafe {
            self.long = val;
        }
    }
    pub fn add_byte_register(&mut self, val: u8, idx: usize) {
        unsafe {
            self.byte[idx] += val;
        }
    }
    pub fn sub_byte_register(&mut self, val: u8, idx: usize) {
        unsafe {
            self.byte[idx] -= val;
        }
    }
    pub fn add_long_register(&mut self, val: u64) {
        unsafe {
            self.long += val;
        }
    }
    pub fn sub_long_register(&mut self, val: u64) {
        unsafe {
            self.long -= val;
        }
    }
    pub fn eq_long_register(&mut self, val: u64) -> bool {
        unsafe {
            self.long == val
        }
    }
    pub fn neq_long_register(&mut self, val: u64) -> bool {
        unsafe {
            self.long != val
        }
    }
    pub fn greater_long_register(&mut self, val: u64) -> bool {
        unsafe {
            self.long > val
        }
    }
    pub fn lesser_long_register(&mut self, val: u64) -> bool {
        unsafe {
            self.long < val
        }
    }
    pub fn greater_eq_long_register(&mut self, val: u64) -> bool {
        unsafe {
            self.long >= val
        }
    }
    pub fn lesser_eq_long_register(&mut self, val: u64) -> bool {
        unsafe {
            self.long <= val
        }
    }
}
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GPCAEntityInternal {
    registers: [DataRegister; 2],
    pub(crate) pos: [u32; 2],
    energy: u32,
}

impl GPCAEntityInternal {
    pub fn new(x: u32, y: u32, reg0: u64, reg1: u64, energy: u32) -> Self {
        Self { registers: [DataRegister { long: reg0 }, DataRegister { long: reg1 }], pos: [x, y], energy }
    }
    pub fn get(&self, register: Register) -> u64 {
        unsafe {
            match register {
                Register::ByteRegister0_0 => self.registers[0].byte[0] as u64,
                Register::ByteRegister0_1 => self.registers[0].byte[1] as u64,
                Register::ByteRegister0_2 => self.registers[0].byte[2] as u64,
                Register::ByteRegister0_3 => self.registers[0].byte[3] as u64,
                Register::ByteRegister1_0 => self.registers[1].byte[0] as u64,
                Register::ByteRegister1_1 => self.registers[1].byte[1] as u64,
                Register::ByteRegister1_2 => self.registers[1].byte[2] as u64,
                Register::ByteRegister1_3 => self.registers[1].byte[3] as u64,
                Register::LongRegister0 => self.registers[0].long,
                Register::LongRegister1 => self.registers[1].long,
            }
        }
    }
    pub fn get_const(&self, register: RegConst) -> u64 {
        unsafe {
            match register {
                RegConst::Constant(constant) => constant as u64,
                RegConst::Register(reg) => self.get(reg),
            }
        }
    }
    pub fn set_register(&mut self, register: Register, val: u64) {
        unsafe {
            match register {
                Register::ByteRegister0_0 => self.registers[0].byte[0] = val as u8,
                Register::ByteRegister0_1 => self.registers[0].byte[1] = val as u8,
                Register::ByteRegister0_2 => self.registers[0].byte[2] = val as u8,
                Register::ByteRegister0_3 => self.registers[0].byte[3] = val as u8,
                Register::ByteRegister1_0 => self.registers[1].byte[0] = val as u8,
                Register::ByteRegister1_1 => self.registers[1].byte[1] = val as u8,
                Register::ByteRegister1_2 => self.registers[1].byte[2] = val as u8,
                Register::ByteRegister1_3 => self.registers[1].byte[3] = val as u8,
                Register::LongRegister0 => self.registers[0].long = val,
                Register::LongRegister1 => self.registers[1].long = val,
            }
        }
    }
    pub fn move_step(&mut self, step: MoveEntity, world: &Rc<RefCell<BooleanMap>>) {
        let mut world = world.borrow_mut();
        world.set(self.pos[0], self.pos[1], false);
        let prev = self.pos;
        match step {
            MoveEntity::Right => { 
                self.pos[0] = self.pos[0].add(1).clamp(0, world.width);
            }
            MoveEntity::TopRight => { 
                self.pos[0] = self.pos[0].add(1).clamp(0, world.width);
                self.pos[1] = self.pos[1].add(1).clamp(0, world.height); 
            }
            MoveEntity::Top => { 
                self.pos[1] = self.pos[1].add(1).clamp(0, world.height); 
            }
            MoveEntity::TopLeft => { 
                self.pos[0] = if self.pos[0] == 0 { self.pos[0] } else { self.pos[0]-1 };
                self.pos[1] = self.pos[1].add(1).clamp(0, world.height); 
            }
            MoveEntity::Left => { 
                self.pos[0] = if self.pos[0] == 0 { self.pos[0] } else { self.pos[0]-1 };
            }
            MoveEntity::BottomLeft => { 
                self.pos[0] = if self.pos[0] == 0 { self.pos[0] } else { self.pos[0]-1 };
                self.pos[1] = if self.pos[1] == 0 { self.pos[1] } else { self.pos[1]-1 }; 
            }
            MoveEntity::Bottom => { 
                self.pos[1] = if self.pos[1] == 0 { self.pos[1] } else { self.pos[1]-1 }; 
            }
            MoveEntity::BottomRight => { 
                self.pos[0] = self.pos[0].add(1).clamp(0, world.width);
                self.pos[1] = if self.pos[1] == 0 { self.pos[1] } else { self.pos[1]-1 }; 
            }
        }
        if world.get(self.pos[0], self.pos[1]) { // space is already occupied
            self.pos = prev;
        } 
        world.set(self.pos[0], self.pos[1], true);
    }
    pub fn handle_event(&mut self, event: Event, world: &Rc<RefCell<BooleanMap>>) -> bool {
        let world = world.borrow();
        match event {
            Event::Equal(lhs, rhs) => {
                let (lhs, rhs) = (self.get(lhs), self.get(rhs));
                lhs == rhs
            }
            Event::NotEqual(lhs, rhs) => {
                let (lhs, rhs) = (self.get(lhs), self.get(rhs));
                lhs != rhs
            }
            Event::Greater(lhs, rhs) => {
                let (lhs, rhs) = (self.get(lhs), self.get(rhs));
                lhs > rhs
            }
            Event::Lesser(lhs, rhs) => {
                let (lhs, rhs) = (self.get(lhs), self.get(rhs));
                lhs < rhs
            }
            Event::GreaterEqual(lhs, rhs) => {
                let (lhs, rhs) = (self.get(lhs), self.get(rhs));
                lhs >= rhs
            }
            Event::LesserEqual(lhs, rhs) => {
                let (lhs, rhs) = (self.get(lhs), self.get(rhs));
                lhs <= rhs
            }
            Event::SurroundingSquaresEqual(lhs) => {
                let lhs = self.get_const(lhs);
                lhs == world.surrounding_square_count(self.pos[0], self.pos[1]) as u64
            }
            Event::SurroundingSquaresNotEqual(lhs) => {
                let lhs = self.get_const(lhs);
                lhs != world.surrounding_square_count(self.pos[0], self.pos[1]) as u64
            }
            Event::SurroundingSquaresGreater(lhs) => {
                let lhs = self.get_const(lhs);
                lhs > world.surrounding_square_count(self.pos[0], self.pos[1]) as u64
            }
            Event::SurroundingSquaresLesser(lhs) => {
                let lhs = self.get_const(lhs);
                lhs < world.surrounding_square_count(self.pos[0], self.pos[1]) as u64
            }
            Event::SurroundingSquaresGreaterEqual(lhs) => {
                let lhs = self.get_const(lhs);
                lhs >= world.surrounding_square_count(self.pos[0], self.pos[1]) as u64
            }
            Event::SurroundingSquaresLesserEqual(lhs) => {
                let lhs = self.get_const(lhs);
                lhs <= world.surrounding_square_count(self.pos[0], self.pos[1]) as u64
            }
            Event::Unconditional => {
                true
            }
        }
    }
}

pub struct GPCAEntity {
    internal: GPCAEntityInternal,
    rip: usize,
    pub color: u32,
    code: Vec<u32>,
}
#[derive(Debug, Clone, Copy)]
pub struct EventResponse {
    pub event: Event,
    pub response: Response,
}
impl GPCAEntity {
    pub fn new(x: u32, y: u32, reg0: u64, reg1: u64, energy: u32, color: u32, code: Vec<u32>) -> Self {
        Self { internal: GPCAEntityInternal::new(x, y, reg0, reg1, energy), color, rip: 0, code }
    }
    pub fn parse(&self) -> Option<EventResponse> {
        let code = self.code.get(self.rip)?;
        let event = ((code >> 16)&0xffff) as u16;
        let response= (code&0xffff) as u16;
        Some(EventResponse { event: Event::from(event), response: Response::from(response) })
    }
    /// [`clear`] when an entity is deleted or moved, this function will be used to
    /// clear the current spot.
    /// 
    /// [`place`] when an entity is moved this function will be used to place the next
    /// spot.
    /// 
    /// returns whether the entity should be deleted. if true then delete else continue.
    pub fn step<F, H>(&mut self, 
        world: &World, 
        mut clear: F, 
        mut place: H) -> bool
        where F: FnMut(&GPCAEntity),
            H: FnMut(&GPCAEntity) {
        if !world.map.borrow().get(self.x(), self.y()) {
            clear(self);
            return true;
        }
        let event_response = self.next().unwrap_or_else(||{self.next().unwrap()});
        if self.handle_event(event_response.event, &world.map) {
            if event_response.response.is_move_step() {
                clear(self);
                self.handle_response(event_response.response, world);
                place(self);
            } else {
                self.handle_response(event_response.response, world);
            }
        }
        false
    }
    pub fn handle_event(&mut self, event: Event, world: &Rc<RefCell<BooleanMap>>) -> bool {
        self.internal.handle_event(event, world)
    }
    pub fn handle_response(&mut self, response: Response, world: &World) {
        match response {
            Response::BinaryOp(bytecode::BinaryOp::Add(lhs, rhs)) => {
                let lhs_val = self.internal.get(lhs);
                let rhs_val = self.internal.get_const(rhs);
                self.internal.set_register(lhs, lhs_val.wrapping_add(rhs_val));
            }
            Response::BinaryOp(bytecode::BinaryOp::Sub(lhs, rhs)) => {
                let lhs_val = self.internal.get(lhs);
                let rhs_val = self.internal.get_const(rhs);
                self.internal.set_register(lhs, lhs_val.wrapping_sub(rhs_val));
            }
            Response::BinaryOp(bytecode::BinaryOp::Mul(lhs, rhs)) => {
                let lhs_val = self.internal.get(lhs);
                let rhs_val = self.internal.get_const(rhs);
                self.internal.set_register(lhs, lhs_val.wrapping_mul(rhs_val));
            }
            Response::BinaryOp(bytecode::BinaryOp::Div(lhs, rhs)) => {
                let lhs_val = self.internal.get(lhs);
                let rhs_val = self.internal.get_const(rhs);
                if rhs_val == 0 {
                    self.internal.set_register(lhs, u64::MAX);
                } else {
                    self.internal.set_register(lhs, lhs_val.wrapping_sub(rhs_val));
                }
            }
            Response::BinaryOp(bytecode::BinaryOp::And(lhs, rhs)) => {
                let lhs_val = self.internal.get(lhs);
                let rhs_val = self.internal.get_const(rhs);
                self.internal.set_register(lhs, lhs_val&rhs_val);
            }
            Response::BinaryOp(bytecode::BinaryOp::Xor(lhs, rhs)) => {
                let lhs_val = self.internal.get(lhs);
                let rhs_val = self.internal.get_const(rhs);
                self.internal.set_register(lhs, lhs_val^rhs_val);
            }
            Response::BinaryOp(bytecode::BinaryOp::Or(lhs, rhs)) => {
                let lhs_val = self.internal.get(lhs);
                let rhs_val = self.internal.get_const(rhs);
                self.internal.set_register(lhs, lhs_val|rhs_val);
            }
            Response::BinaryOp(bytecode::BinaryOp::MoveAdd(lhs, rhs)) => {
                let lhs_val = self.internal.get(lhs);
                let rhs_val = self.internal.get_const(rhs);
                let val = lhs_val.wrapping_add(rhs_val);
                let step = MoveEntity::from(val);
                self.internal.move_step(step, &world.map);
            }
            Response::BinaryOp(bytecode::BinaryOp::MoveSub(lhs, rhs)) => {
                let lhs_val = self.internal.get(lhs);
                let rhs_val = self.internal.get_const(rhs);
                let val = lhs_val.wrapping_sub(rhs_val);
                let step = MoveEntity::from(val);
                self.internal.move_step(step, &world.map);
            }
            Response::BinaryOp(bytecode::BinaryOp::MoveMul(lhs, rhs)) => {
                let lhs_val = self.internal.get(lhs);
                let rhs_val = self.internal.get_const(rhs);
                let val = lhs_val.wrapping_mul(rhs_val);
                let step = MoveEntity::from(val);
                self.internal.move_step(step, &world.map);
            }
            Response::BinaryOp(bytecode::BinaryOp::MoveDiv(lhs, rhs)) => {
                let lhs_val = self.internal.get(lhs);
                let rhs_val = self.internal.get_const(rhs);
                let val = if rhs_val == 0 {
                    u64::MAX
                } else {
                    lhs_val.wrapping_sub(rhs_val)
                };
                let step = MoveEntity::from(val);
                self.internal.move_step(step, &world.map);
            }
            Response::BinaryOp(bytecode::BinaryOp::MoveAnd(lhs, rhs)) => {
                let lhs_val = self.internal.get(lhs);
                let rhs_val = self.internal.get_const(rhs);
                let val = lhs_val&rhs_val;
                let step = MoveEntity::from(val);
                self.internal.move_step(step, &world.map);
            }
            Response::BinaryOp(bytecode::BinaryOp::MoveXor(lhs, rhs)) => {
                let lhs_val = self.internal.get(lhs);
                let rhs_val = self.internal.get_const(rhs);
                let val = lhs_val^rhs_val;
                let step = MoveEntity::from(val);
                self.internal.move_step(step, &world.map);
            }
            Response::BinaryOp(bytecode::BinaryOp::MoveOr(lhs, rhs)) => {
                let lhs_val = self.internal.get(lhs);
                let rhs_val = self.internal.get_const(rhs);
                let val = lhs_val|rhs_val;
                let step = MoveEntity::from(val);
                self.internal.move_step(step, &world.map);
            }
            Response::BinaryOp(bytecode::BinaryOp::Mov(lhs, rhs)) => {
                let rhs_val = self.internal.get_const(rhs);
                self.internal.set_register(lhs, rhs_val);
            }
            Response::BinaryOp(bytecode::BinaryOp::Xchg(lhs, rhs)) => {
                let lhs_val = self.internal.get(lhs);
                let rhs_val = self.internal.get(rhs);
                self.internal.set_register(lhs, rhs_val);
                self.internal.set_register(rhs, lhs_val);
            }
            Response::Call(reg) => {
                let get = self.internal.get_const(reg) as usize;
                
                if !world.functions.is_empty() {
                    world.functions[get%world.functions.len()](self, world);
                }
            }
            Response::Move(reg) => {
                let get = self.internal.get_const(reg);
                let step = MoveEntity::from(get);
                self.internal.move_step(step, &world.map);
            }
            Response::Jmp(reg) => {
                let reg0 = unsafe { self.internal.registers[0].long };
                let reg1 = unsafe { self.internal.registers[1].long };
                let (should_jump) = match reg {
                    bytecode::Jump::Reg0Eq(_) => {
                        reg0 == reg1
                    }
                    bytecode::Jump::Reg0Greater(_) => {
                        reg0 > reg1
                    }
                    bytecode::Jump::Reg0GreaterEq(_) => {
                        reg0 >= reg1
                    }
                    bytecode::Jump::Reg0Lesser(_) => {
                        reg0 <= reg1
                    }
                    bytecode::Jump::Reg0LesserEq(_) => {
                        reg0 <= reg1
                    }
                    bytecode::Jump::Reg0Neq(_) => {
                        reg0 != reg1
                    }
                    bytecode::Jump::Reg1Eq(_) => {
                        reg1 == reg0
                    }
                    bytecode::Jump::Reg1Greater(_) => {
                        reg1 > reg0
                    }
                    bytecode::Jump::Reg1GreaterEq(_) => {
                        reg1 >= reg0
                    }
                    bytecode::Jump::Reg1Lesser(_) => {
                        reg1 <= reg0
                    }
                    bytecode::Jump::Reg1LesserEq(_) => {
                        reg1 <= reg0
                    }
                    bytecode::Jump::Reg1Neq(_) => {
                        reg1 != reg0
                    }
                    bytecode::Jump::Unconditional(_) => {
                        true
                    }
                };
                if should_jump {
                    let (bytecode::Jump::Reg0Eq(jmp)    | bytecode::Jump::Reg0Greater(jmp)  | 
                    bytecode::Jump::Reg0GreaterEq(jmp)  | bytecode::Jump::Reg0Lesser(jmp)   | 
                    bytecode::Jump::Reg0LesserEq(jmp)   | bytecode::Jump::Reg0Neq(jmp)      | 
                    bytecode::Jump::Unconditional(jmp) | bytecode::Jump::Reg1Eq(jmp)    | bytecode::Jump::Reg1Greater(jmp)  | 
                    bytecode::Jump::Reg1GreaterEq(jmp)  | bytecode::Jump::Reg1Lesser(jmp)   | 
                    bytecode::Jump::Reg1LesserEq(jmp)   | bytecode::Jump::Reg1Neq(jmp)) = reg;
                    let len = self.code.len() as isize;
                    let rip = self.rip as isize;
                    let jmp_loc = ((jmp as isize + rip)%len) as usize;
                    self.rip = jmp_loc;
                }
            }
            Response::Nop => {}
        }
    }
    pub fn x(&self) -> u32 {
        self.internal.pos[0]
    }
    pub fn y(&self) -> u32 {
        self.internal.pos[1]
    }
    pub fn next_rip(&mut self) {
        if self.rip < self.code.len() {
            self.rip += 1;
        } else {
            self.rip = 0;
        }
    }
    pub fn execute_next(&mut self, world: &World) {
        let next = self.next().unwrap_or_else(||{self.next().unwrap()});
        if self.handle_event(next.event, &world.map) {
            self.handle_response(next.response, world);
        }
    }
    pub fn inner(&self) -> GPCAEntityInternal {
        self.internal
    }
}

impl Iterator for GPCAEntity {
    type Item = EventResponse;
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.parse();
        self.next_rip();
        next
    }
}