use std::{cell::{RefCell, UnsafeCell}, fmt::Debug, ops::{Add, AddAssign}, rc::Rc, sync::Arc};

use bytecode::{Event, Jump, RegConst, Register, Response};

use super::world::World;

mod bytecode;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Direction {
    Right,
    TopRight,
    Top,
    TopLeft,
    Left,
    BottomLeft,
    Bottom,
    BottomRight,
}
impl Direction {
    pub fn perform_direction(&self, dir: &mut [u32; 2], width: u32, height: u32) {
        match self {
            Direction::Right => { 
                dir[0] = dir[0].add(1).clamp(0, width);
            }
            Direction::TopRight => { 
                dir[0] = dir[0].add(1).clamp(0, width);
                dir[1] = dir[1].add(1).clamp(0, height); 
            }
            Direction::Top => { 
                dir[1] = dir[1].add(1).clamp(0, height); 
            }
            Direction::TopLeft => { 
                dir[0] = if dir[0] == 0 { dir[0] } else { dir[0]-1 };
                dir[1] = dir[1].add(1).clamp(0, height); 
            }
            Direction::Left => { 
                dir[0] = if dir[0] == 0 { dir[0] } else { dir[0]-1 };
            }
            Direction::BottomLeft => { 
                dir[0] = if dir[0] == 0 { dir[0] } else { dir[0]-1 };
                dir[1] = if dir[1] == 0 { dir[1] } else { dir[1]-1 }; 
            }
            Direction::Bottom => { 
                dir[1] = if dir[1] == 0 { dir[1] } else { dir[1]-1 }; 
            }
            Direction::BottomRight => { 
                dir[0] = dir[0].add(1).clamp(0, width);
                dir[1] = if dir[1] == 0 { dir[1] } else { dir[1]-1 }; 
            }
        }
    }
}
impl From<u64> for Direction {
    fn from(value: u64) -> Self {
        match value%8 {
            0 => Direction::Right,
            1 => Direction::TopRight,
            2 => Direction::Top,
            3 => Direction::TopLeft,
            4 => Direction::Left,
            5 => Direction::BottomLeft,
            6 => Direction::Bottom,
            7 => Direction::BottomRight,
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
    pub fn eq_long_register(&self, val: u64) -> bool {
        unsafe {
            self.long == val
        }
    }
    pub fn neq_long_register(&self, val: u64) -> bool {
        unsafe {
            self.long != val
        }
    }
    pub fn greater_long_register(&self, val: u64) -> bool {
        unsafe {
            self.long > val
        }
    }
    pub fn lesser_long_register(&self, val: u64) -> bool {
        unsafe {
            self.long < val
        }
    }
    pub fn greater_eq_long_register(&self, val: u64) -> bool {
        unsafe {
            self.long >= val
        }
    }
    pub fn lesser_eq_long_register(&self, val: u64) -> bool {
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
    pub(crate) id: u32,
    energy: u32,
    rip: usize,
}

impl GPCAEntityInternal {
    pub fn new(x: u32, y: u32, id: u32, reg0: u64, reg1: u64, energy: u32) -> Self {
        Self { registers: [DataRegister { long: reg0 }, DataRegister { long: reg1 }], pos: [x, y], id, energy, rip: 0 }
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
    pub fn handle_event(&self, event: Event, world: &World) -> bool {
        // let world = world.borrow();
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
    pub fn x(&self) -> u32 {
        self.pos[0]
    }
    pub fn y(&self) -> u32 {
        self.pos[1]
    }
    pub fn decrement_energy(&mut self) {
        self.energy -= 1;
    }
    pub fn set_energy(&mut self, energy: u32) {
        self.energy = energy;
    }
    pub fn get_energy(&self) -> u32 {
        self.energy
    }
}

pub struct GPCAEntity {
    internal: UnsafeCell<GPCAEntityInternal>,
    pub color: u32,
    pub code: Vec<u32>,
}
#[derive(Debug, Clone, Copy)]
pub struct EventResponse {
    pub event: Event,
    pub response: Response,
}
impl GPCAEntity {
    pub fn new(x: u32, y: u32, id: u32, reg0: u64, reg1: u64, energy: u32, color: u32, code: Vec<u32>) -> Self {
        Self { internal: UnsafeCell::new(GPCAEntityInternal::new(x, y, id, reg0, reg1, energy)), color, code }
    }
    pub fn parse(&self) -> Option<EventResponse> {
        let code = self.code.get(self.inner().rip)?;
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
    pub fn step<F, H>(self: &Arc<Self>, 
        world: &World, 
        mut clear: F, 
        mut place: H) -> bool
        where F: FnMut(&GPCAEntity),
            H: FnMut(&GPCAEntity) {
        if !world.get(self.x(), self.y()) {
            clear(self);
            return true;
        }
        let event_response = self.next().unwrap_or_else(||{self.next().unwrap()});
        if self.handle_event(event_response.event, &world) {
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
    pub fn handle_event(&self, event: Event, world: &World) -> bool {
        self.inner_mut().handle_event(event, world)
    }
    pub fn move_step(self: &Arc<Self>, step: Direction, world: &World) {
        // let mut world = world.borrow_mut();
        let prev = self.inner().pos;
        step.perform_direction(&mut self.inner_mut().pos, world.width(), world.height());
        
        if world.get(self.inner().pos[0], self.inner().pos[1]) { // space is already occupied
            self.inner_mut().pos = prev;
        } else {
            world.remove(prev[0], prev[1]);
            world.set(&self);
        }
    }
    pub fn handle_response(self: &Arc<Self>, response: Response, world: &World) {
        match response {
            Response::BinaryOp(bytecode::BinaryOp::Add(lhs, rhs)) => {
                let lhs_val = self.inner().get(lhs);
                let rhs_val = self.inner().get_const(rhs);
                self.inner_mut().set_register(lhs, lhs_val.wrapping_add(rhs_val));
            }
            Response::BinaryOp(bytecode::BinaryOp::Sub(lhs, rhs)) => {
                let lhs_val = self.inner().get(lhs);
                let rhs_val = self.inner().get_const(rhs);
                self.inner_mut().set_register(lhs, lhs_val.wrapping_sub(rhs_val));
            }
            Response::BinaryOp(bytecode::BinaryOp::Mul(lhs, rhs)) => {
                let lhs_val = self.inner().get(lhs);
                let rhs_val = self.inner().get_const(rhs);
                self.inner_mut().set_register(lhs, lhs_val.wrapping_mul(rhs_val));
            }
            Response::BinaryOp(bytecode::BinaryOp::Div(lhs, rhs)) => {
                let lhs_val = self.inner().get(lhs);
                let rhs_val = self.inner().get_const(rhs);
                if rhs_val == 0 {
                    self.inner_mut().set_register(lhs, u64::MAX);
                } else {
                    self.inner_mut().set_register(lhs, lhs_val.wrapping_sub(rhs_val));
                }
            }
            Response::BinaryOp(bytecode::BinaryOp::And(lhs, rhs)) => {
                let lhs_val = self.inner().get(lhs);
                let rhs_val = self.inner().get_const(rhs);
                self.inner_mut().set_register(lhs, lhs_val&rhs_val);
            }
            Response::BinaryOp(bytecode::BinaryOp::Xor(lhs, rhs)) => {
                let lhs_val = self.inner().get(lhs);
                let rhs_val = self.inner().get_const(rhs);
                self.inner_mut().set_register(lhs, lhs_val^rhs_val);
            }
            Response::BinaryOp(bytecode::BinaryOp::Or(lhs, rhs)) => {
                let lhs_val = self.inner().get(lhs);
                let rhs_val = self.inner().get_const(rhs);
                self.inner_mut().set_register(lhs, lhs_val|rhs_val);
            }
            Response::BinaryOp(bytecode::BinaryOp::MoveAdd(lhs, rhs)) => {
                let lhs_val = self.inner().get(lhs);
                let rhs_val = self.inner().get_const(rhs);
                let val = lhs_val.wrapping_add(rhs_val);
                let step = Direction::from(val);
                self.move_step(step, &world);
            }
            Response::BinaryOp(bytecode::BinaryOp::MoveSub(lhs, rhs)) => {
                let lhs_val = self.inner().get(lhs);
                let rhs_val = self.inner().get_const(rhs);
                let val = lhs_val.wrapping_sub(rhs_val);
                let step = Direction::from(val);
                self.move_step(step, &world);
            }
            Response::BinaryOp(bytecode::BinaryOp::MoveMul(lhs, rhs)) => {
                let lhs_val = self.inner().get(lhs);
                let rhs_val = self.inner().get_const(rhs);
                let val = lhs_val.wrapping_mul(rhs_val);
                let step = Direction::from(val);
                self.move_step(step, &world);
            }
            Response::BinaryOp(bytecode::BinaryOp::MoveDiv(lhs, rhs)) => {
                let lhs_val = self.inner().get(lhs);
                let rhs_val = self.inner().get_const(rhs);
                let val = if rhs_val == 0 {
                    u64::MAX
                } else {
                    lhs_val.wrapping_sub(rhs_val)
                };
                let step = Direction::from(val);
                self.move_step(step, &world);
            }
            Response::BinaryOp(bytecode::BinaryOp::MoveAnd(lhs, rhs)) => {
                let lhs_val = self.inner().get(lhs);
                let rhs_val = self.inner().get_const(rhs);
                let val = lhs_val&rhs_val;
                let step = Direction::from(val);
                self.move_step(step, &world);
            }
            Response::BinaryOp(bytecode::BinaryOp::MoveXor(lhs, rhs)) => {
                let lhs_val = self.inner().get(lhs);
                let rhs_val = self.inner().get_const(rhs);
                let val = lhs_val^rhs_val;
                let step = Direction::from(val);
                self.move_step(step, &world);
            }
            Response::BinaryOp(bytecode::BinaryOp::MoveOr(lhs, rhs)) => {
                let lhs_val = self.inner().get(lhs);
                let rhs_val = self.inner().get_const(rhs);
                let val = lhs_val|rhs_val;
                let step = Direction::from(val);
                self.move_step(step, &world);
            }
            Response::BinaryOp(bytecode::BinaryOp::Mov(lhs, rhs)) => {
                let rhs_val = self.inner().get_const(rhs);
                self.inner_mut().set_register(lhs, rhs_val);
            }
            Response::BinaryOp(bytecode::BinaryOp::Xchg(lhs, rhs)) => {
                let lhs_val = self.inner().get(lhs);
                let rhs_val = self.inner().get(rhs);
                self.inner_mut().set_register(lhs, rhs_val);
                self.inner_mut().set_register(rhs, lhs_val);
            }
            Response::Call(reg) => {
                let get = self.inner().get_const(reg) as usize;
                
                if !world.functions.is_empty() {
                    world.functions[get%world.functions.len()](&self, world);
                }
            }
            Response::Move(reg) => {
                let get = self.inner().get_const(reg);
                let step = Direction::from(get);
                self.move_step(step, &world);
            }
            Response::Jmp(reg) => {
                let reg0 = unsafe { self.inner().registers[0].long };
                let reg1 = unsafe { self.inner().registers[1].long };
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
                    let rip = self.inner().rip as isize;
                    let jmp_loc = ((jmp as isize + rip)%len) as usize;
                    self.inner_mut().rip = jmp_loc;
                }
            }
            Response::Nop => {}
        }
    }
    pub fn x(&self) -> u32 {
        self.inner().pos[0]
    }
    pub fn y(&self) -> u32 {
        self.inner().pos[1]
    }
    pub fn next_rip(&self) {
        if self.inner().rip < self.code.len() {
            self.inner_mut().rip += 1;
        } else {
            self.inner_mut().rip = 0;
        }
    }
    pub fn execute_next(self: Arc<Self>, world: &World) {
        let next = self.next().unwrap_or_else(||{self.next().unwrap()});
        if self.handle_event(next.event, &world) {
            self.handle_response(next.response, world);
        }
    }
    pub fn inner(&self) -> &GPCAEntityInternal {
        unsafe { self.internal.get().as_ref().unwrap() }
    }
    pub fn inner_mut(&self) -> &mut GPCAEntityInternal {
        unsafe { self.internal.get().as_mut().unwrap() }
    }
    pub fn decrement_energy(&self) {
        self.inner_mut().decrement_energy();
    }
    pub fn set_energy(&self, energy: u32) {
        self.inner_mut().set_energy(energy);
    }
    pub fn get_energy(&self) -> u32 {
        self.inner().get_energy()
    }
    pub fn next(self: &Arc<Self>) -> Option<EventResponse> {
        let next = self.parse();
        self.next_rip();
        next
    }
}