#[macro_export]
macro_rules! define_event {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident = $val:literal
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[non_exhaustive]
        #[repr(u8)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize))]
        #[cfg_attr(feature = "defmt", derive(defmt::Format))]
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant = $val,
            )*
        }

        impl TryFrom<u8> for $name {
            type Error = &'static str;

            #[inline]
            fn try_from(value: u8) -> Result<Self, Self::Error> {
                match value {
                    $( $val => Ok(Self::$variant), )*
                    _ => Err(concat!("unknown ", stringify!($name), " code")),
                }
            }
        }

        impl From<$name> for u8 {
            #[inline]
            fn from(event: $name) -> Self {
                event as u8
            }
        }
    };
}
