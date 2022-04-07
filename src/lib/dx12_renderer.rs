use std::{env, ffi::c_void, path::PathBuf};
use windows::{
    core::{Interface, PCSTR, PCWSTR},
    Win32::{
        Foundation::{HANDLE, HWND, RECT},
        Graphics::{
            Direct3D::{
                Fxc::{D3DCompileFromFile, D3DCOMPILE_DEBUG, D3DCOMPILE_SKIP_OPTIMIZATION},
                D3D_FEATURE_LEVEL_11_0, D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
            },
            Direct3D12::{
                D3D12CreateDevice, D3D12GetDebugInterface, D3D12SerializeRootSignature,
                ID3D12CommandAllocator, ID3D12CommandList, ID3D12CommandQueue, ID3D12Debug,
                ID3D12DescriptorHeap, ID3D12Device, ID3D12Fence, ID3D12GraphicsCommandList,
                ID3D12PipelineState, ID3D12Resource, ID3D12RootSignature, D3D12_BLEND_DESC,
                D3D12_BLEND_ONE, D3D12_BLEND_OP_ADD, D3D12_BLEND_ZERO,
                D3D12_COLOR_WRITE_ENABLE_ALL, D3D12_COMMAND_LIST_TYPE_DIRECT,
                D3D12_COMMAND_QUEUE_DESC, D3D12_CPU_DESCRIPTOR_HANDLE, D3D12_CULL_MODE_NONE,
                D3D12_DEPTH_STENCIL_DESC, D3D12_DESCRIPTOR_HEAP_DESC,
                D3D12_DESCRIPTOR_HEAP_TYPE_RTV, D3D12_FENCE_FLAG_NONE, D3D12_FILL_MODE_SOLID,
                D3D12_GRAPHICS_PIPELINE_STATE_DESC, D3D12_HEAP_FLAG_NONE, D3D12_HEAP_PROPERTIES,
                D3D12_HEAP_TYPE_UPLOAD, D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
                D3D12_INPUT_ELEMENT_DESC, D3D12_INPUT_LAYOUT_DESC, D3D12_LOGIC_OP_NOOP,
                D3D12_MAX_DEPTH, D3D12_MIN_DEPTH, D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
                D3D12_RASTERIZER_DESC, D3D12_RENDER_TARGET_BLEND_DESC, D3D12_RESOURCE_BARRIER,
                D3D12_RESOURCE_BARRIER_0, D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                D3D12_RESOURCE_BARRIER_FLAG_NONE, D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
                D3D12_RESOURCE_DESC, D3D12_RESOURCE_DIMENSION_BUFFER, D3D12_RESOURCE_STATES,
                D3D12_RESOURCE_STATE_GENERIC_READ, D3D12_RESOURCE_STATE_PRESENT,
                D3D12_RESOURCE_STATE_RENDER_TARGET, D3D12_RESOURCE_TRANSITION_BARRIER,
                D3D12_ROOT_SIGNATURE_DESC,
                D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT,
                D3D12_SHADER_BYTECODE, D3D12_TEXTURE_LAYOUT_ROW_MAJOR, D3D12_VERTEX_BUFFER_VIEW,
                D3D12_VIEWPORT, D3D_ROOT_SIGNATURE_VERSION_1,
            },
            Dxgi::{
                Common::{
                    DXGI_FORMAT_R32G32B32_FLOAT, DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_SAMPLE_DESC,
                },
                CreateDXGIFactory2, IDXGIAdapter1, IDXGIFactory4, IDXGISwapChain3, IDXGISwapChain4,
                DXGI_ADAPTER_FLAG, DXGI_ADAPTER_FLAG_NONE, DXGI_ADAPTER_FLAG_SOFTWARE,
                DXGI_CREATE_FACTORY_DEBUG, DXGI_MWA_NO_ALT_ENTER, DXGI_SWAP_CHAIN_DESC1,
                DXGI_SWAP_EFFECT_FLIP_DISCARD, DXGI_USAGE_RENDER_TARGET_OUTPUT,
            },
        },
        System::{
            Threading::{CreateEventA, WaitForSingleObject},
            WindowsProgramming::INFINITE,
        },
    },
};

use crate::utils::ToWide;
pub use crate::{
    error::{Error, Win32Error},
    win_error,
};
pub type Result<T> = std::result::Result<T, crate::error::Error>;

const FRAME_COUNT: u32 = 2;

#[derive(Debug)]
pub struct D3D12Renderer {
    factory: IDXGIFactory4,
    device: ID3D12Device,
}

impl D3D12Renderer {
    pub fn new() -> Result<Self> {
        // Enable the D3D12 debug layer when debug build.
        if cfg!(debug_assertions) {
            unsafe {
                let mut debug: Option<ID3D12Debug> = None;
                if let Some(debug) = D3D12GetDebugInterface(&mut debug).ok().and(debug) {
                    debug.EnableDebugLayer();
                }
            }
        }

        // Flag to load DXGIDebug.dll or not
        let dxgi_factory_flags = if cfg!(debug_assertions) {
            DXGI_CREATE_FACTORY_DEBUG
        } else {
            0
        };

        // Factory is used to load other DXGI objects.
        let dxgi_factory: IDXGIFactory4 = unsafe { CreateDXGIFactory2(dxgi_factory_flags) }
            .map_err(|e| Error::Win32Error(win_error!(e)))?;

        let adapter = Self::get_hardware_adapter(&dxgi_factory)?;

        //a ID3D12Device struct represents the DX12 compatible GPU (the adapter)
        let mut device: Option<ID3D12Device> = None;
        unsafe { D3D12CreateDevice(adapter, D3D_FEATURE_LEVEL_11_0, &mut device) }
            .map_err(|e| Error::Win32Error(win_error!(e)))?;

        Ok(Self {
            factory: dxgi_factory,
            device: device.unwrap(),
        })
    }

    /// Find and return a DX12 compatible GPU
    fn get_hardware_adapter(factory: &IDXGIFactory4) -> Result<IDXGIAdapter1> {
        for i in 0.. {
            let adapter = unsafe {
                factory
                    .EnumAdapters1(i)
                    .map_err(|e| Error::Win32Error(win_error!(e)))?
            };

            let desc = unsafe {
                adapter
                    .GetDesc1()
                    .map_err(|e| Error::Win32Error(win_error!(e)))?
            };

            if (desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0) != DXGI_ADAPTER_FLAG_NONE.0 {
                // Don't select the Basic Render Driver adapter. If you want a
                // software adapter, pass in "/warp" on the command line.
                continue;
            }

            // Check to see whether the adapter supports Direct3D 12, but don't
            // create the actual device yet.
            if unsafe {
                D3D12CreateDevice(
                    &adapter,
                    D3D_FEATURE_LEVEL_11_0,
                    std::ptr::null_mut::<Option<ID3D12Device>>(),
                )
            }
            .is_ok()
            {
                return Ok(adapter);
            }
        }
        // Panic when we can't find DirectX12 compatible device
        unreachable!("Unable to find DirectX12 compatible device!")
    }
}
#[derive(Debug)]
pub(crate) struct Resources {
    command_queue: ID3D12CommandQueue,
    swap_chain: IDXGISwapChain3,
    frame_index: u32,
    render_targets: [ID3D12Resource; FRAME_COUNT as usize],
    rtv_heap: ID3D12DescriptorHeap,
    rtv_descriptor_size: usize,
    viewport: D3D12_VIEWPORT,
    scissor_rect: RECT,
    command_allocator: ID3D12CommandAllocator,
    root_signature: ID3D12RootSignature,
    pso: ID3D12PipelineState,
    command_list: ID3D12GraphicsCommandList,

    // we need to keep this around to keep the reference alive, even though
    // nothing reads from it
    #[allow(dead_code)]
    vertex_buffer: ID3D12Resource,

    vbv: D3D12_VERTEX_BUFFER_VIEW,
    fence: ID3D12Fence,
    fence_value: u64,
    fence_event: HANDLE,
}

impl Resources {
    pub fn new(
        renderer: &mut D3D12Renderer,
        win32_window_handle: HWND,
        size: (i32, i32),
    ) -> Result<Self> {
        // Create a command buffer (queue) that the GPU can execute.
        let command_queue: ID3D12CommandQueue = unsafe {
            renderer
                .device
                .CreateCommandQueue(&D3D12_COMMAND_QUEUE_DESC {
                    Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
                    ..Default::default()
                })
                .map_err(|e| Error::Win32Error(win_error!(e)))?
        };

        // window width and height
        let (width, height) = size;

        // descriptor struct for creating the swap chain.
        // Describes a four-component, 32-bit unsigned-normalized-integer format that supports 8 bits per channel including alpha.
        // Intended as an output render target and backbuffer content is discarded after presentation with no multi-sampling.
        let swap_chain_desc = DXGI_SWAP_CHAIN_DESC1 {
            BufferCount: FRAME_COUNT,
            Width: width as u32,
            Height: height as u32,
            Format: DXGI_FORMAT_R8G8B8A8_UNORM,
            BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
            SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                ..Default::default()
            },
            ..Default::default()
        };

        let swap_chain: IDXGISwapChain3 = unsafe {
            renderer
                .factory
                .CreateSwapChainForHwnd(
                    &command_queue,
                    win32_window_handle,
                    &swap_chain_desc,
                    std::ptr::null(),
                    None,
                )
                .map_err(|e| Error::Win32Error(win_error!(e)))?
        }
        .cast()
        .map_err(|e| Error::Win32Error(win_error!(e)))?;

        // TODO: Geert: Make the application support full-screen transitions (via ALT + ENTER?)
        // reference: https://docs.microsoft.com/en-us/windows/win32/api/dxgi/nf-dxgi-idxgifactory-makewindowassociation
        unsafe {
            renderer
                .factory
                .MakeWindowAssociation(win32_window_handle, DXGI_MWA_NO_ALT_ENTER)
                .map_err(|e| Error::Win32Error(win_error!(e)))?;
        }
        // Index of the current back buffer
        let frame_index = unsafe { swap_chain.GetCurrentBackBufferIndex() };

        // Describe and create a render target view (RTV) descriptor heap.
        let rtv_heap: ID3D12DescriptorHeap = unsafe {
            renderer
                .device
                .CreateDescriptorHeap(&D3D12_DESCRIPTOR_HEAP_DESC {
                    NumDescriptors: FRAME_COUNT,
                    Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
                    ..Default::default()
                })
        }
        .map_err(|e| Error::Win32Error(win_error!(e)))?;

        // used to increment a handle into our rtv descriptor heap array by the correct amount.
        let rtv_descriptor_size = unsafe {
            renderer
                .device
                .GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV)
        } as usize;

        // Returns the CPU descriptor handle that represents the start of our rtv descriptor heap.
        let rtv_handle = unsafe { rtv_heap.GetCPUDescriptorHandleForHeapStart() };

        // Initialize all render targets (2 in this case) and store in an array.
        // Uses the array-init crate. (https://docs.rs/array-init/latest/array_init/)
        let render_targets: [ID3D12Resource; FRAME_COUNT as usize] =
            array_init::try_array_init(|i: usize| -> Result<ID3D12Resource> {
                let render_target: ID3D12Resource = unsafe { swap_chain.GetBuffer(i as u32) }
                    .map_err(|e| Error::Win32Error(win_error!(e)))?;
                unsafe {
                    renderer.device.CreateRenderTargetView(
                        &render_target,
                        std::ptr::null_mut(),
                        &D3D12_CPU_DESCRIPTOR_HANDLE {
                            ptr: rtv_handle.ptr + i * rtv_descriptor_size,
                        },
                    )
                };
                Ok(render_target)
            })?;

        let viewport = D3D12_VIEWPORT {
            TopLeftX: 0.0,
            TopLeftY: 0.0,
            Width: width as f32,
            Height: height as f32,
            MinDepth: D3D12_MIN_DEPTH,
            MaxDepth: D3D12_MAX_DEPTH,
        };

        let scissor_rect = RECT {
            left: 0,
            top: 0,
            right: width,
            bottom: height,
        };

        let command_allocator: ID3D12CommandAllocator = unsafe {
            renderer
                .device
                .CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT)
        }
        .map_err(|e| Error::Win32Error(win_error!(e)))?;

        let root_signature = Resources::create_root_signature(&renderer.device)?;

        // Generate pipeline state object (pso)
        let pso = Resources::create_pipeline_state(&renderer.device, &root_signature)?;

        // Encapsulates a list of graphics commands for rendering.
        let command_list: ID3D12GraphicsCommandList = unsafe {
            renderer.device.CreateCommandList(
                0,
                D3D12_COMMAND_LIST_TYPE_DIRECT,
                &command_allocator,
                &pso,
            )
        }
        .map_err(|e| Error::Win32Error(win_error!(e)))?;

        // Indicates that recording to the command list has finished.
        unsafe {
            command_list
                .Close()
                .map_err(|e| Error::Win32Error(win_error!(e)))?;
        };

        let aspect_ratio = width as f32 / height as f32;

        let (vertex_buffer, vbv) = Resources::create_vertex_buffer(&renderer.device, aspect_ratio)?;

        let fence = unsafe {
            renderer
                .device
                .CreateFence::<ID3D12Fence>(0, D3D12_FENCE_FLAG_NONE)
        }
        .map_err(|e| Error::Win32Error(win_error!(e)))?;

        let fence_value = 1;

        let fence_event = unsafe {
            CreateEventA(std::ptr::null_mut(), false, false, None)
                .map_err(|e| Error::Win32Error(win_error!(e)))?
        };

        Ok(Resources {
            command_queue,
            swap_chain,
            frame_index,
            render_targets,
            rtv_heap,
            rtv_descriptor_size,
            viewport,
            scissor_rect,
            command_allocator,
            root_signature,
            pso,
            command_list,
            vertex_buffer,
            vbv,
            fence,
            fence_value,
            fence_event,
        })
    }

    fn create_root_signature(device: &ID3D12Device) -> Result<ID3D12RootSignature> {
        // Describes the layout of a root signature.
        // Opting in to using the Input Assembler (requiring an input layout that defines a set of vertex buffer bindings).
        let desc = D3D12_ROOT_SIGNATURE_DESC {
            Flags: D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT,
            ..Default::default()
        };

        // Serializes a root signature that can be passed to ID3D12Device::CreateRootSignature.
        let mut signature = None;
        let signature = unsafe {
            D3D12SerializeRootSignature(
                &desc,
                D3D_ROOT_SIGNATURE_VERSION_1,
                &mut signature,
                std::ptr::null_mut(),
            )
        }
        .map(|()| signature.unwrap())
        .map_err(|e| Error::Win32Error(win_error!(e)))?;

        unsafe {
            device
                .CreateRootSignature::<ID3D12RootSignature>(
                    0,
                    std::slice::from_raw_parts(
                        signature.GetBufferPointer() as *const u8,
                        signature.GetBufferSize(),
                    ),
                )
                .map_err(|e| Error::Win32Error(win_error!(e)))
        }
    }

    fn create_pipeline_state(
        device: &ID3D12Device,
        root_signature: &ID3D12RootSignature,
    ) -> Result<ID3D12PipelineState> {
        let compile_flags = if cfg!(debug_assertions) {
            D3DCOMPILE_DEBUG | D3DCOMPILE_SKIP_OPTIMIZATION
        } else {
            0
        };

        // Loading a shader file (.hlsl)
        let exe_path = std::env::current_exe().ok().unwrap();
        let asset_path = exe_path.parent().unwrap();
        let mut shaders_hlsl_path = asset_path.join("shaders.hlsl");
        if let Some(hlsl_file_name) = env::args().skip(1).next() {
            let potential_hlsl_path = asset_path.join(&hlsl_file_name);
            if potential_hlsl_path.is_file() {
                println!("Shader file: {}", hlsl_file_name);
                shaders_hlsl_path = potential_hlsl_path;
            } else {
                println!("{}, is not a file.", hlsl_file_name);
            }
        }

        let shaders_hlsl = shaders_hlsl_path.to_str().unwrap();

        println!("START SHADER COMPILATION");

        let mut vertex_shader = None;
        let vertex_shader = unsafe {
            D3DCompileFromFile(
                PCWSTR(shaders_hlsl.to_wide()),
                std::ptr::null_mut(),
                None,
                PCSTR(b"VSMain\0".as_ptr() as *mut u8),
                PCSTR(b"vs_5_0\0".as_ptr() as *mut u8),
                compile_flags,
                0,
                &mut vertex_shader,
                std::ptr::null_mut(),
            )
        }
        .map(|()| vertex_shader.unwrap())
        .map_err(|e| Error::Win32Error(win_error!(e)))?;

        println!("VS Compile OK");

        let mut pixel_shader = None;

        // Note Geert: 'D3DCompileFromFile is expected to just accept &str from v0.31.0'
        let pixel_shader = unsafe {
            D3DCompileFromFile(
                PCWSTR(shaders_hlsl.to_wide()),
                std::ptr::null_mut(),
                None,
                PCSTR(b"PSMain\0".as_ptr() as *mut u8),
                PCSTR(b"ps_5_0\0".as_ptr() as *mut u8),
                compile_flags,
                0,
                &mut pixel_shader,
                std::ptr::null_mut(),
            )
        }
        .map(|()| pixel_shader.unwrap())
        .map_err(|e| Error::Win32Error(win_error!(e)))?;

        println!("PS Compile OK");

        let mut input_element_descs: [D3D12_INPUT_ELEMENT_DESC; 2] = [
            D3D12_INPUT_ELEMENT_DESC {
                SemanticName: PCSTR(b"POSITION\0".as_ptr() as *mut u8),
                SemanticIndex: 0,
                Format: DXGI_FORMAT_R32G32B32_FLOAT,
                InputSlot: 0,
                AlignedByteOffset: 0,
                InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
                InstanceDataStepRate: 0,
            },
            D3D12_INPUT_ELEMENT_DESC {
                SemanticName: PCSTR(b"COLOR\0".as_ptr() as *mut u8),
                SemanticIndex: 0,
                Format: DXGI_FORMAT_R32G32B32_FLOAT,
                InputSlot: 0,
                AlignedByteOffset: 12,
                InputSlotClass: D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
                InstanceDataStepRate: 0,
            },
        ];

        let mut desc = D3D12_GRAPHICS_PIPELINE_STATE_DESC {
            InputLayout: D3D12_INPUT_LAYOUT_DESC {
                pInputElementDescs: input_element_descs.as_mut_ptr(),
                NumElements: input_element_descs.len() as u32,
            },
            pRootSignature: Some(root_signature.clone()),
            VS: D3D12_SHADER_BYTECODE {
                pShaderBytecode: unsafe { vertex_shader.GetBufferPointer() },
                BytecodeLength: unsafe { vertex_shader.GetBufferSize() },
            },
            PS: D3D12_SHADER_BYTECODE {
                pShaderBytecode: unsafe { pixel_shader.GetBufferPointer() },
                BytecodeLength: unsafe { pixel_shader.GetBufferSize() },
            },
            RasterizerState: D3D12_RASTERIZER_DESC {
                FillMode: D3D12_FILL_MODE_SOLID,
                CullMode: D3D12_CULL_MODE_NONE,
                ..Default::default()
            },
            BlendState: D3D12_BLEND_DESC {
                AlphaToCoverageEnable: false.into(),
                IndependentBlendEnable: false.into(),
                RenderTarget: [
                    D3D12_RENDER_TARGET_BLEND_DESC {
                        BlendEnable: false.into(),
                        LogicOpEnable: false.into(),
                        SrcBlend: D3D12_BLEND_ONE,
                        DestBlend: D3D12_BLEND_ZERO,
                        BlendOp: D3D12_BLEND_OP_ADD,
                        SrcBlendAlpha: D3D12_BLEND_ONE,
                        DestBlendAlpha: D3D12_BLEND_ZERO,
                        BlendOpAlpha: D3D12_BLEND_OP_ADD,
                        LogicOp: D3D12_LOGIC_OP_NOOP,
                        RenderTargetWriteMask: D3D12_COLOR_WRITE_ENABLE_ALL.0 as u8,
                    },
                    D3D12_RENDER_TARGET_BLEND_DESC::default(),
                    D3D12_RENDER_TARGET_BLEND_DESC::default(),
                    D3D12_RENDER_TARGET_BLEND_DESC::default(),
                    D3D12_RENDER_TARGET_BLEND_DESC::default(),
                    D3D12_RENDER_TARGET_BLEND_DESC::default(),
                    D3D12_RENDER_TARGET_BLEND_DESC::default(),
                    D3D12_RENDER_TARGET_BLEND_DESC::default(),
                ],
            },
            DepthStencilState: D3D12_DEPTH_STENCIL_DESC::default(),
            SampleMask: u32::max_value(),
            PrimitiveTopologyType: D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE,
            NumRenderTargets: 1,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                ..Default::default()
            },
            ..Default::default()
        };
        desc.RTVFormats[0] = DXGI_FORMAT_R8G8B8A8_UNORM;

        unsafe { device.CreateGraphicsPipelineState::<ID3D12PipelineState>(&desc) }
            .map_err(|e| Error::Win32Error(win_error!(e)))
    }

    fn create_vertex_buffer(
        device: &ID3D12Device,
        aspect_ratio: f32,
    ) -> Result<(ID3D12Resource, D3D12_VERTEX_BUFFER_VIEW)> {
        let vertices = [
            Vertex {
                position: [0.0, 0.25 * aspect_ratio, 0.0],
                color: [1.0, 0.0, 0.0, 1.0],
            },
            Vertex {
                position: [0.25, -0.25 * aspect_ratio, 0.0],
                color: [0.0, 1.0, 0.0, 1.0],
            },
            Vertex {
                position: [-0.25, -0.25 * aspect_ratio, 0.0],
                color: [0.0, 0.0, 1.0, 1.0],
            },
        ];

        // Note: using upload heaps to transfer static data like vert buffers is
        // not recommended. Every time the GPU needs it, the upload heap will be
        // marshalled over. Please read up on Default Heap usage. An upload heap
        // is used here for code simplicity and because there are very few verts
        // to actually transfer.
        let mut vertex_buffer: Option<ID3D12Resource> = None;

        unsafe {
            device
                .CreateCommittedResource(
                    &D3D12_HEAP_PROPERTIES {
                        Type: D3D12_HEAP_TYPE_UPLOAD,
                        ..Default::default()
                    },
                    D3D12_HEAP_FLAG_NONE,
                    &D3D12_RESOURCE_DESC {
                        Dimension: D3D12_RESOURCE_DIMENSION_BUFFER,
                        Width: std::mem::size_of_val(&vertices) as u64,
                        Height: 1,
                        DepthOrArraySize: 1,
                        MipLevels: 1,
                        SampleDesc: DXGI_SAMPLE_DESC {
                            Count: 1,
                            Quality: 0,
                        },
                        Layout: D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
                        ..Default::default()
                    },
                    D3D12_RESOURCE_STATE_GENERIC_READ,
                    std::ptr::null(),
                    &mut vertex_buffer,
                )
                .map_err(|e| Error::Win32Error(win_error!(e)))?
        };

        let vertex_buffer = vertex_buffer.unwrap();

        // Copy the triangle data to the vertex buffer.
        // Gets a CPU pointer to the specified subresource in the resource, but may not disclose the pointer value to applications.
        // Map also invalidates the CPU cache, when necessary, so that CPU reads to this address reflect any modifications made by the GPU.
        unsafe {
            let mut data = std::ptr::null_mut();
            vertex_buffer
                .Map(0, std::ptr::null(), &mut data)
                .map_err(|e| Error::Win32Error(win_error!(e)))?;
            std::ptr::copy_nonoverlapping(
                vertices.as_ptr(),
                data as *mut Vertex,
                std::mem::size_of_val(&vertices),
            );
            // Invalidate the CPU pointer to the specified subresource in the resource.
            vertex_buffer.Unmap(0, std::ptr::null());
        }

        let vbv = D3D12_VERTEX_BUFFER_VIEW {
            BufferLocation: unsafe { vertex_buffer.GetGPUVirtualAddress() },
            StrideInBytes: std::mem::size_of::<Vertex>() as u32,
            SizeInBytes: std::mem::size_of_val(&vertices) as u32,
        };

        Ok((vertex_buffer, vbv))
    }
}

#[derive(Debug)]
struct Camera {
    position: Vec3f,
    direction: Vec3f,
}

struct Vertex {
    position: Vec3f,
    color: Vec4f,
}

type Vec3f = [f32; 3];
type Vec4f = [f32; 4];

#[derive(Debug)]
pub struct D3D12 {
    renderer: D3D12Renderer,
    resources: Option<Resources>,
}

impl D3D12 {
    pub fn new() -> Result<Self> {
        let renderer = D3D12Renderer::new()?;
        Ok(Self {
            renderer,
            resources: None,
        })
    }

    pub fn create_resources(&mut self, win32_window_handle: HWND, size: (i32, i32)) -> Result<()> {
        let resources = Resources::new(&mut self.renderer, win32_window_handle, size).ok();
        self.resources = resources;

        Ok(())
    }

    pub fn render(&mut self) {
        if let Some(resources) = &mut self.resources {
            // Poluplate command list in Resources struct.
            D3D12::populate_command_list(resources).unwrap();

            // Execute the command list.
            let command_list = ID3D12CommandList::from(&resources.command_list);
            unsafe {
                resources
                    .command_queue
                    .ExecuteCommandLists(&[Some(command_list)])
            };

            // Present the frame.
            unsafe { resources.swap_chain.Present(1, 0) }.ok().unwrap();

            D3D12::wait_for_previous_frame(resources);
        }
    }

    fn populate_command_list(resources: &Resources) -> Result<()> {
        // Command list allocators can only be reset when the associated
        // command lists have finished execution on the GPU; apps should use
        // fences to determine GPU execution progress.
        unsafe {
            // Indicates to re-use the memory that is associated with the command allocator.
            resources
                .command_allocator
                .Reset()
                .map_err(|e| Error::Win32Error(win_error!(e)))?;
        }

        let command_list = &resources.command_list;

        // However, when ExecuteCommandList() is called on a particular
        // command list, that command list can then be reset at any time and
        // must be before re-recording.
        unsafe {
            command_list
                .Reset(&resources.command_allocator, &resources.pso)
                .map_err(|e| Error::Win32Error(win_error!(e)))?;
        }

        // Set necessary state.
        unsafe {
            command_list.SetGraphicsRootSignature(&resources.root_signature);
            command_list.RSSetViewports(&[resources.viewport]);
            command_list.RSSetScissorRects(&[resources.scissor_rect]);
        }

        // Indicate that the back buffer will be used as a render target.
        let barrier = D3D12::transition_barrier(
            &resources.render_targets[resources.frame_index as usize],
            D3D12_RESOURCE_STATE_PRESENT,
            D3D12_RESOURCE_STATE_RENDER_TARGET,
        );

        // Notifies the driver that it needs to synchronize multiple accesses to resources.
        unsafe { command_list.ResourceBarrier(&[barrier]) };

        let rtv_handle = D3D12_CPU_DESCRIPTOR_HANDLE {
            ptr: unsafe { resources.rtv_heap.GetCPUDescriptorHandleForHeapStart() }.ptr
                + resources.frame_index as usize * resources.rtv_descriptor_size,
        };

        unsafe { command_list.OMSetRenderTargets(1, &rtv_handle, false, std::ptr::null()) };

        // Record commands.
        unsafe {
            command_list.ClearRenderTargetView(rtv_handle, [0.0, 0.2, 0.4, 1.0].as_ptr(), &[]);
            command_list.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
            command_list.IASetVertexBuffers(0, &[resources.vbv]);
            command_list.DrawInstanced(3, 1, 0, 0);

            // Indicate that the back buffer will now be used to present.
            command_list.ResourceBarrier(&[D3D12::transition_barrier(
                &resources.render_targets[resources.frame_index as usize],
                D3D12_RESOURCE_STATE_RENDER_TARGET,
                D3D12_RESOURCE_STATE_PRESENT,
            )]);
        }

        unsafe {
            command_list
                .Close()
                .map_err(|e| Error::Win32Error(win_error!(e)))
        }
    }

    fn transition_barrier(
        resource: &ID3D12Resource,
        state_before: D3D12_RESOURCE_STATES,
        state_after: D3D12_RESOURCE_STATES,
    ) -> D3D12_RESOURCE_BARRIER {
        // Describes a resource barrier (transition in resource use).
        // A transition barriers indicates that a set of subresources transition between different usages.
        // The caller must specify the before and after usages of the subresources.
        // The D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES flag is used to transition all subresources in a resource at the same time.
        D3D12_RESOURCE_BARRIER {
            Type: D3D12_RESOURCE_BARRIER_TYPE_TRANSITION,
            Flags: D3D12_RESOURCE_BARRIER_FLAG_NONE,
            Anonymous: D3D12_RESOURCE_BARRIER_0 {
                Transition: std::mem::ManuallyDrop::new(D3D12_RESOURCE_TRANSITION_BARRIER {
                    pResource: Some(resource.clone()),
                    StateBefore: state_before,
                    StateAfter: state_after,
                    Subresource: D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES,
                }),
            },
        }
    }

    fn wait_for_previous_frame(resources: &mut Resources) {
        // WAITING FOR THE FRAME TO COMPLETE BEFORE CONTINUING IS NOT BEST
        // PRACTICE. This is code implemented as such for simplicity. The
        // D3D12HelloFrameBuffering sample illustrates how to use fences for
        // efficient resource usage and to maximize GPU utilization.

        // Signal and increment the fence value.
        let fence = resources.fence_value;

        unsafe { resources.command_queue.Signal(&resources.fence, fence) }
            .ok()
            .unwrap();

        resources.fence_value += 1;

        // Wait until the previous frame is finished.
        if unsafe { resources.fence.GetCompletedValue() } < fence {
            unsafe {
                resources
                    .fence
                    .SetEventOnCompletion(fence, resources.fence_event)
            }
            .ok()
            .unwrap();

            unsafe { WaitForSingleObject(resources.fence_event, INFINITE) };
        }

        resources.frame_index = unsafe { resources.swap_chain.GetCurrentBackBufferIndex() };
    }
}