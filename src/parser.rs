//! Parse Raw data to messages
use crate::yd_raw::{YDRaw, YDRawParseError, FromStr};
use crate::message::*;
use crate::types::*;

use std::collections::HashMap;

/// Holds the state of the parser
#[derive(Default)]
pub struct Parser{
    /// Container for messages
    /// 
    /// Each Key (TSrc,TPgn) contains a pointer to a Message that might not be fully received
    messages: HashMap<(TSrc,TPgn),Box<dyn N2kMessage>>
}

impl Parser{
    /// Initializes a new parser object
    pub fn new() -> Self{
        Parser{..Default::default()}
    }
    
    /// Parse from a supplied string
    pub fn parse_string(&mut self, string: &str) -> Result<Option<Box<dyn N2kMessage>>, YDRawParseError>{
        let raw = YDRaw::from_str(string)?;
        Ok(self.parse(&raw))
    }

    /// Parse raw data to a Message type
    /// 
    /// Returns either a `Some(Message)` if the message is complete or `None`
    pub fn parse(&mut self, raw: &(dyn N2kRaw + 'static)) -> Option<Box<dyn N2kMessage>>{
        let mut message : Box<dyn N2kMessage>;
        
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
