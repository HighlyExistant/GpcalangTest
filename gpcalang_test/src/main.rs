use std::sync::Arc;

use app::standard;
use frappe::{collection::alloc::{allocator::{freelist::FreeListAllocatorInternal, standard::StandardMemoryAllocator}, descriptor::{DescriptorSetAllocator, StandardDescriptorSetAllocatorCreateInfo}, set_global_descriptor_allocator, set_global_gpu_allocator}, TimeCycle};
use frappe_core::{ash::vk::{self}, commands::CommandPool};
use winit::event_loop::{ControlFlow, EventLoop};

mod app;
mod gpca;
fn main() {
    let (device, queues) = standard();
    let (general_purpose_queue, compute_queue) = {
        let mut general_purpose_queue = None;
        let mut compute_queue = None;
        for queue in queues {
            if queue.queue_flags().contains(vk::QueueFlags::GRAPHICS|vk::QueueFlags::COMPUTE|vk::QueueFlags::TRANSFER) {
                general_purpose_queue = Some(queue.clone());
            }
            if queue.queue_flags().contains(vk::QueueFlags::COMPUTE) {
                compute_queue = Some(queue.clone());
            }
        }
        (general_purpose_queue.unwrap(), compute_queue.unwrap())
    };
    let general_purpose_command_pool = Arc::new(CommandPool::new(device.clone(), vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER, general_purpose_queue.family_index()).unwrap());
    let descriptor_allocator = Arc::new(DescriptorSetAllocator::new(device.clone(), StandardDescriptorSetAllocatorCreateInfo {
        set_count: 8,
        update_after_bind: false,
    }));
    
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let allocator = Arc::new(StandardMemoryAllocator::<FreeListAllocatorInternal>::new_default(device.clone()));
    unsafe { set_global_descriptor_allocator(descriptor_allocator.clone()) };
    unsafe { set_global_gpu_allocator(allocator.clone()) };
    // let update_loop = UpdateLoop::new();
    let mut app = app::App::new(general_purpose_queue.clone(), general_purpose_command_pool.clone(), descriptor_allocator.clone(), allocator.clone());
    
    let mut time_cycle = TimeCycle::new(1.0/15.0);
    event_loop.run_app(&mut app).unwrap();
}
