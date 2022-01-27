use std::ffi::c_void;
use windows::Win32::{
    Foundation::HWND,
    Graphics::{
        Direct3D12::{
            ID3D12CommandAllocator, ID3D12CommandQueue, ID3D12Debug, ID3D12DescriptorHeap,
            ID3D12Device, ID3D12Fence, ID3D12GraphicsCommandList, ID3D12PipelineState,
            ID3D12Resource, ID3D12RootSignature, D3D12_VERTEX_BUFFER_VIEW,
        },
        Dxgi::{IDXGIAdapter1, IDXGIFactory4, IDXGISwapChain4},
    },
};

use crate::error::Error;
pub type Result<T> = std::result::Result<T, crate::error::Error>;

const NUM_RENDERTARGETS: usize = 2;

#[derive(Debug)]
pub struct D3D12Renderer {
    debug: ID3D12Debug,
    factory: IDXGIFactory4,
    adapter: IDXGIAdapter1,
    device: ID3D12Device,
    queue: ID3D12CommandQueue,
    swapchain: IDXGISwapChain4,
    rtvDescriptorHeap: ID3D12DescriptorHeap,
    commandAllocator: ID3D12CommandAllocator,
    rootSignature: ID3D12RootSignature,
    pipeline: ID3D12PipelineState,
    cmdlist: ID3D12GraphicsCommandList,
}

impl Default for D3D12Renderer {
    fn default() -> Self {
        unsafe { ::core::mem::zeroed() }
    }
}

#[derive(Debug)]
pub struct D3D12Resources {
    targets: [ID3D12Resource; NUM_RENDERTARGETS],
    vertexBuffer: ID3D12Resource,
    vertexBufferView: D3D12_VERTEX_BUFFER_VIEW,
    fence: ID3D12Fence,
    fenceEvent: *mut c_void,
    fenceValue: u32,
    frameIndex: usize,
    camera: Camera,
}

impl Default for D3D12Resources {
    fn default() -> Self {
        unsafe { ::core::mem::zeroed() }
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
    resources: D3D12Resources,
}

impl D3D12 {
    pub fn new(win32_window_handle: HWND) -> Result<Self> {
        let renderer = D3D12Renderer::default();
        let resources = D3D12Resources::default();
        Ok(Self {
            renderer,
            resources,
        })
    }
}
