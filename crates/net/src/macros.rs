//! Macros for protocol, event, and packet definitions.

/// Defines an event enum with automatic `TryFrom<u8>`, `From<Event> for u8`,
/// and standard formatting/serialization derive attributes.
///
/// # Example
/// ```rust,ignore
/// define_event! {
///     pub enum ServerEvent {
///         HandshakeAck = 0x81,
///         GameUpdate = 0xc0,
///     }
/// }
/// ```
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

/// Defines a typed message enum matching an event code and its typed payload.
///
/// Generates:
/// - A typed enum containing payloads (or unit variants for signals without payloads)
/// - An event accessor `event(&self) -> EventType`
/// - A decode helper `decode(event: EventType, data: &[u8]) -> Result<Self, &'static str>`
///
/// # Example
/// ```rust,ignore
/// define_packets! {
///     pub enum ClientMessage(ClientEvent) {
///         HandshakeRequest(Handshake) = ClientEvent::HandshakeRequest,
///         GameStart = ClientEvent::GameStart,
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_packets {
    (
        $(#[$meta:meta])*
        $vis:vis enum $msg_enum:ident($event_ty:ident) {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident $( ( $payload_ty:ty ) )? = $event_val:path
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq)]
        #[cfg_attr(feature = "defmt", derive(defmt::Format))]
        $vis enum $msg_enum {
            $(
                $(#[$variant_meta])*
                $variant $( ( $payload_ty ) )?,
            )*
        }

        impl $msg_enum {
            /// Returns the corresponding event code for this packet.
            #[inline]
            pub const fn event(&self) -> $event_ty {
                match self {
                    $(
                        Self::$variant $( ( $crate::macros::_ignore_payload!($payload_ty) ) )? => $event_val,
                    )*
                }
            }

            /// Decodes an event code and raw payload slice into a typed message.
            pub fn decode(event: $event_ty, data: &[u8]) -> Result<Self, &'static str> {
                match event {
                    $(
                        $event_val => {
                            $crate::macros::_decode_variant!(data, Self::$variant $(, $payload_ty)? )
                        }
                    )*
                    #[allow(unreachable_patterns)]
                    _ => Err("unhandled event"),
                }
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _ignore_payload {
    ($ty:ty) => { _ };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _decode_variant {
    ($data:expr, $variant:path, $ty:ty) => {
        <$ty>::from_bytes($data).map($variant)
    };
    ($data:expr, $variant:path) => {
        Ok($variant)
    };
}

pub use _decode_variant;
pub use _ignore_payload;
