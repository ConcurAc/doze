use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use bytemuck::Pod;

pub mod primitive;
pub use primitive::{Primitive, PrimitiveCast};

pub trait AddSelf: Sized + Add<Self, Output = Self> + AddAssign<Self> {}
impl<T: Add<Self, Output = Self> + AddAssign<Self>> AddSelf for T {}

pub trait SubSelf: Sized + Sub<Self, Output = Self> + SubAssign<Self> {}
impl<T: Sub<Self, Output = Self> + SubAssign<Self>> SubSelf for T {}

pub trait MulSelf: Sized + Mul<Self, Output = Self> + MulAssign<Self> {}
impl<T: Mul<Self, Output = Self> + MulAssign<Self>> MulSelf for T {}

pub trait DivSelf: Sized + Div<Self, Output = Self> + DivAssign<Self> {}
impl<T: Div<Self, Output = Self> + DivAssign<Self>> DivSelf for T {}

pub trait OpsSelf: AddSelf + SubSelf + MulSelf + DivSelf {}
impl<T: AddSelf + SubSelf + MulSelf + DivSelf> OpsSelf for T {}

pub trait Sample: Primitive + OpsSelf + Pod {}
impl<T: Primitive + OpsSelf + Pod> Sample for T {}
