//! vector and matrix types designed off of GLSL's vector and matrix types

use std::fmt;
use std::mem;
use std::ops::*;

pub trait Mappable<I, O, F: FnMut(I) -> O> {
    type Output;
    fn map(self, f: F) -> Self::Output;
}

pub trait VecSizeTrait: Default {
    fn len(&self) -> usize;
}

pub trait Vec {
    type Element;
    type SizeTrait: VecSizeTrait;
    fn len(&self) -> usize {
        <Self::SizeTrait as Default>::default().len()
    }
}

pub trait VecRef {
    type Element;
    type ElementRef;
    type SizeTrait: VecSizeTrait;
}

impl<T, S: VecSizeTrait> VecRef for Vec<Element = T, SizeTrait = S> {
    type Element = T;
    type ElementRef = T;
    type SizeTrait = S;
}

impl<'a, T, S: VecSizeTrait> VecRef for &'a Vec<Element = T, SizeTrait = S> {
    type Element = T;
    type ElementRef = &'a T;
    type SizeTrait = S;
}

impl<'a, T, S: VecSizeTrait> VecRef for &'a mut Vec<Element = T, SizeTrait = S> {
    type Element = T;
    type ElementRef = &'a mut T;
    type SizeTrait = S;
}

pub trait Reducible<T> {
    fn reduce<F>(self, f: F) -> T
    where
        Self: Sized,
        F: FnMut(T, T) -> T;
}

#[derive(Copy, Clone, Debug)]
enum VecIterImpl<T> {
    None,
    Scalar(T),
    Vec2(Vec2<T>),
    Vec3(Vec3<T>),
    Vec4(Vec4<T>),
}

#[derive(Copy, Clone, Debug)]
pub struct VecIter<T>(VecIterImpl<T>);

impl<T> Iterator for VecIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        match mem::replace(&mut self.0, VecIterImpl::None) {
            VecIterImpl::None => None,
            VecIterImpl::Scalar(v) => Some(v),
            VecIterImpl::Vec2(v) => {
                self.0 = VecIterImpl::Scalar(v.y);
                Some(v.x)
            }
            VecIterImpl::Vec3(v) => {
                self.0 = VecIterImpl::Vec2(Vec2::new(v.y, v.z));
                Some(v.x)
            }
            VecIterImpl::Vec4(v) => {
                self.0 = VecIterImpl::Vec3(Vec3::new(v.y, v.z, v.w));
                Some(v.x)
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.len();
        (size, Some(size))
    }

    fn count(self) -> usize {
        self.len()
    }

    fn last(mut self) -> Option<T> {
        self.next_back()
    }
}

impl<T> DoubleEndedIterator for VecIter<T> {
    fn next_back(&mut self) -> Option<T> {
        match mem::replace(&mut self.0, VecIterImpl::None) {
            VecIterImpl::None => None,
            VecIterImpl::Scalar(v) => Some(v),
            VecIterImpl::Vec2(v) => {
                self.0 = VecIterImpl::Scalar(v.x);
                Some(v.y)
            }
            VecIterImpl::Vec3(v) => {
                self.0 = VecIterImpl::Vec2(Vec2::new(v.x, v.y));
                Some(v.z)
            }
            VecIterImpl::Vec4(v) => {
                self.0 = VecIterImpl::Vec3(Vec3::new(v.x, v.y, v.z));
                Some(v.w)
            }
        }
    }
}

impl<T> ExactSizeIterator for VecIter<T> {
    fn len(&self) -> usize {
        match &self.0 {
            VecIterImpl::None => 0,
            VecIterImpl::Scalar(_) => 1,
            VecIterImpl::Vec2(_) => 2,
            VecIterImpl::Vec3(_) => 3,
            VecIterImpl::Vec4(_) => 4,
        }
    }
}

macro_rules! impl_op {
    ($vec:ident, $op:ident, $fn:ident) => {
        impl<R, T: $op<R>> $op<$vec<R>> for $vec<T> {
            type Output = $vec<T::Output>;
            fn $fn(self, rhs: $vec<R>) -> Self::Output {
                self.zip(rhs).map(|(l, r)| l.$fn(r))
            }
        }
    };
}

macro_rules! impl_op_assign {
    ($vec:ident, $op:ident, $fn:ident) => {
        impl<R, T: $op<R>> $op<$vec<R>> for $vec<T> {
            fn $fn(&mut self, rhs: $vec<R>) {
                self.as_mut().zip(rhs).map(|(l, r)| l.$fn(r));
            }
        }
    };
}

macro_rules! impl_op_unary {
    ($vec:ident, $op:ident, $fn:ident) => {
        impl<T: $op> $op for $vec<T> {
            type Output = $vec<T::Output>;
            fn $fn(self) -> Self::Output {
                self.map(|v| v.$fn())
            }
        }
    };
}

pub trait Dot<RHS = Self> {
    type Output;
    fn dot(self, rhs: RHS) -> Self::Output;
}

macro_rules! make_vec {
    ($name:ident, $size:expr, ($($members:ident), *), $last_member:ident) => {
        #[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Default)]
        #[repr(C)]
        pub struct $name<T = f32> {
            $(pub $members: T,)*
            pub $last_member: T
        }

        impl<T> Into<[T; $size]> for $name<T> {
            fn into(self) -> [T; $size] {
                [$(self.$members,)* self.$last_member]
            }
        }

        impl<'a, T: Clone> Into<[T; $size]> for &'a $name<T> {
            fn into(self) -> [T; $size] {
                [$(self.$members.clone(),)* self.$last_member.clone()]
            }
        }

        impl<'a, T: Clone> Into<[T; $size]> for &'a mut $name<T> {
            fn into(self) -> [T; $size] {
                [$(self.$members.clone(),)* self.$last_member.clone()]
            }
        }

        impl VecSizeTrait for $name<()> {
            fn len(&self) -> usize {
                $size
            }
        }

        impl<T> Vec for $name<T> {
            type Element = T;
            type SizeTrait = $name<()>;
        }

        impl<T> IntoIterator for $name<T> {
            type Item = T;
            type IntoIter = VecIter<T>;
            fn into_iter(self) -> VecIter<T> {
                VecIter(VecIterImpl::$name(self))
            }
        }

        impl<'a, T> IntoIterator for &'a $name<T> {
            type Item = &'a T;
            type IntoIter = VecIter<&'a T>;
            fn into_iter(self) -> VecIter<&'a T> {
                self.map(|v| v).into_iter()
            }
        }

        impl<'a, T> IntoIterator for &'a mut $name<T> {
            type Item = &'a mut T;
            type IntoIter = VecIter<&'a mut T>;
            fn into_iter(self) -> VecIter<&'a mut T> {
                self.map(|v| v).into_iter()
            }
        }

        impl<T: fmt::Debug> fmt::Debug for $name<T> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.debug_tuple(stringify!($name))
                $(.field(&self.$members))*
                .field(&self.$last_member)
                .finish()
            }
        }

        impl<T> $name<T> {
            pub fn new($($members: T,)* $last_member: T) -> Self {
                Self {
                    $($members: $members,)*
                    $last_member: $last_member
                }
            }
            pub fn zip<R>(self, rhs: $name<R>) -> $name<(T, R)> {
                $name {
                    $($members: (self.$members, rhs.$members),)*
                    $last_member: (self.$last_member, rhs.$last_member)
                }
            }
            pub fn as_ref(&self) -> $name<&T> {
                $name {
                    $($members: &self.$members,)*
                    $last_member: &self.$last_member
                }
            }
            pub fn as_mut(&mut self) -> $name<&mut T> {
                $name {
                    $($members: &mut self.$members,)*
                    $last_member: &mut self.$last_member
                }
            }
            pub fn iter(&self) -> VecIter<&T> {
                self.into_iter()
            }
            pub fn iter_mut(&mut self) -> VecIter<&mut T> {
                self.into_iter()
            }
        }

        impl<T, O, F: FnMut(T) -> O> Mappable<T, O, F> for $name<T> {
            type Output = $name<O>;
            fn map(self, mut f: F) -> Self::Output {
                $name {
                    $($members: f(self.$members),)*
                    $last_member: f(self.$last_member)
                }
            }
        }

        impl<'a, T, O, F: FnMut(&'a mut T) -> O> Mappable<&'a mut T, O, F> for &'a mut $name<T> {
            type Output = $name<O>;
            fn map(self, mut f: F) -> Self::Output {
                $name {
                    $($members: f(&mut self.$members),)*
                    $last_member: f(&mut self.$last_member)
                }
            }
        }

        impl<'a, T, O, F: FnMut(&'a T) -> O> Mappable<&'a T, O, F> for &'a $name<T> {
            type Output = $name<O>;
            fn map(self, mut f: F) -> Self::Output {
                $name {
                    $($members: f(&self.$members),)*
                    $last_member: f(&self.$last_member)
                }
            }
        }

        impl<T> Reducible<T> for $name<T> {
            fn reduce<F>(self, f: F) -> T
            where
                Self: Sized,
                F: FnMut(T, T) -> T
            {
                let mut iter = self.into_iter();
                let first = iter.next().unwrap();
                iter.fold(first, f)
            }
        }

        impl<T: Clone> $name<T> {
            pub fn splat(value: T) -> Self {
                Self {
                    $($members: value.clone(),)*
                    $last_member: value
                }
            }
        }

        impl<T> Index<usize> for $name<T> {
            type Output = T;
            fn index(&self, index: usize) -> &T {
                [$(&self.$members,)* &self.$last_member][index]
            }
        }

        impl<T> IndexMut<usize> for $name<T> {
            fn index_mut(&mut self, index: usize) -> &mut T {
                [$(&mut self.$members,)* &mut self.$last_member][index]
            }
        }

        impl_op!($name, Add, add);
        impl_op!($name, BitAnd, bitand);
        impl_op!($name, BitOr, bitor);
        impl_op!($name, BitXor, bitxor);
        impl_op!($name, Div, div);
        impl_op!($name, Mul, mul);
        impl_op!($name, Rem, rem);
        impl_op!($name, Shl, shl);
        impl_op!($name, Shr, shr);
        impl_op!($name, Sub, sub);

        impl_op_assign!($name, AddAssign, add_assign);
        impl_op_assign!($name, BitAndAssign, bitand_assign);
        impl_op_assign!($name, BitOrAssign, bitor_assign);
        impl_op_assign!($name, BitXorAssign, bitxor_assign);
        impl_op_assign!($name, DivAssign, div_assign);
        impl_op_assign!($name, MulAssign, mul_assign);
        impl_op_assign!($name, RemAssign, rem_assign);
        impl_op_assign!($name, ShlAssign, shl_assign);
        impl_op_assign!($name, ShrAssign, shr_assign);
        impl_op_assign!($name, SubAssign, sub_assign);

        impl_op_unary!($name, Neg, neg);
        impl_op_unary!($name, Not, not);

        impl<R, T: Mul<R>> Dot<$name<R>> for $name<T> where T::Output: Add<Output = T::Output> {
            type Output = <T::Output as Add>::Output;
            fn dot(self, rhs: $name<R>) -> Self::Output {
                self.mul(rhs).reduce(|a, b| a.add(b))
            }
        }

        impl $name<f32> {
            pub fn abs(self) -> f32 {
                self.dot(self).sqrt()
            }
            pub fn normalize(self) -> Option<Self> {
                let abs = self.abs();
                if abs == 0.0 {
                    None
                } else {
                    Some(self / Self::splat(abs))
                }
            }
        }

        impl $name<f64> {
            pub fn abs(self) -> f64 {
                self.dot(self).sqrt()
            }
            pub fn normalize(self) -> Option<Self> {
                let abs = self.abs();
                if abs == 0.0 {
                    None
                } else {
                    Some(self / Self::splat(abs))
                }
            }
        }
    };
}

make_vec!(Vec2, 2, (x), y);
make_vec!(Vec3, 3, (x, y), z);
make_vec!(Vec4, 4, (x, y, z), w);

pub trait Cross<RHS = Self> {
    type Output;
    fn cross(self, rhs: RHS) -> Self::Output;
}

impl<R: Clone, T: Mul<R> + Clone> Cross<Vec3<R>> for Vec3<T>
where
    T::Output: Sub,
{
    type Output = Vec3<<T::Output as Sub>::Output>;
    fn cross(self, rhs: Vec3<R>) -> Self::Output {
        Vec3::new(
            self.y.clone() * rhs.z.clone() - self.z.clone() * rhs.y.clone(),
            self.z * rhs.x.clone() - self.x.clone() * rhs.z,
            self.x * rhs.y - self.y * rhs.x,
        )
    }
}

/// Column-major 4x4 Matrix
#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Default)]
#[repr(C)]
pub struct Mat4<T = f32> {
    pub c0: Vec4<T>,
    pub c1: Vec4<T>,
    pub c2: Vec4<T>,
    pub c3: Vec4<T>,
}

impl<T> Mat4<T> {
    pub fn new(c0: Vec4<T>, c1: Vec4<T>, c2: Vec4<T>, c3: Vec4<T>) -> Self {
        Self {
            c0: c0,
            c1: c1,
            c2: c2,
            c3: c3,
        }
    }
    pub fn transpose(self) -> Self {
        let Self { c0, c1, c2, c3 } = self;
        Self::new(
            Vec4::new(c0.x, c1.x, c2.x, c3.x),
            Vec4::new(c0.y, c1.y, c2.y, c3.y),
            Vec4::new(c0.z, c1.z, c2.z, c3.z),
            Vec4::new(c0.w, c1.w, c2.w, c3.w),
        )
    }
}

impl<T> Into<[Vec4<T>; 4]> for Mat4<T> {
    fn into(self) -> [Vec4<T>; 4] {
        [self.c0, self.c1, self.c2, self.c3]
    }
}

impl<T> Into<[[T; 4]; 4]> for Mat4<T> {
    fn into(self) -> [[T; 4]; 4] {
        [
            self.c0.into(),
            self.c1.into(),
            self.c2.into(),
            self.c3.into(),
        ]
    }
}

impl<'a, T: Clone> Into<[Vec4<T>; 4]> for &'a Mat4<T> {
    fn into(self) -> [Vec4<T>; 4] {
        [
            self.c0.clone(),
            self.c1.clone(),
            self.c2.clone(),
            self.c3.clone(),
        ]
    }
}

impl<'a, T: Clone> Into<[[T; 4]; 4]> for &'a Mat4<T> {
    fn into(self) -> [[T; 4]; 4] {
        [
            (&self.c0).into(),
            (&self.c1).into(),
            (&self.c2).into(),
            (&self.c3).into(),
        ]
    }
}

impl<'a, T: Clone> Into<[Vec4<T>; 4]> for &'a mut Mat4<T> {
    fn into(self) -> [Vec4<T>; 4] {
        [
            self.c0.clone(),
            self.c1.clone(),
            self.c2.clone(),
            self.c3.clone(),
        ]
    }
}

impl<'a, T: Clone> Into<[[T; 4]; 4]> for &'a mut Mat4<T> {
    fn into(self) -> [[T; 4]; 4] {
        [
            (&mut self.c0).into(),
            (&mut self.c1).into(),
            (&mut self.c2).into(),
            (&mut self.c3).into(),
        ]
    }
}

impl<T: Clone> Mat4<T> {
    pub fn splat(value: T) -> Self {
        Self::new(
            Vec4::splat(value.clone()),
            Vec4::splat(value.clone()),
            Vec4::splat(value.clone()),
            Vec4::splat(value),
        )
    }
}

pub trait Zero: Add<Output = Self> + Sized {
    fn zero() -> Self;
}

pub trait One: Mul<Output = Self> + Sized {
    fn one() -> Self;
}

macro_rules! impl_zero_one {
    ($($types:ty)*) => {
        $(
            impl Zero for $types {
                fn zero() -> $types {
                    0 as $types
                }
            }

            impl One for $types {
                fn one() -> $types {
                    1 as $types
                }
            }
        )*
    };
}

impl_zero_one!(u8 i8 u16 i16 u32 i32 u64 i64 f32 f64);

impl<T: Zero> Mat4<T> {
    pub fn zero() -> Self {
        let zero = || Zero::zero();
        Self::new(
            Vec4::new(zero(), zero(), zero(), zero()),
            Vec4::new(zero(), zero(), zero(), zero()),
            Vec4::new(zero(), zero(), zero(), zero()),
            Vec4::new(zero(), zero(), zero(), zero()),
        )
    }
}

impl<T: Zero + One> Mat4<T> {
    pub fn identity() -> Self {
        let zero = || Zero::zero();
        let one = || One::one();
        Self::new(
            Vec4::new(one(), zero(), zero(), zero()),
            Vec4::new(zero(), one(), zero(), zero()),
            Vec4::new(zero(), zero(), one(), zero()),
            Vec4::new(zero(), zero(), zero(), one()),
        )
    }
    pub fn translation(amount: Vec3<T>) -> Self {
        let zero = || Zero::zero();
        let one = || One::one();
        Self::new(
            Vec4::new(one(), zero(), zero(), zero()),
            Vec4::new(zero(), one(), zero(), zero()),
            Vec4::new(zero(), zero(), one(), zero()),
            Vec4::new(amount.x, amount.y, amount.z, one()),
        )
    }
    pub fn scaling(amount: Vec3<T>) -> Self {
        let zero = || Zero::zero();
        let one = || One::one();
        Self::new(
            Vec4::new(amount.x, zero(), zero(), zero()),
            Vec4::new(zero(), amount.y, zero(), zero()),
            Vec4::new(zero(), zero(), amount.z, zero()),
            Vec4::new(zero(), zero(), zero(), one()),
        )
    }
}

impl<T: Zero + One + Clone> Mat4<T> {
    pub fn translate(self, amount: Vec3<T>) -> Self {
        self * Mat4::translation(amount)
    }
    pub fn scale(self, amount: Vec3<T>) -> Self {
        self * Mat4::scaling(amount)
    }
}

impl<T: Zero + One + Clone + Sub<Output = T>> Mat4<T> {
    pub fn rotation_sin_cos(sin: T, cos: T, axis_normalized: Vec3<T>) -> Self {
        let zero = || Zero::zero();
        let one = || One::one();
        let x_x = axis_normalized.x.clone() * axis_normalized.x.clone();
        let x_y = axis_normalized.x.clone() * axis_normalized.y.clone();
        let x_z = axis_normalized.x.clone() * axis_normalized.z.clone();
        let y_y = axis_normalized.y.clone() * axis_normalized.y.clone();
        let y_z = axis_normalized.y.clone() * axis_normalized.z.clone();
        let z_z = axis_normalized.z.clone() * axis_normalized.z.clone();
        let versine: T = one() - cos.clone();
        Mat4::new(
            Vec4::new(
                x_x * versine.clone() + cos.clone(),
                x_y.clone() * versine.clone() + axis_normalized.z.clone() * sin.clone(),
                x_z.clone() * versine.clone() - axis_normalized.y.clone() * sin.clone(),
                zero(),
            ),
            Vec4::new(
                x_y.clone() * versine.clone() - axis_normalized.z * sin.clone(),
                y_y * versine.clone() + cos.clone(),
                y_z.clone() * versine.clone() + axis_normalized.x.clone() * sin.clone(),
                zero(),
            ),
            Vec4::new(
                x_z.clone() * versine.clone() + axis_normalized.y * sin.clone(),
                y_z.clone() * versine.clone() - axis_normalized.x * sin,
                z_z * versine + cos,
                zero(),
            ),
            Vec4::new(zero(), zero(), zero(), one()),
        )
    }
    pub fn rotate_sin_cos(self, sin: T, cos: T, axis_normalized: Vec3<T>) -> Self {
        self * Self::rotation_sin_cos(sin, cos, axis_normalized)
    }
}

macro_rules! float_mat4_methods {
    ($T:ty) => {
        impl Mat4<$T> {
            pub fn rotation(angle_in_radians: $T, axis: Vec3<$T>) -> Self {
                Mat4::rotation_sin_cos(
                    angle_in_radians.sin(),
                    angle_in_radians.cos(),
                    axis.normalize().unwrap(),
                )
            }
            pub fn rotate(self, angle_in_radians: $T, axis: Vec3<$T>) -> Self {
                self * Self::rotation(angle_in_radians, axis)
            }
            pub fn orthographic_projection(
                left: $T,
                right: $T,
                bottom: $T,
                top: $T,
                near: $T,
                far: $T,
            ) -> Self {
                Mat4::new(
                    Vec4::new(2.0 / (right - left), 0.0, 0.0, 0.0),
                    Vec4::new(0.0, 2.0 / (top - bottom), 0.0, 0.0),
                    Vec4::new(0.0, 0.0, -2.0 / (far - near), 0.0),
                    Vec4::new(
                        -(right + left) / (right - left),
                        -(top + bottom) / (top - bottom),
                        -(far + near) / (far - near),
                        1.0,
                    ),
                )
            }
            pub fn orthographic_project(
                self,
                left: $T,
                right: $T,
                bottom: $T,
                top: $T,
                near: $T,
                far: $T,
            ) -> Self {
                self * Self::orthographic_projection(left, right, bottom, top, near, far)
            }
            pub fn perspective_projection(
                left: $T,
                right: $T,
                bottom: $T,
                top: $T,
                near: $T,
                far: $T,
            ) -> Self {
                Mat4::new(
                    Vec4::new(2.0 * near / (right - left), 0.0, 0.0, 0.0),
                    Vec4::new(0.0, 2.0 * near / (top - bottom), 0.0, 0.0),
                    Vec4::new(
                        (right + left) / (right - left),
                        (top + bottom) / (top - bottom),
                        -(far + near) / (far - near),
                        -1.0,
                    ),
                    Vec4::new(0.0, 0.0, -2.0 * far * near / (far - near), 0.0),
                )
            }
            pub fn perspective_project(
                self,
                left: $T,
                right: $T,
                bottom: $T,
                top: $T,
                near: $T,
                far: $T,
            ) -> Self {
                self * Self::perspective_projection(left, right, bottom, top, near, far)
            }
        }
    };
}

float_mat4_methods!(f32);
float_mat4_methods!(f64);

impl<T> Index<usize> for Mat4<T> {
    type Output = Vec4<T>;
    fn index(&self, index: usize) -> &Vec4<T> {
        [&self.c0, &self.c1, &self.c2, &self.c3][index]
    }
}

impl<T> IndexMut<usize> for Mat4<T> {
    fn index_mut(&mut self, index: usize) -> &mut Vec4<T> {
        [&mut self.c0, &mut self.c1, &mut self.c2, &mut self.c3][index]
    }
}

impl<L, R, O> Mul<Mat4<R>> for Mat4<L>
where
    L: Mul<R, Output = O> + Clone,
    R: Clone,
    O: Add<Output = O>,
{
    type Output = Mat4<O>;
    fn mul(self, r: Mat4<R>) -> Mat4<O> {
        let l = self.transpose();
        Mat4::new(
            Vec4::new(
                l.c0.clone().dot(r.c0.clone()),
                l.c1.clone().dot(r.c0.clone()),
                l.c2.clone().dot(r.c0.clone()),
                l.c3.clone().dot(r.c0),
            ),
            Vec4::new(
                l.c0.clone().dot(r.c1.clone()),
                l.c1.clone().dot(r.c1.clone()),
                l.c2.clone().dot(r.c1.clone()),
                l.c3.clone().dot(r.c1),
            ),
            Vec4::new(
                l.c0.clone().dot(r.c2.clone()),
                l.c1.clone().dot(r.c2.clone()),
                l.c2.clone().dot(r.c2.clone()),
                l.c3.clone().dot(r.c2),
            ),
            Vec4::new(
                l.c0.dot(r.c3.clone()),
                l.c1.dot(r.c3.clone()),
                l.c2.dot(r.c3.clone()),
                l.c3.dot(r.c3),
            ),
        )
    }
}

impl<L, R, O> Mul<Mat4<R>> for Vec4<L>
where
    L: Mul<R, Output = O> + Clone,
    R: Clone,
    O: Add<Output = O>,
{
    type Output = Vec4<O>;
    fn mul(self, rhs: Mat4<R>) -> Vec4<O> {
        Vec4::new(
            self.clone().dot(rhs.c0),
            self.clone().dot(rhs.c1),
            self.clone().dot(rhs.c2),
            self.dot(rhs.c3),
        )
    }
}

impl<L, R, O> Mul<Vec4<R>> for Mat4<L>
where
    L: Mul<R, Output = O> + Clone,
    R: Clone,
    O: Add<Output = O>,
{
    type Output = Vec4<O>;
    fn mul(self, rhs: Vec4<R>) -> Vec4<O> {
        let l = self.transpose();
        Vec4::new(
            l.c0.dot(rhs.clone()),
            l.c1.dot(rhs.clone()),
            l.c2.dot(rhs.clone()),
            l.c3.dot(rhs.clone()),
        )
    }
}

impl<I, O, F: FnMut(I) -> O> Mappable<I, O, F> for Mat4<I> {
    type Output = Mat4<O>;
    fn map(self, mut f: F) -> Mat4<O> {
        Mat4::new(
            self.c0.map(&mut f),
            self.c1.map(&mut f),
            self.c2.map(&mut f),
            self.c3.map(f),
        )
    }
}
