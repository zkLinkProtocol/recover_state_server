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
        impl From<$name> for u8  {
            fn from(t: $name) -> u8 {
                *t as u8
            }
        }
        impl From<$name> for u16  {
            fn from(t: $name) -> u16 {
                *t as u16
            }
        }
        impl From<$name> for u32  {
            fn from(t: $name) -> u32 {
                *t as u32
            }
        }
        impl From<$name> for u64  {
            fn from(t: $name) -> u64 {
                *t as u64
            }
        }
        impl From<$name> for i8  {
            fn from(t: $name) -> i8 {
                *t as i8
            }
        }
        impl From<$name> for i16  {
            fn from(t: $name) -> i16 {
                *t as i16
            }
        }
        impl From<$name> for i32  {
            fn from(t: $name) -> i32 {
                *t as i32
            }
        }
        impl From<$name> for i64  {
            fn from(t: $name) -> i64 {
                *t as i64
            }
        }
    };
}
