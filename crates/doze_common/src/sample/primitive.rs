macro_rules! primitive {
    ($($ty:ty)+) => {
        pub trait Primitive: $(PrimitiveCast<$ty> +)+ Sized {
            #[inline]
            fn as_primitive<P: PrimitiveCast<Self>>(self) -> P {
                P::from_primitive(self)
            }
        }
        impl<T: $(PrimitiveCast<$ty> +)+ Sized> Primitive for T {}
    };
}

primitive!(u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize f32 f64);

pub trait PrimitiveCast<T> {
    fn from_primitive(value: T) -> Self;
}

macro_rules! primitive_cast {
    ($as:ty=> $($from:ty$(|)?)+) => {
        $(
        impl PrimitiveCast<$from> for $as {
            #[inline]
            fn from_primitive(value: $from) -> Self {
                value as $as
            }
        }
        )+
    };
}

primitive_cast!(
    u8 =>
        u8 | u16 | u32 | u64 | u128 | usize |
        i8 | i16 | i32 | i64 | i128 | isize |
        f32 | f64
);
primitive_cast!(
    u16 =>
        u8 | u16 | u32 | u64 | u128 | usize |
        i8 | i16 | i32 | i64 | i128 | isize |
        f32 | f64
);
primitive_cast!(
    u32 =>
        u8 | u16 | u32 | u64 | u128 | usize |
        i8 | i16 | i32 | i64 | i128 | isize |
        f32 | f64
);
primitive_cast!(
    u64 =>
        u8 | u16 | u32 | u64 | u128 | usize |
        i8 | i16 | i32 | i64 | i128 | isize |
        f32 | f64
);
primitive_cast!(
    u128 =>
        u8 | u16 | u32 | u64 | u128 | usize |
        i8 | i16 | i32 | i64 | i128 | isize |
        f32 | f64
);
primitive_cast!(
    usize =>
        u8 | u16 | u32 | u64 | u128 | usize |
        i8 | i16 | i32 | i64 | i128 | isize |
        f32 | f64
);
primitive_cast!(
    i8 =>
        u8 | u16 | u32 | u64 | u128 | usize |
        i8 | i16 | i32 | i64 | i128 | isize |
        f32 | f64
);
primitive_cast!(
    i16 =>
        u8 | u16 | u32 | u64 | u128 | usize |
        i8 | i16 | i32 | i64 | i128 | isize |
        f32 | f64
);
primitive_cast!(
    i32 =>
        u8 | u16 | u32 | u64 | u128 | usize |
        i8 | i16 | i32 | i64 | i128 | isize |
        f32 | f64
);
primitive_cast!(
    i64 =>
        u8 | u16 | u32 | u64 | u128 | usize |
        i8 | i16 | i32 | i64 | i128 | isize |
        f32 | f64
);
primitive_cast!(
    i128 =>
        u8 | u16 | u32 | u64 | u128 | usize |
        i8 | i16 | i32 | i64 | i128 | isize |
        f32 | f64
);
primitive_cast!(
    isize =>
        u8 | u16 | u32 | u64 | u128 | usize |
        i8 | i16 | i32 | i64 | i128 | isize |
        f32 | f64
);
primitive_cast!(
    f32 =>
        u8 | u16 | u32 | u64 | u128 | usize |
        i8 | i16 | i32 | i64 | i128 | isize |
        f32 | f64
);
primitive_cast!(
    f64 =>
        u8 | u16 | u32 | u64 | u128 | usize |
        i8 | i16 | i32 | i64 | i128 | isize |
        f32 | f64
);
