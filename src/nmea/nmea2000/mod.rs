use crate::nmea::types::{TData, TDest, TPgn, TPrio, TSrc, Timestamp};
use crate::nmea::nmea2000::messages::*;
use crate::state::State;
use crate::nmea::nmea2000::yd::FromStr;

use std::collections::HashMap;

pub mod messages;
pub mod yd;

/// Raw format.
/// 
/// Implements the standard a raw packet that is used in a message needs to implement.
pub trait Raw{
    //type ErrType;
    fn timestamp(&self) -> Timestamp;
    fn src(&self) -> TSrc;
    fn dest(&self) -> TDest;
    fn prio(&self) -> TPrio;
    fn pgn(&self) -> TPgn;
    fn data(&self) -> TData;
    //fn ingest_str(&self) -> Result<(),Self::ErrType>;
}

/// Read information from a `Raw`-type.
/// 
/// Allows a type to read from a given `RawType`. Usually applied with `RawType=dyn Raw`.
pub trait FromRaw{
    type RawType: ?Sized;

    /// Returns true if the message is complete.
    fn is_complete(&self) -> bool;
    
    /// Parses message from a Raw type.
    /// 
    /// Returns `()` if all data was copied an `MessageErr` in case of a parsing error.
    fn from_raw(&mut self, raw: &Self::RawType) -> Result<(),MessageErr>;
}

/// NMEA2000 message.
/// 
/// Message type that can update a State and must be able to be read from a raw type.
pub trait Message : FromRaw{
    fn update(&self, s: &mut State);
}

/// Errors relating to parsing NMEA2000 messages
pub enum MessageErr{
    /// The supplied raw packet is out of sequence
    OutOfSequence,
    /// The supplied raw packet is of unexpected length
    UnexpectedLength
}


/// Holds the state of the parser
#[derive(Default)]
pub struct Parser{
    /// Container for messages
    /// 
    /// Each Key (TSrc,TPgn) contains a pointer to a Message that might not be fully received
    messages: HashMap<(TSrc,TPgn),Box<dyn Message<RawType=dyn Raw>>>
}

impl Parser{
    /// Initializes a new parser object
    pub fn new() -> Self{
        Parser{..Default::default()}
    }
    
    /// Parse from a supplied string
    pub fn parse_string(&mut self, string: &str) -> Result<Option<Box<dyn Message<RawType=dyn Raw>>>, yd::YDRawParseError>{
        let raw = yd::Raw::from_str(string)?;
        Ok(self.parse(&raw))
    }

    /// Parse raw data to a Message type
    /// 
    /// Returns either a `Some(Message)` if the message is complete or `None`
    pub fn parse<T: Raw + 'static>(&mut self, raw: &T) -> Option<Box<dyn Message<RawType=dyn Raw>>>{
        let mut message : Box<dyn Message<RawType=dyn Raw>>;
        
        // Check if there is an incomplete message in the storage,
        // otherwise create a new one
        if let Some(m) = self.messages.remove(&(raw.src(),raw.pgn())){
            message = m;
        } else {
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
            Some(message)
        }else{
            //Message is not complete, push it into storage
            self.messages.insert((raw.src(),raw.pgn()), message);
            None
        }
    }
}
