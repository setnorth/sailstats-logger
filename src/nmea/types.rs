//! Types definitions

/// Time in the format `(HH,mm,ss.SSS)`
/// 
/// Fields are as follows:
/// 
/// `u8: HH` - Hours in 24h format
/// 
/// `u9: mm` - minutes
/// 
/// `f32: ss` - seconds
/// 
/// `f32': SSS` - milliseconds
pub type Timestamp = (u8,u8,f32);

/// Parameter group number
pub type TPgn = u32;

/// Priority
pub type TPrio = u8;

/// Source adress
pub type TSrc = u8;

/// Destination Adress
pub type TDest = u8;

/// Data bytes
pub type TData = Vec<u8>;