use super::ydwg::{YDWGRaw, YDWGParseError, FromStr};
use super::packet::{Packet};
use super::message::*;
use super::types::*;

use std::collections::HashMap;
use std::cmp::min;

pub struct Parser{
    messages: HashMap<(TSrc,TPgn),Box<dyn Message>>,    // Container (Source,Pgn) -> Message
}
impl Parser{
    /// Initializes a new parser object with a set of predefined parameter groups of interest that are filtered
    pub fn new() -> Self{
        Parser{
            messages: HashMap::new(),
        }
    }
    
    pub fn parse_string(&mut self, string: &str) -> Result<Option<Box<dyn Message>>, YDWGParseError>{
        let raw = YDWGRaw::from_str(string)?;
        Ok(self.parse(&raw))
    }

    pub fn parse(&mut self, raw: &YDWGRaw) -> Option<Box<dyn Message>>{
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

    fn from_raw(message: &mut Box<dyn Message>, raw: &YDWGRaw) -> Result<(),&'static str>{
        //Get the message internals
        let message_bytes = message.bytes();
        let message_fast = message.fast();
        let p : &mut Packet = message.packet_mut();

        //Is this a fast message?
        if message_fast{
            //Is this the first package? (If we haven't read anything yet)
            //And is everything except the sequence identifier zero in the first bit?
            if (p.next_packet == 0) && ((raw.data[0] & 0x1F) == 0){
                //Check if this packet has the same length as we expect to see
                if message_bytes != raw.data[1] as usize{
                    //The following error could also occur when we start the program and the
                    //first packet we read is in the middle of a sequence.
                    return Err("Unexpected length for fast packet.");
                }
                //Set values and the first 6 bytes for this package
                p.timestamp = raw.timestamp;
                p.src = raw.src;
                p.dest = raw.dest;
                p.prio = raw.prio;
                p.counter_mask = raw.data[0];
                p.next_packet += 1;
                p.remaining_bytes = message_bytes - 6;
                p.data.append(&mut raw.data[2..8_usize].to_vec());
            } else{ //This packet is already begun
                //If the packet is the next in series
                if p.next_packet == (p.counter_mask ^ raw.data[0]){
                    p.data.append(&mut raw.data[1..(min(p.remaining_bytes+1,8)) as usize].to_vec());
                    p.remaining_bytes -= min(p.remaining_bytes,7);
                    p.next_packet += 1;
                } else {
                    //It seems that the previous sequence was not finished. Try to start a new sequence.
                    //Check that only bits in sequence identifier (raw.data[0] & 0b00011111) and sequence
                    //size with what we expect.
                    if ((raw.data[0] & 0x1F) == 0) && ((raw.data[1] as usize) == message_bytes){
                        p.timestamp = raw.timestamp;
                        p.src = raw.src;
                        p.dest = raw.dest;
                        p.prio = raw.prio;
                        p.counter_mask = raw.data[0];
                        p.next_packet = 0x01;
                        p.remaining_bytes = message_bytes - 6;
                        p.data.clear();
                        p.data.append(&mut raw.data[2..8_usize].to_vec());
                    }else{
                        return Err("Fast packet not in sequence.");
                    }
                }
            }
        }else{
            //Not a fast packet, i.e., nothing in sequence, read required bytes.
            p.timestamp = raw.timestamp;
            p.src = raw.src;
            p.dest = raw.dest;
            p.prio = raw.prio;
            p.data.append(&mut raw.data[0..message_bytes].to_vec());
            p.remaining_bytes = message_bytes - p.data.len();
        }
        Ok(())
    }
}
