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

macro_rules! make_vec {
    ($name:ident, $size:expr, ($($members:ident), *), $last_member:ident) => {
        #[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash, Default)]
        #[repr(C)]
        pub struct $name<T> {
            $(pub $members: T,)*
            pub $last_member: T
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
    };
}

make_vec!(Vec2, 2, (x), y);
make_vec!(Vec3, 3, (x, y), z);
make_vec!(Vec4, 4, (x, y, z), w);
