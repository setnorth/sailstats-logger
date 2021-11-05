//! Tools to read Yacht Devices Raw format messages from string.
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
use std::num::{ParseIntError, ParseFloatError};
use std::io::{Error, ErrorKind};
use std::fmt;
use crate::types::*;

pub use std::str::FromStr;

/// Holds a YDRaw message.
/// 
/// The values for priority, pgn, src and dest are derived.
pub struct YDRaw{
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

/// Denotes the direction, i.e., if a package was received or transmitted.
pub enum YDRawDirection {Received,Transmitted}

/// Createes a YDRawRaw package from a string
impl FromStr for YDRaw{
    type Err = YDRawParseError;
    fn from_str(s : &str) -> Result<Self,Self::Err> {
        // Split data fields
        let mut fields = s.split_whitespace();
        
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
            _ => return Err(YDRawParseError::InvalidField)
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
        
        Ok(YDRaw{
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

/// Display trait implementation
impl fmt::Display for YDRaw{
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
pub enum YDRawParseError {
    IntegerError(ParseIntError),
    FloatError(ParseFloatError),
    IteratorError,
    InvalidField
}

/// Converts ParseIntError to YDRawParseError
impl From<ParseIntError> for YDRawParseError{
    fn from(err : ParseIntError) -> YDRawParseError{
        YDRawParseError::IntegerError(err)
    }
}

/// Converts ParseFloatError to YDRawParseError
impl From<ParseFloatError> for YDRawParseError{
    fn from(err : ParseFloatError) -> YDRawParseError{
        YDRawParseError::FloatError(err)
    }
}

/// Display trait implementation of YDRawParseError
impl fmt::Display for YDRawParseError{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        match &*self {
            YDRawParseError::IntegerError(err) => err.fmt(f),
            YDRawParseError::FloatError(err) => err.fmt(f),
            YDRawParseError::IteratorError => write!(f, "Empty Iterator."),
            YDRawParseError::InvalidField => write!(f, "Invalid input.")
        }
    }
}

/// Converts YDRawParseError to Error
impl From<YDRawParseError> for Error{
    fn from(err : YDRawParseError) -> Error{
        Error::new(ErrorKind::InvalidInput,err.to_string())
    }
}
