//! Parse Raw data to messages
use crate::yd_raw::{YDRaw, YDRawParseError, FromStr};
use crate::message::*;
use crate::types::*;

use std::collections::HashMap;
use std::cmp::min;

/// Holds the state of the parser
#[derive(Default)]
pub struct Parser{
    /// Container for messages
    /// 
    /// Each Key (TSrc,TPgn) contains a pointer to a Message that might not be fully received
    messages: HashMap<(TSrc,TPgn),Box<dyn Message>>
}

impl Parser{
    /// Initializes a new parser object
    pub fn new() -> Self{
        Parser{..Default::default()}
    }
    
    /// Parse from a supplied string
    pub fn parse_string(&mut self, string: &str) -> Result<Option<Box<dyn Message>>, YDRawParseError>{
        let raw = YDRaw::from_str(string)?;
        Ok(self.parse(&raw))
    }

    /// Parse raw data to a Message type
    pub fn parse(&mut self, raw: &YDRaw) -> Option<Box<dyn Message>>{
        let mut message : Box<dyn Message>;
        
        // Check if there is an incomplete message in the storage,
        // otherwise create a new one
        if let Some(m) = self.messages.remove(&(raw.src,raw.pgn)){
            message = m;
        } else {
            message = match raw.pgn{
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
        
        //Need a bit better error treatment here.
        if Parser::from_raw(&mut message, raw).is_err() {
            return None
        }
        
        if message.complete(){
            Some(message)
        }else{
            //Message is not complete, push it into storage
            self.messages.insert((raw.src,raw.pgn), message);
            None
        }
    }

    /// Fill the supplied Message with information from the raw data
    fn from_raw(m: &mut Box<dyn Message>, raw: &YDRaw) -> Result<(),&'static str>{
        //Is this a fast message?
        if m.fast(){
            //Is this the first package? (If we haven't read anything yet)
            //And is everything except the sequence identifier zero in the first bit?
            if (m.next_packet() == 0) && ((raw.data[0] & 0x1F) == 0){
                //Check if this packet has the same length as we expect to see
                if m.bytes() != raw.data[1] as usize{
                    //The following error could also occur when we start the program and the
                    //first packet we read is in the middle of a sequence.
                    return Err("Unexpected length for fast packet.");
                }
                //Set values and the first 6 bytes for this package
                *m.mut_timestamp() = raw.timestamp;
                *m.mut_src() = raw.src;
                *m.mut_dest() = raw.dest;
                *m.mut_prio() = raw.prio;
                *m.mut_counter_mask() = raw.data[0];
                *m.mut_next_packet() += 1;
                *m.mut_remaining_bytes() = m.bytes() - 6;
                (*m.mut_data()).append(&mut raw.data[2..8_usize].to_vec());
            } else{ //This packet is already begun
                //If the packet is the next in series
                if m.next_packet() == (m.counter_mask() ^ raw.data[0]){
                    let bytes = min(m.remaining_bytes()+1,8) as usize;
                    (*m.mut_data()).append(&mut raw.data[1..bytes].to_vec());
                    *m.mut_remaining_bytes() -= min(m.remaining_bytes(),7);
                    *m.mut_next_packet() += 1;
                } else {
                    //It seems that the previous sequence was not finished. Try to start a new sequence.
                    //Check that only bits in sequence identifier (raw.data[0] & 0b00011111) and sequence
                    //size with what we expect.
                    if ((raw.data[0] & 0x1F) == 0) && ((raw.data[1] as usize) == m.bytes()){
                        *m.mut_timestamp() = raw.timestamp;
                        *m.mut_src() = raw.src;
                        *m.mut_dest() = raw.dest;
                        *m.mut_prio() = raw.prio;
                        *m.mut_counter_mask() = raw.data[0];
                        *m.mut_next_packet() = 0x01;
                        *m.mut_remaining_bytes() = m.bytes() - 6;
                        (*m.mut_data()).clear();
                        (*m.mut_data()).append(&mut raw.data[2..8_usize].to_vec());
                    }else{
                        return Err("Fast packet not in sequence.");
                    }
                }
            }
        }else{
            //Not a fast packet, i.e., nothing in sequence, read required bytes.
            *m.mut_timestamp() = raw.timestamp;
            *m.mut_src() = raw.src;
            *m.mut_dest() = raw.dest;
            *m.mut_prio() = raw.prio;
            let bytes = m.bytes();
            (*m.mut_data()).append(&mut raw.data[0..bytes].to_vec());
            *m.mut_remaining_bytes() = m.bytes() - m.data().len();
        }
        Ok(())
    }
}
