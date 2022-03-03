/// SIMD enabled linear algebra library for 3D Graphics.
/// API design largely taken over by the zmath library:
/// https://github.com/michal-z/zig-gamedev/blob/main/libs/common/zmath.zig
/// Implementation largely based on the glam-rs Rust library:
/// https://github.com/bitshifter/glam-rs

pub struct F32x4 {
    e0: f32,
    e1: f32,
    e2: f32,
    e3: f32,
}
pub struct F32x8 {
    e0: f32,
    e1: f32,
    e2: f32,
    e3: f32,
    e4: f32,
    e5: f32,
    e6: f32,
    e7: f32,
}
pub struct F32x16 {
    e0: f32,
    e1: f32,
    e2: f32,
    e3: f32,
    e4: f32,
    e5: f32,
    e6: f32,
    e7: f32,
    e8: f32,
    e9: f32,
    e10: f32,
    e11: f32,
    e12: f32,
    e13: f32,
    e14: f32,
    e15: f32,
}

pub struct U32x4 {
    e0: u32,
    e1: u32,
    e2: u32,
    e3: u32,
}
pub struct U32x8 {
    e0: u32,
    e1: u32,
    e2: u32,
    e3: u32,
    e4: u32,
    e5: u32,
    e6: u32,
    e7: u32,
}
pub struct U32x16 {
    e0: u32,
    e1: u32,
    e2: u32,
    e3: u32,
    e4: u32,
    e5: u32,
    e6: u32,
    e7: u32,
    e8: u32,
    e9: u32,
    e10: u32,
    e11: u32,
    e12: u32,
    e13: u32,
    e14: u32,
    e15: u32,
}

impl F32x4 {
    pub fn f32x4(
        e0: f32,
        e1: f32, 
        e2: f32, 
        e3: f32) -> Self {
        Self { e0, e1, e2, e3 }
    }

    pub fn f32x4s(e0: f32) -> Self {
        Self {
            e0: e0,
            e1: e0,
            e2: e0,
            e3: e0,
        }
    }
}

impl F32x8 {
    pub fn f32x8(
        e0: f32,
        e1: f32,
        e2: f32, 
        e3: f32, 
        e4: f32, 
        e5: f32, 
        e6: f32, 
        e7: f32) -> Self {
        Self {
            e0,
            e1,
            e2,
            e3,
            e4,
            e5,
            e6,
            e7,
        }
    }

    pub fn f32x8s(e0: f32) -> F32x8 {
        Self {
            e0: e0,
            e1: e0,
            e2: e0,
            e3: e0,
            e4: e0,
            e5: e0,
            e6: e0,
            e7: e0,
        }
    }
}

impl F32x16 {
    pub fn f32x16(
        e0: f32,
        e1: f32,
        e2: f32,
        e3: f32,
        e4: f32,
        e5: f32,
        e6: f32,
        e7: f32,
        e8: f32,
        e9: f32,
        e10: f32,
        e11: f32,
        e12: f32,
        e13: f32,
        e14: f32,
        e15: f32,
    ) -> Self {
        Self {
            e0,
            e1,
            e2,
            e3,
            e4,
            e5,
            e6,
            e7,
            e8,
            e9,
            e10,
            e11,
            e12,
            e13,
            e14,
            e15,
        }
    }

    pub fn f32x16s(e0: f32) -> F32x16 {
        Self {
            e0: e0,
            e1: e0,
            e2: e0,
            e3: e0,
            e4: e0,
            e5: e0,
            e6: e0,
            e7: e0,
            e8: e0,
            e9: e0,
            e10: e0,
            e11: e0,
            e12: e0,
            e13: e0,
            e14: e0,
            e15: e0,
        }
    }
}

impl U32x4 {
    pub fn u32x4(
        e0: u32,
        e1: u32, 
        e2: u32, 
        e3: u32) -> Self {
        Self { e0, e1, e2, e3 }
    }

    pub fn u32x4s(e0: u32) -> Self {
        Self {
            e0: e0,
            e1: e0,
            e2: e0,
            e3: e0,
        }
    }
}

impl U32x8 {
    pub fn u32x8(
        e0: u32,
        e1: u32, 
        e2: u32, 
        e3: u32, 
        e4: u32, 
        e5: u32, 
        e6: u32, 
        e7: u32) -> Self {
        Self {
            e0,
            e1,
            e2,
            e3,
            e4,
            e5,
            e6,
            e7,
        }
    }

    pub fn u32x8s(e0: u32) -> Self {
        Self {
            e0: e0,
            e1: e0,
            e2: e0,
            e3: e0,
            e4: e0,
            e5: e0,
            e6: e0,
            e7: e0,
        }
    }
}

impl U32x16 {
    pub fn u32x16(
        e0: u32,
        e1: u32,
        e2: u32,
        e3: u32,
        e4: u32,
        e5: u32,
        e6: u32,
        e7: u32,
        e8: u32,
        e9: u32,
        e10: u32,
        e11: u32,
        e12: u32,
        e13: u32,
        e14: u32,
        e15: u32,
    ) -> Self {
        Self {
            e0,
            e1,
            e2,
            e3,
            e4,
            e5,
            e6,
            e7,
            e8,
            e9,
            e10,
            e11,
            e12,
            e13,
            e14,
            e15,
        }
    }

    pub fn u32x16s(e0: u32) -> Self {
        Self {
            e0: e0,
            e1: e0,
            e2: e0,
            e3: e0,
            e4: e0,
            e5: e0,
            e6: e0,
            e7: e0,
            e8: e0,
            e9: e0,
            e10: e0,
            e11: e0,
            e12: e0,
            e13: e0,
            e14: e0,
            e15: e0,
        }
    }
}
