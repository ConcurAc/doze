pub mod primitive;

use primitive::Primitive;

use bytemuck::Pod;

pub trait Sample: Primitive + Pod {}

impl<T: Primitive + Pod> Sample for T {}
