use crate::nmea::types::{TData, TDest, TPgn, TPrio, TSrc, Timestamp};
use crate::nmea::nmea2000::messages::*;
use crate::state::State;

use std::collections::HashMap;

pub mod messages;
pub mod yd;

/// Raw format.
/// 
/// Implements the standard a raw packet that is used in a message needs to implement.
pub trait Raw{
    fn timestamp(&self) -> Timestamp;
    fn src(&self) -> TSrc;
    fn dest(&self) -> TDest;
    fn prio(&self) -> TPrio;
    fn pgn(&self) -> TPgn;
    fn data(&self) -> TData;
}

/// Read message information from a `Raw`-type.
pub trait FromRaw<T>{
    /// Returns true if the message is complete.
    fn is_complete(&self) -> bool;
    
    /// Parses message from a Raw type.
    fn from_raw(&mut self, raw: &T) -> Result<(),MessageErr>;
}

pub trait From<U>{
    fn from(s: &U) -> Result<Self,Box<dyn std::error::Error>> where Self: Sized;
}

/// NMEA2000 message.
pub trait Message<T: Raw> : FromRaw<T>{
    fn update(&self, s: &mut State);
}

/// Errors relating to parsing NMEA2000 messages
pub enum MessageErr{
    /// The supplied raw packet is out of sequence
    OutOfSequence,
    /// The supplied raw packet is of unexpected length
    UnexpectedLength
}

pub struct Parser2<T>{
    messages: HashMap<(TSrc, TPgn), Box<dyn Message<T>>>
}

impl<T: Raw+From<String>> Parser2<T>{
    pub fn new() -> Self{ Parser2::<T>{messages: HashMap::new()} }

    pub fn parse_string(&mut self, src: &String) -> Result<Option<Box<dyn Message<T>>>,Box<dyn std::error::Error>>{
        let raw = T::from(src)?;
        Ok(self.parse(&raw))
    }

    pub fn parse(&mut self, raw: &T) -> Option<Box<dyn Message<T>>>{
        let mut message : Box<dyn Message<T>>;
        if let Some(m) = self.messages.remove(&(raw.src(),raw.pgn())){
            message = m;
        }else{
            message = match raw.pgn(){
                WindMessage::PGN                    => Box::new(WindMessage::new()),
                /*PositionRapidUpdateMessage::PGN     => Box::new(PositionRapidUpdateMessage::new()),
                GNSSPositionData::PGN               => Box::new(GNSSPositionData::new()),
                VesselHeadingMessage::PGN           => Box::new(VesselHeadingMessage::new()),
                CogSogRapidUpdateMessage::PGN       => Box::new(CogSogRapidUpdateMessage::new()),
                SpeedMessage::PGN                   => Box::new(SpeedMessage::new()),
                RateOfTurnMessage::PGN              => Box::new(RateOfTurnMessage::new()),
                AttitudeMessage::PGN                => Box::new(AttitudeMessage::new()),
                RudderMessage::PGN                  => Box::new(RudderMessage::new()),*/
                _ => return None
            }
        }

        if message.from_raw(raw).is_err(){
            return None
        }

        if message.is_complete(){
            return Some(message)
        }else{
            self.messages.insert((raw.src(),raw.pgn()), message);
        }

        None
    }
}

