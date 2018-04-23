use hal::{Instance, Adapter, Device, PhysicalDevice, Surface, Backend};
use winit;
use std::any::Any;

pub struct Context<B: Backend> {
    instance: Box<Any + Send + Sync>,
    device: B::Device,
    adapter: Adapter,
    surface: B::Surface,
}

impl<B> Context<B>
where
    B: Backend,
{
    pub fn new(&self, window: &winit::Window) -> Context<B> {
        let instance = B::Instance::create("Gfx HAL", 1);
        let mut surface = instance.create_surface(window);
        let mut adapter = instance.enumarate_adapters().remove(0);

        println!("Adapter: {:?}", adapter.info);

        let (mut device, mut queue_group) = adapter
            .open_with::<_, hal::General>(1, |family| {
                surface.supports_queue_family(family)
            }).unwrap();

        Context {
            instance: instance,
            device: device,
            adapter: adapter,
            surface: surface,
        }
    }
}
