use std::ffi::c_void;
use windows::{
    core::Interface,
    Win32::{
        Foundation::{HANDLE, HWND, RECT},
        Graphics::{
            Direct3D::D3D_FEATURE_LEVEL_11_0,
            Direct3D12::{
                D3D12CreateDevice, D3D12GetDebugInterface, D3D12SerializeRootSignature,
                ID3D12CommandAllocator, ID3D12CommandQueue, ID3D12Debug, ID3D12DescriptorHeap,
                ID3D12Device, ID3D12Fence, ID3D12GraphicsCommandList, ID3D12PipelineState,
                ID3D12Resource, ID3D12RootSignature, D3D12_COMMAND_LIST_TYPE_DIRECT,
                D3D12_COMMAND_QUEUE_DESC, D3D12_CPU_DESCRIPTOR_HANDLE, D3D12_DESCRIPTOR_HEAP_DESC,
                D3D12_DESCRIPTOR_HEAP_TYPE_RTV, D3D12_MAX_DEPTH, D3D12_MIN_DEPTH,
                D3D12_ROOT_SIGNATURE_DESC,
                D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT,
                D3D12_VERTEX_BUFFER_VIEW, D3D12_VIEWPORT, D3D_ROOT_SIGNATURE_VERSION_1,
            },
            Dxgi::{
                Common::{DXGI_FORMAT_R8G8B8A8_UNORM, DXGI_SAMPLE_DESC},
                CreateDXGIFactory2, IDXGIAdapter1, IDXGIFactory4, IDXGISwapChain3, IDXGISwapChain4,
                DXGI_ADAPTER_FLAG, DXGI_ADAPTER_FLAG_NONE, DXGI_ADAPTER_FLAG_SOFTWARE,
                DXGI_CREATE_FACTORY_DEBUG, DXGI_MWA_NO_ALT_ENTER, DXGI_SWAP_CHAIN_DESC1,
                DXGI_SWAP_EFFECT_FLIP_DISCARD, DXGI_USAGE_RENDER_TARGET_OUTPUT,
            },
        },
    },
};

pub use crate::{
    error::{Error, Win32Error},
    win_error,
};
pub type Result<T> = std::result::Result<T, crate::error::Error>;

const NUM_RENDERTARGETS: usize = 2;
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

            if (desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE) != DXGI_ADAPTER_FLAG_NONE {
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
struct Resources {
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

        // TODO: Geert continue implementation from here...
        let pso = Resources::create_pipeline_state(&renderer.device, &root_signature)?;

        todo!()

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
                    signature.GetBufferPointer(),
                    signature.GetBufferSize(),
                )
                .map_err(|e| Error::Win32Error(win_error!(e)))
        }
    }

    fn create_pipeline_state(device: &ID3D12Device, root_signature: &ID3D12RootSignature) -> Result<ID3D12PipelineState> {
        todo!()
    }

}

#[derive(Debug)]
struct Camera {
    pos: Vec3f,
    dir: Vec3f,
}

struct Vertex {
    pos: Vec3f,
    col: Vec4f,
}

#[derive(Debug)]
struct Vec3f(f32, f32, f32);

#[derive(Debug)]
struct Vec4f(f32, f32, f32, f32);

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
}
