use std::ops::Deref;
use std::sync::Arc;

use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, DynamicState, SubpassContents};
use vulkano::descriptor::pipeline_layout::PipelineLayoutDesc;
use vulkano::device::Device;
use vulkano::device::DeviceExtensions;
use vulkano::device::Features;
use vulkano::image::{ImageUsage, SwapchainImage};
use vulkano::image::view::ImageView;
use vulkano::instance::Instance;
use vulkano::instance::InstanceExtensions;
use vulkano::instance::PhysicalDevice;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::viewport::Viewport;
use vulkano::render_pass::{Framebuffer, FramebufferAbstract, RenderPass, Subpass};
use vulkano::swapchain::{AcquireError, ColorSpace, FullscreenExclusive, PresentMode, SurfaceTransform, Swapchain, SwapchainCreationError};
use vulkano::sync;
use vulkano::sync::{FlushError, GpuFuture};
use vulkano_win::VkSurfaceBuild;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

fn main() {
    let instance = {
        let extensions = vulkano_win::required_extensions();
        Instance::new(None, &extensions, None).expect("failed to create vulkan instance")
    };

    print_devices_info(&instance);

    let physical = PhysicalDevice::enumerate(&instance).next().expect("no device available");

    let events_loop = EventLoop::new();
    let surface = WindowBuilder::new()
        .build_vk_surface(&events_loop, instance.clone())
        .unwrap();

    let queue_family = physical.queue_families()
                               .find(|&q| {
                                   q.supports_graphics() && surface.is_supported(q).unwrap_or(false)
                               })
                               .expect("couldn't find a graphical queue family");

    let device_ext = vulkano::device::DeviceExtensions {
        khr_swapchain: true,
        ..vulkano::device::DeviceExtensions::none()
    };

    let (device, mut queues) = {
        Device::new(physical, &Features::none(), &device_ext, [(queue_family, 0.5)].iter().cloned())
            .expect("failed device creation")
    };

    let queue = queues.next().unwrap();

    // params missing from guide: 1 + true
    let (mut swapchain, image_views) = {
        let capabilities = surface.capabilities(physical).expect("failed to get surface capabilities");

        let dimensions = capabilities.current_extent.unwrap_or([1280, 1024]);
        let alpha = capabilities.supported_composite_alpha.iter().next().unwrap();
        let format = capabilities.supported_formats[0].0;

        let (swapchain, images) = Swapchain::start(device.clone(), surface.clone())
            .num_images(capabilities.min_image_count)
            .format(format)
            .dimensions(dimensions)
            .usage(ImageUsage::color_attachment())
            .sharing_mode(&queue)
            .composite_alpha(alpha)
            .transform(SurfaceTransform::Identity)
            .present_mode(PresentMode::Fifo)
            .fullscreen_exclusive(FullscreenExclusive::Default)
            .color_space(ColorSpace::SrgbNonLinear)
            .build()
            .unwrap();
        let images: Vec<_> = images.into_iter().map(|img| ImageView::new(img).unwrap()).collect();
        (swapchain, images)
    };

    let vertex_buffer = {
        #[derive(Default, Debug, Clone)]
        struct Vertex {
            position: [f32; 2],
        }
        vulkano::impl_vertex!(Vertex, position);

        CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            false,
            [
                Vertex {
                    position: [-0.5, -0.25],
                },
                Vertex {
                    position: [0.0, 0.5],
                },
                Vertex {
                    position: [0.25, -0.1],
                },
            ]
                .iter()
                .cloned(),
        )
            .unwrap()
    };

    mod vs {
        vulkano_shaders::shader! {
            ty: "vertex",
            src: "
				#version 450
				layout(location = 0) in vec2 position;
				void main() {
					gl_Position = vec4(position, 0.0, 1.0);
				}
			"
        }
    }

    mod fs {
        vulkano_shaders::shader! {
            ty: "fragment",
            src: "
				#version 450
				layout(location = 0) out vec4 f_color;
				void main() {
					f_color = vec4(1.0, 0.0, 0.0, 1.0);
				}
			"
        }
    }

    let vs = vs::Shader::load(device.clone()).unwrap();
    let fs = fs::Shader::load(device.clone()).unwrap();

    let render_pass = Arc::new(
        vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                // `color` is a custom name we give to the first and only attachment.
                color: {
                    // `load: Clear` means that we ask the GPU to clear the content of this
                    // attachment at the start of the drawing.
                    load: Clear,
                    // `store: Store` means that we ask the GPU to store the output of the draw
                    // in the actual image. We could also ask it to discard the result.
                    store: Store,
                    // `format: <ty>` indicates the type of the format of the image. This has to
                    // be one of the types of the `vulkano::format` module (or alternatively one
                    // of your structs that implements the `FormatDesc` trait). Here we use the
                    // same format as the swapchain.
                    format: swapchain.format(),
                    // TODO:
                    samples: 1,
                }
            },
            pass: {
                // We use the attachment named `color` as the one and only color attachment.
                color: [color],
                // No depth-stencil attachment is indicated with empty brackets.
                depth_stencil: {}
            }
        )
            .unwrap(),
    );

    let pipeline = Arc::new(
        GraphicsPipeline::start()
            // We need to indicate the layout of the vertices.
            // The type `SingleBufferDefinition` actually contains a template parameter corresponding
            // to the type of each vertex. But in this code it is automatically inferred.
            .vertex_input_single_buffer()
            // A Vulkan shader can in theory contain multiple entry points, so we have to specify
            // which one. The `main` word of `main_entry_point` actually corresponds to the name of
            // the entry point.
            .vertex_shader(vs.main_entry_point(), ())
            // The content of the vertex buffer describes a list of triangles.
            .triangle_list()
            // Use a resizable viewport set to draw over the entire window
            .viewports_dynamic_scissors_irrelevant(1)
            // See `vertex_shader`.
            .fragment_shader(fs.main_entry_point(), ())
            // We have to indicate which subpass of which render pass this pipeline is going to be used
            // in. The pipeline will only be usable from this particular subpass.
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            // Now that our builder is filled, we call `build()` to obtain an actual pipeline.
            .build(device.clone())
            .unwrap(),
    );

    let mut dynamic_state = DynamicState {
        line_width: None,
        viewports: None,
        scissors: None,
        compare_mask: None,
        write_mask: None,
        reference: None,
    };

    let mut framebuffers =
        window_size_dependent_setup(&image_views, render_pass.clone(), &mut dynamic_state);

    let mut recreate_swapchain = false;

    let mut previous_frame_end = Some(sync::now(device.clone()).boxed());

    events_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent { event: WindowEvent::Resized(_), .. } => {
                recreate_swapchain = true;
            }
            Event::RedrawEventsCleared => {
                previous_frame_end.as_mut().unwrap().cleanup_finished();

                // Whenever the window resizes we need to recreate everything dependent on the window size.
                // In this example that includes the swapchain, the framebuffers and the dynamic state viewport.
                if recreate_swapchain {
                    // Get the new dimensions of the window.
                    let dimensions: [u32; 2] = surface.window().inner_size().into();
                    let (new_swapchain, new_image_views) =
                        match swapchain.recreate().dimensions(dimensions).build() {
                            Ok((new_swapchain, new_images)) => {
                                let new_image_views: Vec<_> = new_images.into_iter().map(|img| ImageView::new(img).unwrap()).collect();
                                (new_swapchain, new_image_views)
                            }
                            // This error tends to happen when the user is manually resizing the window.
                            // Simply restarting the loop is the easiest way to fix this issue.
                            Err(SwapchainCreationError::UnsupportedDimensions) => return,
                            Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
                        };

                    swapchain = new_swapchain;
                    // Because framebuffers contains an Arc on the old swapchain, we need to
                    // recreate framebuffers as well.
                    framebuffers = window_size_dependent_setup(
                        &new_image_views,
                        render_pass.clone(),
                        &mut dynamic_state,
                    );
                    recreate_swapchain = false;
                }

                let (image_num, suboptimal, acquire_future) =
                    match vulkano::swapchain::acquire_next_image(swapchain.clone(), None) {
                        Ok(r) => r,
                        Err(AcquireError::OutOfDate) => {
                            recreate_swapchain = true;
                            return;
                        }
                        Err(e) => panic!("Failed to acquire next image: {:?}", e),
                    };

                if suboptimal {
                    recreate_swapchain = true;
                }

                let clear_values = vec![[0.0, 0.0, 1.0, 1.0].into()];

                let mut builder = vulkano::command_buffer::AutoCommandBufferBuilder::primary(
                    device.clone(),
                    queue.family(),
                    CommandBufferUsage::OneTimeSubmit
                ).unwrap();


                let x = builder
                    .begin_render_pass(framebuffers[image_num].clone(), SubpassContents::Inline, clear_values)
                    .unwrap()
                    .draw(pipeline.clone(), &dynamic_state, vertex_buffer.clone(), (), (), vec![])
                    .unwrap()
                    .end_render_pass()
                    .unwrap();

                let command_buffer = builder.build().unwrap();

                let future = previous_frame_end
                    .take()
                    .unwrap()
                    .join(acquire_future)
                    .then_execute(queue.clone(), command_buffer)
                    .unwrap()
                    .then_swapchain_present(queue.clone(), swapchain.clone(), image_num)
                    .then_signal_fence_and_flush();

                match future {
                    Ok(future) => {
                        previous_frame_end = Some(future.boxed());
                    }
                    Err(FlushError::OutOfDate) => {
                        recreate_swapchain = true;
                        previous_frame_end = Some(sync::now(device.clone()).boxed())
                    }
                    Err(e) => {
                        println!("Failed to flush future: {:?}", e);
                        previous_frame_end = Some(sync::now(device.clone()).boxed());
                    }
                }
            }
            _ => ()
        }
    });
}

fn window_size_dependent_setup(
    image_views: &Vec<Arc<ImageView<Arc<SwapchainImage<Window>>>>>,
    render_pass: Arc<RenderPass>,
    dynamic_state: &mut DynamicState,
) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
    let dimensions = image_views[0].image().dimensions();

    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
        depth_range: 0.0..1.0,
    };
    dynamic_state.viewports = Some(vec![viewport]);

    image_views
        .iter()
        .map(|image_view| {
            let arc_swapchain_image = image_view.clone();
            let builder = Framebuffer::start(render_pass.clone())
                .add(arc_swapchain_image)
                .unwrap();
            let framebuffer = builder
                .build()
                .unwrap();
            let framebuffer_abstract = Arc::new(
                framebuffer,
            ) as Arc<dyn FramebufferAbstract + Send + Sync>;
            framebuffer_abstract
        })
        .collect::<Vec<_>>()
}

fn print_devices_info(instance: &Arc<Instance>) {
    for physical_device in PhysicalDevice::enumerate(&instance) {
        println!("found a physical device name: {}", physical_device.name());
        println!("\tapi version: {}", physical_device.api_version());
        println!("\tdriver_version: {}", physical_device.driver_version());
        println!("\tpci_vendor_id: {}", physical_device.pci_vendor_id());
        println!("\tpci_device_id: {}", physical_device.pci_device_id());
//        println!("\tsupported features: {:#?}", physical_device.supported_features());
        println!("\tqueue families:");
        for family in physical_device.queue_families() {
            println!("\t\tFound a queue family with {:?} queue(s)", family.queues_count());
        }
        println!();
    }
}

