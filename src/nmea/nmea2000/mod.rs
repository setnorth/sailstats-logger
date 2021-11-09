use crate::nmea::types::{TData, TDest, TPgn, TPrio, TSrc, Timestamp};
use crate::nmea::nmea2000::messages::*;
use crate::state::State;

use std::collections::HashMap;
use std::marker;

pub mod messages;
pub mod yd;

/// NMEA2000 Raw format
/// 
/// Defines standard methods for a NMEA2000 raw packet.
pub trait Raw{
    fn timestamp(&self) -> Timestamp;
    fn src(&self) -> TSrc;
    fn dest(&self) -> TDest;
    fn prio(&self) -> TPrio;
    fn pgn(&self) -> TPgn;
    fn data(&self) -> TData;
}

/// Read a `Raw` packet from some type `T`.
pub trait From<T>{
    /// Read a `Raw` packet from some type `T`.
    fn from(s: &T) -> Result<Self,Box<dyn std::error::Error>> where 
        Self: Raw + Sized;
}

/// `Message` of some `Raw`-type `T` that can update the State
pub trait Message<T: Raw> : FromRaw<T>{
    fn update(&self, s: &mut State);
}

/// Read information into a `Message` from some `Raw`-type `T`.
pub trait FromRaw<T: Raw>{
    /// Returns true if the message is complete.
    fn is_complete(&self) -> bool;
    
    /// Parses message from a Raw type.
    fn from_raw(&mut self, raw: &T) -> Result<(),MessageErr>;
}

/// Errors relating to parsing NMEA2000 messages
pub enum MessageErr{
    /// The supplied raw packet is out of sequence
    OutOfSequence,
    /// The supplied raw packet is of unexpected length
    UnexpectedLength
}

/// Parser for that parses some `Raw` of type `T` from values of type `U`.
pub struct Parser<T,U>{
    messages: HashMap<(TSrc, TPgn), Box<dyn Message<T>>>,
    phantom: marker::PhantomData<U>
}

impl<T: Raw + From<U>,U> Parser<T,U>{
    pub fn new() -> Self{ Parser::<T,U>{messages: HashMap::new(), phantom: marker::PhantomData} }

    pub fn parse(&mut self, src: &U) -> Result<Option<Box<dyn Message<T>>>,Box<dyn std::error::Error>>{
        let raw = T::from(src)?;
        Ok(self.parse_from_raw(&raw))
    }

    pub fn parse_from_raw(&mut self, raw: &T) -> Option<Box<dyn Message<T>>>{
        let mut message : Box<dyn Message<T>>;
        if let Some(m) = self.messages.remove(&(raw.src(),raw.pgn())){
            message = m;
        }else{
            message = match raw.pgn(){
                WindMessage::PGN                    => Box::new(WindMessage::new()),
                PositionRapidUpdateMessage::PGN     => Box::new(PositionRapidUpdateMessage::new()),
                GNSSPositionData::PGN               => Box::new(GNSSPositionData::new()),
                VesselHeadingMessage::PGN           => Box::new(VesselHeadingMessage::new()),
                CogSogRapidUpdateMessage::PGN       => Box::new(CogSogRapidUpdateMessage::new()),
                SpeedMessage::PGN                   => Box::new(SpeedMessage::new()),
                RateOfTurnMessage::PGN              => Box::new(RateOfTurnMessage::new()),
                AttitudeMessage::PGN                => Box::new(AttitudeMessage::new()),
                RudderMessage::PGN                  => Box::new(RudderMessage::new()),
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

