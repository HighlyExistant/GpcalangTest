use std::{fs::File, io::Write, sync::Arc};

use affogato::linear::{FVec4, UI8Vec4};
use frappe::collection::{alloc::{allocator::{freelist::FreeListAllocatorInternal, standard::StandardMemoryAllocator}, AllocationCreateInfo, MemoryTypeFilter}, data::{ImageBuilder, ImageWriter, ViewableImage, ViewableImageBuilder}};
use frappe_core::{ash::vk, commands::CommandPoolAllocation};
use gpcalang::{GPCAEntity, World};
use rand::{Rng, RngCore};

pub struct GPCAData {
    pub world: World,
    pub image: Arc<ViewableImage>,
    writer: ImageWriter,
}

fn maleable_op2(entity: &Arc<GPCAEntity>, world: &World) {
    let sq = world.surrounding_square_count(entity.x(), entity.y());
    if world.get_entity_at_direction(entity.inner(), gpcalang::Direction::Bottom).is_some() && 
    world.get_entity_at_direction(entity.inner(), gpcalang::Direction::Top).is_none() && (entity.y()+1) != world.height() {
        world.push_entity(GPCAEntity::new(entity.x(), entity.y()+1, 0, world.pseudo().gen_range(0..u64::MAX), world.pseudo().gen_range(0..u64::MAX), 4096, entity.color, entity.code.clone()));
    }
}
fn maleable_breed2(entity: &Arc<GPCAEntity>, world: &World) {
    let sq = world.surrounding_square_count(entity.x(), entity.y());
    if world.get_entity_at_direction(entity.inner(), gpcalang::Direction::Top).is_some() && 
    world.get_entity_at_direction(entity.inner(), gpcalang::Direction::Bottom).is_none() && (entity.y()) != 0 {
        let entity_b = world.get_entity_at_direction(entity.inner(), gpcalang::Direction::Top).unwrap();
        let new_code = {
            let len = entity.code.len().max(entity_b.code.len());
            let mut new_code = Vec::with_capacity(len);
            let mut interleave = false;
            for i in 0..len {
                if interleave {
                    new_code.push(entity_b.code[i%entity_b.code.len()]);
                    interleave = false;
                } else {
                    new_code.push(entity.code[i%entity.code.len()]);
                    interleave = true;
                }
            }
            new_code
        };
        const COLOR: [u32; 7] = [
            0xff0000ff,
            0x00ff00ff,
            0x0000ffff,
            0xffff00ff,
            0x00ffffff,
            0xff00ffff,
            0xffffffff,
        ];
        static mut IDX: usize = 0;

        let mut color = affogato::lerp(FVec4::rgba_from_u32(entity.color), FVec4::rgba_from_u32(entity_b.color), 0.50).into_rgba8();
        if color < 0x55555555 {
            if color == 0x00000000 {
                color = COLOR[unsafe { IDX }];
                unsafe { IDX += 1 };
                if unsafe { IDX } >= COLOR.len() {
                    unsafe { IDX = 0 };
                }
            } else {
                while color < 0x55555555 {
                    color *= 2;
                }
            }
        }
        world.create_entity(GPCAEntity::new(entity.x(), entity.y()-1, 0, world.pseudo().gen_range(0..u64::MAX), world.pseudo().gen_range(0..u64::MAX), 4097, color, new_code));
        println!("ENTITY COUNT: {}", world.get_entites().len());
    }
}
fn maleable_op(entity: &Arc<GPCAEntity>, world: &World) {
    let sq = world.surrounding_square_count(entity.x(), entity.y());
    if world.get_entity_at_direction(entity.inner(), gpcalang::Direction::TopLeft).is_some() && 
    world.get_entity_at_direction(entity.inner(), gpcalang::Direction::BottomRight).is_none() && (entity.x()+1) < world.height() && (entity.y()) != 0 {
        world.push_entity(GPCAEntity::new(entity.x()+1, entity.y()-1, 0, world.pseudo().gen_range(0..u64::MAX), world.pseudo().gen_range(0..u64::MAX), 4096, entity.color, entity.code.clone()));
    }
}
fn maleable_breed(entity: &Arc<GPCAEntity>, world: &World) {
    let sq = world.surrounding_square_count(entity.x(), entity.y());
    if world.get_entity_at_direction(entity.inner(), gpcalang::Direction::BottomRight).is_some() && 
    world.get_entity_at_direction(entity.inner(), gpcalang::Direction::TopLeft).is_none() && (entity.y()+1) < world.height() && (entity.x()) != 0 {
        let entity_b = world.get_entity_at_direction(entity.inner(), gpcalang::Direction::BottomRight).unwrap();
        let new_code = {
            let len = entity.code.len().max(entity_b.code.len());
            let mut new_code = Vec::with_capacity(len);
            let mut interleave = false;
            for i in 0..len {
                if interleave {
                    new_code.push(entity_b.code[i%entity_b.code.len()]);
                    interleave = false;
                } else {
                    new_code.push(entity.code[i%entity.code.len()]);
                    interleave = true;
                }
            }
            new_code
        };
        const COLOR: [u32; 7] = [
            0xff0000ff,
            0x00ff00ff,
            0x0000ffff,
            0xffff00ff,
            0x00ffffff,
            0xff00ffff,
            0xffffffff,
        ];
        static mut IDX: usize = 0;

        let mut color = affogato::lerp(FVec4::rgba_from_u32(entity.color), FVec4::rgba_from_u32(entity_b.color), 0.50).into_rgba8();
        if color < 0x55555555 {
            if color == 0x00000000 {
                color = COLOR[unsafe { IDX }];
                unsafe { IDX += 1 };
                if unsafe { IDX } >= COLOR.len() {
                    unsafe { IDX = 0 };
                }
            } else {
                while color < 0x55555555 {
                    color *= 2;
                }
            }
        }
        world.create_entity(GPCAEntity::new(entity.x()-1, entity.y()+1, 0, world.pseudo().gen_range(0..u64::MAX), world.pseudo().gen_range(0..u64::MAX), 4097, color, new_code));
        println!("ENTITY COUNT: {}", world.get_entites().len());
    }
}
fn eat3(entity: &Arc<GPCAEntity>, world: &World) {
    let sq = world.surrounding_square_count(entity.x(), entity.y());
    if world.get_entity_at_direction(entity.inner(), gpcalang::Direction::Bottom).is_none() && 
    world.get_entity_at_direction(entity.inner(), gpcalang::Direction::Top).is_some() && (entity.y()+1) < world.height() && (entity.x()) != 0 {
        let entity_eat = world.get_entity_at_direction(entity.inner(), gpcalang::Direction::Top).unwrap();
        world.remove(entity_eat.x(), entity_eat.y());
        let prev_energy = entity.get_energy();
        entity.set_energy((prev_energy+entity_eat.get_energy().div_ceil(4)).min(4097));
        println!("Ate energy prev_energy {prev_energy} current {}", entity.get_energy());
        // world.push_entity(GPCAEntity::new(entity.x()-1, entity.y()+1, rand::thread_rng().gen_range(0..u64::MAX), rand::thread_rng().gen_range(0..u64::MAX), 10, entity.color, entity.code.clone()));
    }
}
fn eat3_op(entity: &Arc<GPCAEntity>, world: &World) {
    let sq = world.surrounding_square_count(entity.x(), entity.y());
    if world.get_entity_at_direction(entity.inner(), gpcalang::Direction::Bottom).is_some() && 
    world.get_entity_at_direction(entity.inner(), gpcalang::Direction::Top).is_none() && (entity.x()+1) < world.height() && (entity.y()) != 0 {
        let entity_eat = world.get_entity_at_direction(entity.inner(), gpcalang::Direction::Bottom).unwrap();
        world.remove(entity_eat.x(), entity_eat.y());
        let prev_energy = entity.get_energy();
        entity.set_energy((prev_energy+entity_eat.get_energy().div_ceil(4)).min(4097));
        println!("Ate energy prev_energy {prev_energy} current {}", entity.get_energy());
        // world.push_entity(GPCAEntity::new(entity.x()-1, entity.y()+1, rand::thread_rng().gen_range(0..u64::MAX), rand::thread_rng().gen_range(0..u64::MAX), 10, entity.color, entity.code.clone()));
    }
}
impl GPCAData {
    pub fn new(allocator: &Arc<StandardMemoryAllocator<FreeListAllocatorInternal>>, entity_count: usize, energy: u32, code_len: u32, width: u32, height: u32, log: &mut File) -> Self {
        let image = Arc::new(ViewableImageBuilder::new()
            .usage(vk::ImageUsageFlags::SAMPLED|vk::ImageUsageFlags::COLOR_ATTACHMENT|vk::ImageUsageFlags::TRANSFER_DST|vk::ImageUsageFlags::TRANSFER_SRC)
            .image_type(vk::ImageType::TYPE_2D)
            .image_view_type(vk::ImageViewType::TYPE_2D)
            .set_extent(width, height, 1)
            .set_format(vk::Format::R8G8B8A8_SRGB)
            .set_subresource_range_access_mask(vk::ImageAspectFlags::COLOR)
            .build(allocator.clone(), AllocationCreateInfo {
                filter: MemoryTypeFilter::PREFER_DEVICE,
                preference: frappe::collection::alloc::MemoryAllocatePreference::Unknown
            }).unwrap());
        let writer = ImageWriter::new(allocator.clone(), image.image(), vk::ImageLayout::UNDEFINED, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL, None);
        // let world = World::new(vec![eat3, maleable_breed, eat3_op, maleable_op, eat3], entity_count, width, height, true, 1.0/1000.0, Some(0xabdf1327932123ffabdf1327932123ff));
        let (use_energy, mutation_chance, seed) = (true, 1.0/1000.0, Some(0xabdf1327932123ffabdf1327932123ff));
        let world = World::new(vec![eat3, maleable_breed, eat3_op, maleable_op, eat3], entity_count, width, height, use_energy, mutation_chance, seed);
        if use_energy {
            log.write(format!("Seed {} Mutation {} UseEnergy? {} EnergyCount {} Width {} Height {} EntityCount {}\n", seed.unwrap_or(0xcafef00dd15ea5e5), mutation_chance, use_energy, energy, width, height, entity_count).as_bytes()).unwrap();
        } else {
            log.write(format!("Seed {} Mutation {} UseEnergy? {} Width {} Height {} EntityCount {}\n", seed.unwrap_or(0xcafef00dd15ea5e5), mutation_chance, use_energy, width, height, entity_count).as_bytes()).unwrap();
        }
        
        let mut this = Self { world, image, writer };
        this.create_entities(entity_count, energy, code_len, width, height);
        this
    }
    fn create_entities(&mut self, entity_count: usize, energy: u32, code_len: u32, width: u32, height: u32) {
        // select a random coordinate, if its not occupied in boolean map, place entity else  continue searching.
        for i in 0..entity_count {
            let mut x = 0;
            let mut y = 0;
            while {
                x = self.world.pseudo().gen_range(0..width);
                y = self.world.pseudo().gen_range(0..height);
                self.world.get(x, y)
            } {}
            let code = (0..code_len).into_iter().map(|_|{
                self.world.pseudo().gen_range(0..u32::MAX)
            }).collect::<Vec<_>>();
            let color = self.world.pseudo().gen_range(0x77777777..u32::MAX);
            self.push_entity(GPCAEntity::new(x, y, 0, self.world.pseudo().gen_range(0..u64::MAX), self.world.pseudo().gen_range(0..u64::MAX), energy, color, code), UI8Vec4::rgba_from_u32(color));
        }
    }
    pub fn push_entity(&mut self, entity: GPCAEntity, rgba: UI8Vec4) {
        let x = entity.x() as usize;
        let y = entity.y() as usize;
        self.world.push_entity(entity);
        self.writer.place_pixel(x, y, rgba);
    }
    // pub fn create_entity(&mut self, entity: GPCAEntity, rgba: UI8Vec4) {
    //     let x = entity.x() as usize;
    //     let y = entity.y() as usize;
    //     self.world.create_entity(entity);
    //     self.writer.place_pixel(x, y, rgba);
    // }
    pub fn step(&mut self) {
        self.world.step(
            |entity| {
            self.writer.place_pixel(entity.x() as usize, entity.y() as usize, 0x00000000u32);
        }, |entity|{
            self.writer.place_pixel(entity.x() as usize, entity.y() as usize, entity.color);
        });
    }
    pub fn flush(&self, cmd: &CommandPoolAllocation) {
        self.writer.enable_flush(cmd, self.image.image(), |mut img|{
            img.flush().unwrap();
        });
    }
}