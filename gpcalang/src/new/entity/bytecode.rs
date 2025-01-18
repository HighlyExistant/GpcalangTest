#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Register {
    LongRegister0,
    LongRegister1,
    ByteRegister0_0,
    ByteRegister0_1,
    ByteRegister0_2,
    ByteRegister0_3,
    ByteRegister1_0,
    ByteRegister1_1,
    ByteRegister1_2,
    ByteRegister1_3,
}
impl From<u8> for Register {
    fn from(value: u8) -> Self {
        match value {
            0b0 => Self::ByteRegister0_0,
            0b1 => Self::ByteRegister0_1,
            0b10 => Self::ByteRegister0_2,
            0b11 => Self::ByteRegister0_3,
            0b100 => Self::ByteRegister1_0,
            0b101 => Self::ByteRegister1_1,
            0b110 => Self::ByteRegister1_2,
            0b111 => Self::ByteRegister1_3,
            _ => panic!()
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RegConst {
    Register(Register),
    Constant(u8),
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Jump {
    Unconditional(i8),
    Reg0Eq(i8),
    Reg0Neq(i8),
    Reg0Greater(i8),
    Reg0Lesser(i8),
    Reg0GreaterEq(i8),
    Reg0LesserEq(i8),
    Reg1Eq(i8),
    Reg1Neq(i8),
    Reg1Greater(i8),
    Reg1Lesser(i8),
    Reg1GreaterEq(i8),
    Reg1LesserEq(i8),
}
// Dst <- Src
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BinaryOp {
    Add(Register, RegConst),
    Sub(Register, RegConst),
    Mul(Register, RegConst),
    Div(Register, RegConst),
    Xor(Register, RegConst),
    And(Register, RegConst),
    Or(Register, RegConst),
    Mov(Register, RegConst),
    Xchg(Register, Register),
    MoveAdd(Register, RegConst),
    MoveSub(Register, RegConst),
    MoveMul(Register, RegConst),
    MoveDiv(Register, RegConst),
    MoveXor(Register, RegConst),
    MoveAnd(Register, RegConst),
    MoveOr(Register, RegConst),
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Response {
    Move(RegConst),
    Jmp(Jump),
    BinaryOp(BinaryOp),
    Call(RegConst),
    Nop
}
fn regbyte_lhs_rhs_ext(ext: u8) -> (Register, Register) {
    if (ext&0b10000000) != 0 { // Is 64 bits
        if (ext&0b01000000) != 0 {
            (Register::LongRegister1, Register::LongRegister0)                
        } else {
            (Register::LongRegister0, Register::LongRegister1)                
        }
    } else { // Is 8 bits
        let lhs = (ext>>3)&0x7;
        let rhs = (ext)&0x7;
        (Register::from(lhs), Register::from(rhs))
    }
}
impl Response {
    pub fn is_move_step(&self) -> bool {
        if let Response::Move(_) = self {
            true
        } else {
            if let Response::BinaryOp(BinaryOp::MoveAdd(_, _)) | 
                   Response::BinaryOp(BinaryOp::MoveSub(_, _)) |
                   Response::BinaryOp(BinaryOp::MoveMul(_, _)) |
                   Response::BinaryOp(BinaryOp::MoveDiv(_, _)) |
                   Response::BinaryOp(BinaryOp::MoveAnd(_, _)) |
                   Response::BinaryOp(BinaryOp::MoveXor(_, _)) |
                   Response::BinaryOp(BinaryOp::MoveOr(_, _)) = self {
                true
            } else {
                false
            }
        }
    }
    fn top_layer(op: u8, ext: u8) -> Self {
        match op {
            0b0 =>          Self::Move(RegConst::Register(Register::LongRegister0)),
            0b1 =>          Self::Move(RegConst::Register(Register::LongRegister1)),
            0b10 =>         Self::Call(RegConst::Register(Register::LongRegister0)),
            0b11 =>         Self::Call(RegConst::Register(Register::LongRegister1)),
            0b01000 =>      Self::Jmp(Jump::Reg0Eq(ext as i8)),
            0b01001 =>      Self::Jmp(Jump::Reg0Neq(ext as i8)),
            0b01010 =>      Self::Jmp(Jump::Reg0Greater(ext as i8)),
            0b01011 =>      Self::Jmp(Jump::Reg0Lesser(ext as i8)),
            0b01100 =>      Self::Jmp(Jump::Reg0GreaterEq(ext as i8)),
            0b01101 =>      Self::Jmp(Jump::Reg0LesserEq(ext as i8)),
            0b1110 =>       Self::Jmp(Jump::Unconditional(ext as i8)),
            0b1111 =>       Self::Move(RegConst::Constant(ext)),
            0b100000 =>     Self::Jmp(Jump::Reg1Eq(ext as i8)),
            0b100001 =>     Self::Jmp(Jump::Reg1Neq(ext as i8)),
            0b100010 =>     Self::Jmp(Jump::Reg1Greater(ext as i8)),
            0b100011 =>     Self::Jmp(Jump::Reg1Lesser(ext as i8)),
            0b100100 =>     Self::Jmp(Jump::Reg1GreaterEq(ext as i8)),
            0b100101 =>     Self::Jmp(Jump::Reg1LesserEq(ext as i8)),
            0b1000000 =>    Self::Move(RegConst::Register(Register::LongRegister0)),
            0b1000001 =>    Self::Move(RegConst::Register(Register::LongRegister1)),
            0b1100000 =>    Self::Move(RegConst::Register(Register::LongRegister0)),
            0b1100001 =>    Self::Move(RegConst::Register(Register::LongRegister1)),
            0b10000000 =>   Self::Move(RegConst::Register(Register::LongRegister0)),
            0b10000001 =>   Self::Move(RegConst::Register(Register::LongRegister1)),
            0b11000000 =>   Self::Move(RegConst::Register(Register::LongRegister0)),
            0b11000001 =>   Self::Move(RegConst::Register(Register::LongRegister1)),
            // 0b11100000 =>   Self::Move(RegConst::Register(Register::LongRegister0)),
            // 0b11100001 =>   Self::Move(RegConst::Register(Register::LongRegister1)),
            _ => {
                if (op&0b10000) != 0 {
                    let (lhs, rhs) = regbyte_lhs_rhs_ext(ext);
                    match op {
                        0b10000 => Self::BinaryOp(BinaryOp::Add(lhs, RegConst::Register(rhs))),
                        0b10001 => Self::BinaryOp(BinaryOp::Sub(lhs, RegConst::Register(rhs))),
                        0b10010 => Self::BinaryOp(BinaryOp::Mul(lhs, RegConst::Register(rhs))),
                        0b10011 => Self::BinaryOp(BinaryOp::Div(lhs, RegConst::Register(rhs))),
                        0b10100 => Self::BinaryOp(BinaryOp::Xor(lhs, RegConst::Register(rhs))),
                        0b10101 => Self::BinaryOp(BinaryOp::And(lhs, RegConst::Register(rhs))),
                        0b10110 => Self::BinaryOp(BinaryOp::Or(lhs, RegConst::Register(rhs))),
                        0b10111 => Self::BinaryOp(BinaryOp::Mov(lhs, RegConst::Register(rhs))),
                        0b11111 => Self::BinaryOp(BinaryOp::Xchg(lhs, rhs)),
                        0b110000 => Self::BinaryOp(BinaryOp::MoveAdd(lhs, RegConst::Register(rhs))),
                        0b110001 => Self::BinaryOp(BinaryOp::MoveSub(lhs, RegConst::Register(rhs))),
                        0b110010 => Self::BinaryOp(BinaryOp::MoveMul(lhs, RegConst::Register(rhs))),
                        0b110011 => Self::BinaryOp(BinaryOp::MoveDiv(lhs, RegConst::Register(rhs))),
                        0b110100 => Self::BinaryOp(BinaryOp::MoveXor(lhs, RegConst::Register(rhs))),
                        0b110101 => Self::BinaryOp(BinaryOp::MoveAnd(lhs, RegConst::Register(rhs))),
                        0b110110 => Self::BinaryOp(BinaryOp::MoveOr(lhs, RegConst::Register(rhs))),
                        0b111000 => Self::Move(RegConst::Constant(0)),
                        0b111001 => Self::Move(RegConst::Constant(4)),
                        0b111010 => Self::Move(RegConst::Constant(2)),
                        0b111011 => Self::Move(RegConst::Constant(6)),
                        _ => Response::Nop,
                    }
                } else if (op&0b10000000) != 0 {
                    match op {
                        0b10000000 => Self::Call(RegConst::Register(Register::LongRegister0)),
                        0b10000001 => Self::Call(RegConst::Register(Register::LongRegister1)),
                        0b11111111 => Self::Call(RegConst::Constant(ext)),
                        _ => Response::Nop,
                    }
                } else {
                    Response::Nop
                }
            }
        }
    }
}
impl From<u16> for Response {
    fn from(value: u16) -> Self {
        let op = ((value >> 8)&0xff) as u8;
        let ext = (value&0xff) as u8;
        Response::top_layer(op, ext)
    }
}
/// Events generate boolean values to see whether a respone
/// should be executed or not. Every single Response needs an
/// Event.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Event {
    Unconditional,
    Equal(Register, Register),
    NotEqual(Register, Register),
    Greater(Register, Register),
    Lesser(Register, Register),
    GreaterEqual(Register, Register),
    LesserEqual(Register, Register),
    SurroundingSquaresEqual(RegConst),
    SurroundingSquaresNotEqual(RegConst),
    SurroundingSquaresGreater(RegConst),
    SurroundingSquaresLesser(RegConst),
    SurroundingSquaresGreaterEqual(RegConst),
    SurroundingSquaresLesserEqual(RegConst),
}

impl Event {
    fn top_layer(op: u8, ext: u8) -> Self {
        let (lhs, rhs) = regbyte_lhs_rhs_ext(ext);
        match op {
            0b0 => Self::Equal(lhs, rhs),
            0b1 => Self::NotEqual(lhs, rhs),
            0b10 => Self::Greater(lhs, rhs),
            0b11 => Self::Lesser(lhs, rhs),
            0b100 => Self::GreaterEqual(lhs, rhs),
            0b101 => Self::LesserEqual(lhs, rhs),
            0b1000 => Self::SurroundingSquaresEqual(RegConst::Constant(ext)),
            0b1001 => Self::SurroundingSquaresNotEqual(RegConst::Constant(ext)),
            0b1010 => Self::SurroundingSquaresGreater(RegConst::Constant(ext)),
            0b1011 => Self::SurroundingSquaresLesser(RegConst::Constant(ext)),
            0b1100 => Self::SurroundingSquaresGreaterEqual(RegConst::Constant(ext)),
            0b1101 => Self::SurroundingSquaresLesserEqual(RegConst::Constant(ext)),
            0b10000 => Self::SurroundingSquaresEqual(RegConst::Register(lhs)),
            0b10001 => Self::SurroundingSquaresNotEqual(RegConst::Register(lhs)),
            0b10010 => Self::SurroundingSquaresGreater(RegConst::Register(lhs)),
            0b10011 => Self::SurroundingSquaresLesser(RegConst::Register(lhs)),
            0b10100 => Self::SurroundingSquaresGreaterEqual(RegConst::Register(lhs)),
            0b10101 => Self::SurroundingSquaresLesserEqual(RegConst::Register(lhs)),
            _ => Self::Unconditional
        }
    }
}

impl From<u16> for Event {
    fn from(value: u16) -> Self {
        let op = ((value >> 8)&0xff) as u8;
        let ext = (value&0xff) as u8;
        Event::top_layer(op, ext)
    }
}