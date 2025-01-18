use std::{fs::File, io::Write, num::NonZeroIsize, os::raw::c_void, sync::Arc, time::Instant};

use affogato::{geometry::Rect, linear::{FMat3, FVec2, FVec3, FVec4, Matrix3, SquareMatrix, Transformation2D}};
use frappe::{collection::{alloc::{allocator::{freelist::FreeListAllocatorInternal, standard::StandardMemoryAllocator}, descriptor::{DescriptorSetAllocator, StandardDescriptorSetAllocatorCreateInfo}, set_global_descriptor_allocator, set_global_gpu_allocator}, collection::HostVec, data::GpuGuard}, core::{ash::vk::{self, CullModeFlags}, commands::{CommandBufferBeginInfo, CommandPool, CommandPoolAllocation}, device::{queue::Queue, LogicalDevice, LogicalDeviceBuilder}, instance::InstanceBuilder, khr::surface::Surface, pipeline::graphics::{FrontFace, LineTopology, RasterizationMode, TriangleTopology}, Version}, obj::Mesh, physics::{collision::{Collision, SeparatingAxisTheorem2D}, kinermatics::Chain}, visual::{raster::{GraphicsRenderer, LinePipelineVertex, PlainPipeline, Raster2DPipelinePushConstant, RenderingSwapchain, UVPipeline, UVPipelineState, UVPipelineUniform, UVVertex}, RecreateableRenderer, Renderer}, TimeCycle};
use winit::{application::ApplicationHandler, event::{KeyEvent, WindowEvent}, event_loop::{ActiveEventLoop, ControlFlow, EventLoop}, keyboard::Key, raw_window_handle::{DisplayHandle, HasWindowHandle, RawWindowHandle, Win32WindowHandle, WindowHandle}, window::{Window, WindowId}};

use crate::gpca::GPCAData;
pub fn standard() -> (Arc<LogicalDevice>, impl ExactSizeIterator<Item = Arc<Queue>>) {
    let instance = InstanceBuilder::new()
        .set_version(Version::new(1, 3, 0))
        .validation_layers()
        .required_windowing_extensions()
        .get_physical_device_properties2()
        .device_group_creation_extension()
        .build().unwrap();
        let physical_device = instance.enumerate_physical_devices().unwrap().next().unwrap();
        let graphics_queue_family_index = physical_device.enumerate_queue_family_properties()
            .iter()
            .enumerate()
            .position(|(_queue_family_index, queue_family_properties)|{
                queue_family_properties.queue_flags.contains(vk::QueueFlags::GRAPHICS|vk::QueueFlags::COMPUTE|vk::QueueFlags::TRANSFER)
            }).unwrap();
        let compute_queue_family_index = physical_device.enumerate_queue_family_properties()
            .iter()
            .enumerate()
            .position(|(_queue_family_index, queue_family_properties)|{
                queue_family_properties.queue_flags.contains(vk::QueueFlags::COMPUTE)
            }).unwrap();
        
        let (device, mut queues) = LogicalDeviceBuilder::new()
            .add_queue(vk::DeviceQueueCreateFlags::empty(), graphics_queue_family_index as u32, 1, 0, &1.0)
            .add_queue(vk::DeviceQueueCreateFlags::empty(), compute_queue_family_index as u32, 1, 0, &1.0)
            .fill_mode_non_solid()
            .device_group()
            .enable_float64()
            .runtime_descriptor_array()
            .enable_int64()
            .wide_lines()
            .enable_anisotropic_sampling()
            .descriptor_indexing(
                true, 
                true, 
                true, 
                true, 
                true, 
                true, 
                true, 
                true, 
                true)
            .subgroup_ballot()
            .enable_swapchain_extensions()
            .enable_buffer_addressing()
            .build(physical_device.clone()).unwrap();
    (device, queues)
}
pub struct AppState {
    window: Arc<Window>,
    surface: Arc<Surface>,
    renderer: RenderingSwapchain,
    graphics: PlainPipeline,
    graphics_uv: UVPipeline,
    uv_state: UVPipelineState,
}
impl AppState {
    pub fn window(&self) -> &Arc<Window> {
        &self.window
    }
}
pub struct App {
    state: Option<AppState>,
    device: Arc<LogicalDevice>,
    general_purpose: Arc<Queue>,
    command_pool: Arc<CommandPool>,
    command_buffers: Vec<CommandPoolAllocation>,
    descriptor_allocator: Arc<DescriptorSetAllocator>,
    mesh: Mesh<LinePipelineVertex>,
    mesh_uv: Mesh<UVVertex>,
    dt: f64,
    time: f64,
    gpca: GPCAData,
    fps60: TimeCycle,
    frames_passed: usize,
    log: File,
    allocator: Arc<StandardMemoryAllocator<FreeListAllocatorInternal>>,
}
impl App {
    pub fn new(queue: Arc<Queue>, command_pool: Arc<CommandPool>, descriptor_allocator: Arc<DescriptorSetAllocator>, allocator: Arc<StandardMemoryAllocator<FreeListAllocatorInternal>>) -> Self {
        let command_buffers = unsafe { command_pool.allocate_command_buffers(vk::CommandBufferLevel::PRIMARY, 2).unwrap().collect::<Vec<_>>() };
        let square = Rect::new(FVec2::new(-1.0, -1.0), FVec2::new(1.0, 1.0));
        let uv_square = Rect::from_lengths(1.0, 1.0);
        let vertices = square.get_vertices().iter().map(|v|{
            LinePipelineVertex { pos: (*v).into(), color: FVec4::rgba_from_u32(0xffffffff) }
        }).collect::<Vec<_>>();
        let vertices_uv = square.get_vertices().iter().zip(uv_square.get_vertices()).map(|(v, uv)|{
            UVVertex { pos: (*v).into(), uv: uv }
        }).collect::<Vec<_>>();
        // let vertices_uv = square.get_vertices().iter().map(|v|{
        //     UVVertex { pos: (*v).into(), uv: (*v).into() }
        // }).collect::<Vec<_>>();
        let mesh = Mesh::from_slice(allocator.clone(), vk::BufferUsageFlags::VERTEX_BUFFER, vk::BufferUsageFlags::INDEX_BUFFER, &vertices, &square.get_tri_indices());
        let mut log = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open("log.txt").unwrap();
        let gpca = GPCAData::new(&allocator, 1024, 4096, 40, 256, 256, &mut log);
        let mesh_uv = Mesh::from_slice(allocator.clone(), vk::BufferUsageFlags::VERTEX_BUFFER, vk::BufferUsageFlags::INDEX_BUFFER, &vertices_uv, &square.get_tri_indices());
        
        Self { 
            state: None, 
            device: queue.device(), 
            general_purpose: queue.clone(), 
            command_pool, 
            command_buffers, 
            descriptor_allocator, 
            allocator,
            mesh,
            mesh_uv,
            gpca,
            dt: 0.0,
            time: 0.0,
            frames_passed: 0,
            log,
            fps60: TimeCycle::new(1.0/1000.0),
        }
    }
}
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_none() {
            let window = Arc::new(event_loop.create_window(Window::default_attributes()).unwrap());

            let handle = window.window_handle().unwrap();
            let raw = handle.as_raw();
            let (hwnd, hinstance) = if let RawWindowHandle::Win32(handle) = raw {
                (isize::from(handle.hwnd) as *const isize as *mut c_void, isize::from(handle.hinstance.unwrap()) as *const isize as *mut c_void)
            } else {
                panic!("FUCK ME")
            };
            let mut win_handle = Win32WindowHandle::new(NonZeroIsize::new(hwnd as isize).unwrap());
            win_handle.hinstance = Some(NonZeroIsize::new(hinstance as isize).unwrap());
            let surface = Arc::new(unsafe { 
                Surface::from_handles(
                self.device.instance(), 
                DisplayHandle::windows(),
                WindowHandle::borrow_raw(RawWindowHandle::Win32(win_handle))
                ).unwrap() 
            });
            let renderer = RenderingSwapchain::new(self.device.clone(), self.allocator.clone(), window.clone(), surface.clone(), 2).unwrap();
            
            let graphics = PlainPipeline::<LinePipelineVertex>::new(self.device.clone(), self.allocator.clone(), self.descriptor_allocator.clone(), renderer.render_pass(), RasterizationMode::triangle(TriangleTopology::List, CullModeFlags::NONE, FrontFace::CounterClockwise), 2).unwrap();
            let graphics_uv = UVPipeline::new(self.device.clone(), self.allocator.clone(), self.descriptor_allocator.clone(), renderer.render_pass(), RasterizationMode::triangle(TriangleTopology::List, CullModeFlags::NONE, FrontFace::CounterClockwise)).unwrap();
            let uv_state = graphics_uv.create_state(self.gpca.image.clone(), UVPipelineUniform { range: 0.0 });
            self.state = Some(AppState { 
                window, 
                surface,
                renderer,
                graphics,
                graphics_uv,
                uv_state,
            });
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let state = self.state.as_mut().unwrap();
        let then = Instant::now();
        self.fps60.then(|dt|{
            self.gpca.step();
            if self.frames_passed % 250 == 0 {
                self.log.write(format!("Frame {}, EntityCount: {}\n", self.frames_passed, self.gpca.world.get_entites().len()).as_bytes()).unwrap();
            }
            self.frames_passed += 1;
        });
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            },
            WindowEvent::CursorMoved { device_id, position } => {
                let window_size = state.window.inner_size();
                frappe::input::modify_mouse_position_vulkan(position.x, position.y, window_size.width, window_size.height);
            }
            WindowEvent::RedrawRequested => {
                state.renderer.next_image().unwrap();
                let cmd = &self.command_buffers[state.renderer.current_frame()];
                cmd.begin(CommandBufferBeginInfo::default()).unwrap();
                self.gpca.flush(cmd);
                state.renderer.render(cmd, |cmd, renderer|{
                    // state.graphics.bind_pipeline(cmd);
                    // cmd.bind_index_buffers(self.mesh.indices().handle(), 0, vk::IndexType::UINT32);
                    // cmd.bind_vertex_buffers(0, &[self.mesh.vertices().handle()], &[0]);
                    // state.graphics.push_constant(cmd, Raster2DPipelinePushConstant { transform: FMat3::identity() });
                    // cmd.draw_indexed(self.mesh.index_count() as u32, 1, 0, 0, 0);
                    state.graphics_uv.bind_pipeline(cmd, &state.uv_state);
                    state.graphics_uv.bind_vertices(cmd, &[self.mesh_uv.vertices().clone()], self.mesh_uv.indices().clone());
                    state.graphics_uv.push_constant(cmd, &Raster2DPipelinePushConstant { transform: Matrix3::identity() });
                    
                    state.graphics_uv.draw(cmd, self.mesh_uv.index_count() as u32, 1, 0, 0, 0);
                });
                
                cmd.end().unwrap();
                state.renderer.submit(self.general_purpose.clone(), cmd);

                state.window.request_redraw();
            }
            WindowEvent::KeyboardInput { device_id, event, is_synthetic } => {
                frappe::input::modify_keyboard_input(&event);
            }
            WindowEvent::Resized(size) => {
                state.renderer.recreate_renderer(size.into());
            }
            _ => (),
        }
        // println!("fps {}", 1.0/self.dt);
        self.dt = then.elapsed().as_secs_f64();
        self.time += self.dt;
        self.fps60.step(self.dt);
    }
    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        self.device.wait().unwrap();
    }
}