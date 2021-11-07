use crate::types::{TData, TDest, TPgn, TPrio, TSrc, Timestamp};
use crate::state::State;

/// NMEA2000 Raw format
pub trait N2kRaw {
    fn timestamp(&self) -> Timestamp;
    fn src(&self) -> TSrc;
    fn dest(&self) -> TDest;
    fn prio(&self) -> TPrio;
    fn pgn(&self) -> TPgn;
    fn data(&self) -> TData;
}

/// N2kMessages that can be read from raw formats
pub trait N2kFromRaw<T: N2kRaw + ?Sized>{
    /// Returns true if the message is complete.
    fn is_complete(&self) -> bool;
    
    /// Parses message from a N2kRaw type.
    /// 
    /// Returns `()` if all data was copied an `N2kMessageErr` in case of a parsing error.
    fn from_raw(&mut self, raw: &T) -> Result<(),N2kMessageErr>;
}

/// NMEA2000 Message type that can update a State
pub trait N2kMessage : N2kFromRaw<dyn N2kRaw>{
    fn update(&self, s: &mut State);
}

/// Errors relating to parsing NMEA2000 messages
pub enum N2kMessageErr{
    /// The supplied raw packet is out of sequence
    OutOfSequence,
    /// The supplied raw packet is of unexpected length
    UnexpectedLength
}