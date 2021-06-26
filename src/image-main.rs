// extern crate image;
// extern crate vulkano;
// extern crate vulkano_shaders;
//
// use std::borrow::Borrow;
// use std::sync::Arc;
//
// use image::{ImageBuffer, Rgba};
// use vulkano::buffer::BufferUsage;
// use vulkano::buffer::CpuAccessibleBuffer;
// use vulkano::command_buffer::{AutoCommandBuffer, AutoCommandBufferBuilder};
// use vulkano::command_buffer::CommandBuffer;
// use vulkano::command_buffer::DynamicState;
// use vulkano::descriptor::descriptor::DescriptorDescSupersetError::DimensionsMismatch;
// use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
// use vulkano::descriptor::pipeline_layout::PipelineLayoutDesc;
// use vulkano::descriptor::PipelineLayoutAbstract;
// use vulkano::device::Device;
// use vulkano::device::DeviceExtensions;
// use vulkano::device::Features;
// use vulkano::format::ClearValue;
// use vulkano::format::Format;
// use vulkano::framebuffer::Framebuffer;
// use vulkano::framebuffer::Subpass;
// use vulkano::image::Dimensions;
// use vulkano::image::StorageImage;
// use vulkano::instance::Instance;
// use vulkano::instance::InstanceExtensions;
// use vulkano::instance::PhysicalDevice;
// use vulkano::pipeline::ComputePipeline;
// use vulkano::pipeline::GraphicsPipeline;
// use vulkano::pipeline::viewport::Viewport;
// use vulkano::sync::GpuFuture;
//
// #[derive(Default, Copy, Clone)]
// struct Vertex {
//     position: [f32; 2],
// }
//
// vulkano::impl_vertex!(Vertex, position);
//
// mod vs {
//     vulkano_shaders::shader! {
//     ty: "vertex",
//     src: "
// #version 450
//
// layout(location = 0) in vec2 position;
//
// void main() {
//     gl_Position = vec4(position, 0.0, 1.0);
// }"
//     }
// }
//
// mod fs {
//     vulkano_shaders::shader! {
//     ty: "fragment",
//     src: "
// #version 450
//
// layout(location = 0) out vec4 f_color;
//
// void main() {
//     f_color = vec4(1.0, 0.0, 0.0, 1.0);
// }"
//     }
// }
//
// fn main() {
//     let instance = Instance::new(None, &InstanceExtensions::none(), None).expect("failed to create vulkan instance");
//
//     print_devices_info(&instance);
//
//     let physical = PhysicalDevice::enumerate(&instance).next().expect("no device available");
//
//     let queue_family = physical.queue_families().find(|&q| q.supports_graphics()).expect("couldn't find a graphical queue family");
//
//     let (device, mut queues) = {
//         Device::new(physical, &Features::none(), &DeviceExtensions { khr_storage_buffer_storage_class: true, ..DeviceExtensions::none() }, [(queue_family, 0.5)].iter().cloned()).expect("failed device creation")
//     };
//
//     let queue = queues.next().unwrap();
//
//     let vs = vs::Shader::load(device.clone()).expect("failed to create shader module");
//     let fs = fs::Shader::load(device.clone()).expect("failed to create shader module");
//
//
//     let render_pass = Arc::new(vulkano::single_pass_renderpass!(device.clone(),
//         attachments: {
//             color: {
//                 load: Clear,
//                 store: Store,
//                 format: Format::R8G8B8A8Unorm,
//                 samples: 1,
//             }
//         },
//         pass: {
//             color: [color],
//             depth_stencil: {}
//         }
//     ).unwrap());
//
//     let image = StorageImage::new(device.clone(), Dimensions::Dim2d { width: 1024, height: 1024 }, Format::R8G8B8A8Unorm, Some(queue.family())).unwrap();
//
//     let framebuffer = Arc::new(Framebuffer::start(render_pass.clone())
//         .add(image.clone()).unwrap()
//         .build().unwrap());
//
//     AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family()).unwrap()
//                                                                                      .begin_render_pass(framebuffer.clone(), false, vec![[0.0, 0.0, 1.0, 1.0].into()])
//                                                                                      .unwrap()
//                                                                                      .end_render_pass()
//                                                                                      .unwrap();
//
//     let pipeline = Arc::new(GraphicsPipeline::start()
//         .vertex_input_single_buffer::<Vertex>()
//         .vertex_shader(vs.main_entry_point(), ())
//         .viewports_dynamic_scissors_irrelevant(1)
//         .fragment_shader(fs.main_entry_point(), ())
//         .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
//         .build(device.clone())
//         .unwrap());
//
//     let vertex1 = Vertex { position: [-0.5, -0.5] };
//     let vertex2 = Vertex { position: [0.0, 0.5] };
//     let vertex3 = Vertex { position: [0.5, -0.25] };
//
//     // let vertex4 = Vertex { position: [0.5, -0.75] };
//
//     let vertex_buffer = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, vec![vertex1.clone(), vertex2.clone(), vertex3.clone()].into_iter()).unwrap();
//     // let vertex_buffer_2 = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, vec![vertex2, vertex3, vertex4].into_iter()).unwrap();
//
//     let dynamic_state = DynamicState {
//         viewports: Some(vec![Viewport {
//             origin: [0.0, 0.0],
//             dimensions: [1024.0, 1024.0],
//             depth_range: 0.0..1.0,
//         }]),
//         ..DynamicState::none()
//     };
//
//     let buf = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, (0..1024 * 1024 * 4).map(|_| 0u8)).expect("failed to create buffer");
//
//     let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family()).unwrap()
//                                                                                                           .begin_render_pass(framebuffer.clone(), false, vec![[0.0, 0.0, 1.0, 1.0].into()])
//                                                                                                           .unwrap()
//                                                                                                           .draw(pipeline.clone(), &dynamic_state, vertex_buffer.clone(), (), ())
//                                                                                                           .unwrap()
//                                                                                                           // .draw(pipeline.clone(), &dynamic_state, vertex_buffer_2.clone(), (), ())
//                                                                                                           // .unwrap()
//                                                                                                           .end_render_pass()
//                                                                                                           .unwrap()
//                                                                                                           .copy_image_to_buffer(image.clone(), buf.clone())
//                                                                                                           .unwrap()
//                                                                                                           .build()
//                                                                                                           .unwrap();
//
//     let finished = command_buffer.execute(queue).unwrap();
//     finished.then_signal_fence_and_flush().unwrap()
//             .wait(None).unwrap();
//
//     let buffer_content = buf.read().unwrap();
//     let image_buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(1024, 1024, &buffer_content[..]).unwrap();
//     let _ = image_buffer.save("triangle.png");
// }
//
// mod cs {
//     vulkano_shaders::shader! {
//     ty: "compute",
//     src: "
// #version 450
// layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;
//
// layout(set = 0, binding = 0) buffer Data {
//     uint data[];
// } buf;
//
// void main()  {
//     uint idx = gl_GlobalInvocationID.x;
//     buf.data[idx] *= 12;
// }"
//     }
// }
// //
// // fn old_main() {
// //     let instance = Instance::new(None, &InstanceExtensions::none(), None).expect("failed to create vulkan instance");
// //
// //     print_devices_info(&instance);
// //
// //     let physical = PhysicalDevice::enumerate(&instance).next().expect("no device available");
// //
// //     let queue_family = physical.queue_families().find(|&q| q.supports_graphics()).expect("couldn't find a graphical queue family");
// //
// //     let (device, mut queues) = {
// //         Device::new(physical, &Features::none(), &DeviceExtensions { khr_storage_buffer_storage_class: true, ..DeviceExtensions::none() }, [(queue_family, 0.5)].iter().cloned()).expect("failed device creation")
// //     };
// //
// //     let queue = queues.next().unwrap();
// //
// //     let shader = mandelbrot_shader::Shader::load(device.clone()).expect("failed to create shader module");
// //     let compute_pipeline = Arc::new(ComputePipeline::new(device.clone(), &shader.main_entry_point(), &()).expect("failed to create compute pipeline"));
// //
// //     let storage_image = StorageImage::new(device.clone(), Dimensions::Dim2d { width: 1024, height: 1024 }, Format::R8G8B8A8Unorm, Some(queue.family())).unwrap();
// //     let layout = compute_pipeline.layout().descriptor_set_layout(0).unwrap();
// //     let set = Arc::new(PersistentDescriptorSet::start(layout.clone())
// //         .add_image(storage_image.clone()).unwrap()
// //         .build().unwrap()
// //     );
// //
// //     let buf = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, (0..1024 * 1024 * 4).map(|_| 0u8)).expect("failed to create buffer");
// //     let command_buffer = AutoCommandBufferBuilder::new(device.clone(), queue.family()).unwrap()
// //                                                                                       .dispatch([1024 / 8, 1024 / 8, 1], compute_pipeline.clone(), set.clone(), ()).unwrap()
// //                                                                                       .copy_image_to_buffer(storage_image.clone(), buf.clone()).unwrap()
// //                                                                                       .build().unwrap();
// //
// //     let finished = command_buffer.execute(queue.clone()).unwrap();
// //     finished.then_signal_fence_and_flush().unwrap()
// //             .wait(None).unwrap();
// //
// //     let buffer_content = buf.read().unwrap();
// //     let image_buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(1024, 1024, &buffer_content[..]).unwrap();
// //     image_buffer.save("image.png").unwrap();
// //
// //
// //     mod mandelbrot_shader {
// //         vulkano_shaders::shader! {
// //     ty: "compute",
// //     src: "
// // #version 450
// // layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;
// //
// // layout(set = 0, binding = 0, rgba8) uniform writeonly image2D img;
// //
// // void main()  {
// //     vec2 norm_coordinates = (gl_GlobalInvocationID.xy + vec2(0.5)) / vec2(imageSize(img));
// //     vec2 c = (norm_coordinates - vec2(0.5)) * 2.0 - vec2(1.0, 0.0);
// //
// //     vec2 z = vec2(0.0, 0.0);
// //     float i;
// //     for (i = 0.0; i < 1.0; i += 0.005) {
// //         z = vec2(
// //             z.x * z.x - z.y * z.y + c.x,
// //             z.y * z.x + z.x * z.y + c.y
// //         );
// //
// //         if (length(z) > 4.0) {
// //             break;
// //         }
// //     }
// //
// //     vec4 to_write = vec4(vec3(i), 1.0);
// //     imageStore(img, ivec2(gl_GlobalInvocationID.xy), to_write);
// // }"
// //     }
// //     }
// //
// //     println!("Everything succeeded!");
// // }
//
// fn print_devices_info(instance: &Arc<Instance>) {
//     for physical_device in PhysicalDevice::enumerate(&instance) {
//         println!("found a physical device name: {}", physical_device.name());
//         println!("\tapi version: {}", physical_device.api_version());
//         println!("\tdriver_version: {}", physical_device.driver_version());
//         println!("\tpci_vendor_id: {}", physical_device.pci_vendor_id());
//         println!("\tpci_device_id: {}", physical_device.pci_device_id());
// //        println!("\tsupported features: {:#?}", physical_device.supported_features());
//         println!("\tqueue families:");
//         for family in physical_device.queue_families() {
//             println!("\t\tFound a queue family with {:?} queue(s)", family.queues_count());
//         }
//         println!();
//     }
// }
