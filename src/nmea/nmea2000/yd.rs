//! Tools to read Yacht Devices Raw format messages from string. Implements the `N2kRaw` trait.
//! 
//! Yacht Devices Raw format:
//! 
//!  `hh:mm:ss.ddd D msgid b0 b1 b2 b3 b4 b5 b6 b7<CR><LF>`
//! 
//!  where:
//! 
//!  • hh:mm:sss.ddd — time of message transmission or reception, ddd are milliseconds
//! 
//!  • D — direction of the message (‘R’ — from NMEA 2000 to application, ‘T’ — from application to NMEA 2000)
//! 
//!  • msgid — 29-bit message identifier in hexadecimal format (contains NMEA 2000 PGN and other fields)
//! 
//!  • b0..b7 — message data bytes (from 1 to 8) in hexadecimal format
//! 
//!  • `<CR><LF>`
use std::fmt;

use crate::nmea::types::{TData, TDest, TPgn, TPrio, TSrc, Timestamp};
use crate::nmea::nmea2000;

use std::cmp;
use std::str::FromStr;

/// Holds a YDRaw message.
/// 
/// The values for priority, pgn, src and dest are derived.
pub struct Raw{
    //Parsed values
    pub timestamp : Timestamp,
    pub direction : YDRawDirection,
    pub msgid : u32,
    pub data : [u8;8],

    //Derived values (ISO11783 Bits)
    pub prio : u8,
    pub pgn : u32,
    pub src : u8,
    pub dest : u8
}

impl nmea2000::Raw for Raw{
    #[inline(always)]
    fn timestamp(&self) -> Timestamp { self.timestamp }
    #[inline(always)]
    fn src(&self) -> TSrc { self.src }
    #[inline(always)]
    fn dest(&self) -> TDest { self.dest }
    #[inline(always)]
    fn prio(&self) -> TPrio { self.prio }
    #[inline(always)]
    fn pgn(&self) -> TPgn { self.pgn }
    #[inline(always)]
    fn data(&self) -> TData { self.data.to_vec() }

    fn write(&self, m: &mut Box<dyn nmea2000::Message>) -> Result<(),nmea2000::MessageErr>{
        //Is this a fast message?
        //(This part is optimized in the compiler and only present
        // in messages which are consisting of several raw-packets)
        if m.is_fast(){
            //If we are just starting this new fast package
            if (m.next_packet() == 0) && (self.data[0] & 0x1F == 0){
                //Check if this packet has the same length as we expect to see
                if m.bytes() != self.data[1] as usize {
                    return Err(nmea2000::MessageErr::UnexpectedLength);
                }
                //Set values and the first 6 bytes for this package
                *m.timestamp_mut() = self.timestamp;
                *m.src_mut() = self.src;
                *m.dest_mut() = self.dest;
                *m.prio_mut() = self.prio;
                *m.counter_mask_mut() = self.data[0];
                *m.next_packet_mut() += 1;
                *m.remaining_bytes_mut() = m.bytes() - 6;
                m.data_mut().append(&mut self.data[2..8_usize].to_vec());
            } else {
                //This packet is already begun...
                //If the packet is the next in series
                if m.next_packet() == (m.counter_mask() ^ self.data[0]){
                    let l = cmp::min(m.remaining_bytes()+1,8);
                    m.data_mut().append(&mut self.data[1..l as usize].to_vec());
                    *m.remaining_bytes_mut() -= cmp::min(m.remaining_bytes(),7);
                    *m.next_packet_mut() += 1;
                } else {
                    //It seems that the previous sequence was not finished. Try to start a new sequence.
                    //Check that only bits in sequence identifier (raw.data[0] & 0b00011111) and sequence
                    //size with what we expect.
                    if (self.data[0] & 0x1F == 0) && ((self.data[1] as usize ) == m.bytes() as usize){
                        *m.timestamp_mut() = self.timestamp;
                        *m.src_mut() = self.src;
                        *m.dest_mut() = self.dest;
                        *m.prio_mut() = self.prio;
                        *m.counter_mask_mut() = self.data[0];
                        *m.next_packet_mut() += 1;
                        *m.remaining_bytes_mut() = m.bytes() - cmp::min(m.bytes(),6);
                        m.data_mut().clear();
                        m.data_mut().append(&mut self.data[2..8_usize].to_vec());
                    } else {
                        return Err(nmea2000::MessageErr::OutOfSequence);
                    }
                }
            }
        } else {
            //Just a normal packet
            *m.timestamp_mut() = self.timestamp;
            *m.src_mut() = self.src;
            *m.dest_mut() = self.dest;
            *m.prio_mut() = self.prio;
            m.data_mut().append(&mut self.data.to_vec());
        }
        Ok(())
    }    
}

impl nmea2000::From<String> for Raw{
    fn from(s: &String) -> Result<Self, Box<dyn std::error::Error>>{
        // Split data fields
        let t = s.to_string();
        let mut fields = t.split_whitespace();
        
        //Parse time
        let t = fields.next().ok_or(YDRawParseError::IteratorError)?;
        let timestamp = (
            u8::from_str(&t[0..2])?,
            u8::from_str(&t[3..5])?,
            f32::from_str(&t[6..12])?
        );

        //Get direction
        let d = fields.next().ok_or(YDRawParseError::IteratorError)?;
        let direction = match d{
            "R" => YDRawDirection::Received,
            "T" => YDRawDirection::Transmitted,
            _ => return Err(Box::new(YDRawParseError::InvalidField))
        };

        //Parse Message Id
        let m = fields.next().ok_or(YDRawParseError::IteratorError)?;
        let msgid = u32::from_str_radix(m,16)?;

        //Derive values from msgid (ISO11783 Bits)
        //Without the help of the canboat project 
        //(https://github.com/canboat/canboat/) it would
        //have been a lot harder to find out how this works.
        let pf : u8 = (msgid >> 16) as u8;
        let ps : u8 = (msgid >> 8) as u8;
        let rdp : u8 = ((msgid >> 24) & 3) as u8;

        let src = msgid as u8;
        let prio = ((msgid >> 26) & 0x7) as u8;
        
        let (dest,pgn) : (u8,u32);
        if pf < 240{
            dest = ps;
            pgn = ((rdp as u32) << 16) + ((pf as u32) << 8);
        }else{
            dest = 0xff;
            pgn = ((rdp as u32) << 16) + ((pf as u32) << 8) + (ps as u32);
        }

        //Get 8 message bytes, no more, no less
        //At this stage the method is not checking if there are enough or too few
        //bytes in the message string. Essentially it trusts that the string received from
        //from Yacht Devices GW is always well formed with exactly 8 bytes in the message.
        let mut data = [0,0,0,0,0,0,0,0];
        for (f,i) in fields.zip(0..8){
            data[i] = u8::from_str_radix(f,16)?;
        }
        
        Ok(Raw{
            timestamp, 
            direction,
            msgid, 
            data,
            prio,
            pgn,
            src,
            dest
        })
    }
}

/// Denotes the direction, i.e., if a package was received or transmitted.
#[derive(Debug)]
pub enum YDRawDirection {Received,Transmitted}

/// Display trait implementation
impl fmt::Display for Raw{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        write!(f,"{:02}:{:02}:{:0>6.3} ",self.timestamp.0, self.timestamp.1, self.timestamp.2)?;
        match self.direction {
            YDRawDirection::Received => { write!(f,"R ")?; }
            YDRawDirection::Transmitted => { write!(f,"T ")?; }
        }
        write!(f,"{:08X} ",self.msgid)?;
        for b in &self.data[0..7]{
            write!(f,"{:02X} ", b)?;
        }
        write!(f,"{:02X}",self.data[7])
    }
}

/* 
 * Error Handling
 */
/// Error type for the YDRawParser
#[derive(Debug)]
pub enum YDRawParseError {
    IteratorError,
    InvalidField
}
impl std::error::Error for YDRawParseError {}

/// Display trait implementation of YDRawParseError
impl fmt::Display for YDRawParseError{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        match &*self {
            YDRawParseError::IteratorError => write!(f, "Empty Iterator."),
            YDRawParseError::InvalidField => write!(f, "Invalid input.")
        }
    }
}

