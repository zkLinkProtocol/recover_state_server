macro_rules! basic_type {
    ($(#[$attr:meta])* $name:ident, $type:ty) => {
        $(#[$attr])*
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash, PartialOrd, Ord, Default
        )]
        pub struct $name(pub $type);

        impl Deref for $name {
            type Target = $type;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl FromStr for $name {
            type Err = ParseIntError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                let value = s.parse::<$type>()?;
                Ok(Self(value))
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl Add<$type> for $name {
            type Output = Self;

            fn add(self, other: $type) -> Self {
                Self(self.0 + other)
            }
        }

        impl Sub<$type> for $name {
            type Output = Self;

            fn sub(self, other: $type) -> Self {
                Self(self.0 - other)
            }
        }

        impl From<u8> for $name {
            fn from(t: u8) -> Self {
                Self(t as $type)
            }
        }
        impl From<u16> for $name {
            fn from(t: u16) -> Self {
                Self(t as $type)
            }
        }
        impl From<u32> for $name {
            fn from(t: u32) -> Self {
                Self(t as $type)
            }
        }
        impl From<u64> for $name {
            fn from(t: u64) -> Self {
                Self(t as $type)
            }
        }
        impl From<i8> for $name {
            fn from(t: i8) -> Self {
                Self(t as $type)
            }
        }
        impl From<i16> for $name {
            fn from(t: i16) -> Self {
                Self(t as $type)
            }
        }
        impl From<i32> for $name {
            fn from(t: i32) -> Self {
                Self(t as $type)
            }
        }
        impl From<i64> for $name {
            fn from(t: i64) -> Self {
                Self(t as $type)
            }
        }
        impl Into<u8> for $name {
            fn into(self) -> u8 {
                *self as u8
            }
        }
        impl Into<u16> for $name {
            fn into(self) -> u16 {
                *self as u16
            }
        }
        impl Into<u32> for $name {
            fn into(self) -> u32 {
                *self as u32
            }
        }
        impl Into<u64> for $name {
            fn into(self) -> u64 {
                *self as u64
            }
        }
        impl Into<i8> for $name {
            fn into(self) -> i8 {
                *self as i8
            }
        }
        impl Into<i16> for $name {
            fn into(self) -> i16 {
                *self as i16
            }
        }
        impl Into<i32> for $name {
            fn into(self) -> i32 {
                *self as i32
            }
        }
        impl Into<i64> for $name {
            fn into(self) -> i64 {
                *self as i64
            }
        }
    };
}
