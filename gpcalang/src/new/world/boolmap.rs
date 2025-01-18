use affogato::spatial::morton::MortonU64;

pub struct BooleanMap {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) bools: Vec<u8>,
}

impl BooleanMap {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height, bools: vec![0; ((width*height).div_ceil(u8::BITS)) as usize] }
    }
    /// if x or y exceed width and height, it will return that true, saying that the space is
    /// occupied
    pub fn get(&self, x: u32, y: u32) -> bool {
        // assert!(x < self.width && y < self.height, "x and y can not exceed width and height respectively");
        if x >= self.width || y >= self.height {
            return true;
        }
        let idx = (x+(y*self.width)) as usize;
        let lower_idx = idx&0x7;
        let pos = idx/(u8::BITS as usize);
        let val = self.bools[pos];
        let is_active = val&(1<<lower_idx) != 0;
        is_active
    }
    pub fn set(&mut self, x: u32, y: u32, active: bool) {
        assert!(x < self.width && y < self.height, "x and y can not exceed width and height respectively");
        let idx = (x+(y*self.width)) as usize;
        let lower_idx = idx&0x7;
        let pos = idx/(u8::BITS as usize);
        let val = &mut self.bools[pos];
        // Reset bit before placing 
        *val &= !(1<<lower_idx);
        let bitmask = (active as u8)<<(lower_idx as u8);
        *val |= (bitmask);
    }
    pub fn surrounding_square_count(&self, x: u32, y: u32) -> usize {
        let mut count = 0;
        for y in (y.checked_sub(1).unwrap_or_default())..((y+1).clamp(0, self.height)) {
            for x in (x.checked_sub(1).unwrap_or_default())..((x+1).clamp(0, self.width)) {
                count += self.get(x, y) as usize;
            }
        }
        count
    }
}
/// 0  0  = 0
/// 0  1  = 1
/// 1  0  = 0
/// 1  1  = 1
pub fn place(prev: &mut u8, bitmask: u8) {
    let og = *prev;
    *prev ^= bitmask;
    println!("STEP 1: {prev:b}");
    *prev ^= !og;
    println!("STEP 2: {prev:b}");
    *prev = !*prev;
    println!("STEP 3: {prev:b}");
}