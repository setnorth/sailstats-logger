use crate::nmea::types::{TData, TDest, TPgn, TPrio, TSrc, Timestamp};
use crate::nmea::nmea2000::messages::*;
use crate::nmea::MessageValue;

use std::collections::HashMap;
use std::marker;

use thiserror::Error;

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
    fn write(&self, message: &mut Box<dyn Message>) -> Result<(),NMEA2000Error>;
}

/// Read a `Raw` packet from some type `T`
pub trait From<T>{
    /// Reads a `Raw` packet from some type `T`.
    fn from(s: &T) -> Result<Self,NMEA2000Error> where 
        Self: Raw + Sized;
}

/// Return [`MessageValue`]s. Must implement [`MessageData`].
pub trait Message: MessageData{
    /// Returns the message values
    fn values(&self) -> Vec<MessageValue>;
}

/// Functions to get access to [`Message`] fields
pub trait MessageData{
    fn timestamp(&self) -> Timestamp;
    fn timestamp_mut(&mut self) -> &mut Timestamp;
    fn src(&self) -> TSrc;
    fn src_mut(&mut self) -> &mut TSrc;
    fn dest(&self) -> TDest;
    fn dest_mut(&mut self) -> &mut TDest;
    fn prio(&self) -> TPrio;
    fn prio_mut(&mut self) -> &mut TPrio;
    fn data(&self) -> &TData;
    fn data_mut(&mut self) -> &mut TData;

    fn pgn(&self) -> TPgn;
    fn bytes(&self) -> usize;
    fn is_fast(&self) -> bool;
    fn is_complete(&self) -> bool;

    fn counter_mask(&self) -> u8;
    fn counter_mask_mut(&mut self) -> &mut u8;
    fn next_packet(&self) -> u8;
    fn next_packet_mut(&mut self) -> &mut u8;
    fn remaining_bytes(&self) -> usize;
    fn remaining_bytes_mut(&mut self) -> &mut usize;
}

/// Parser for NMEA2000 messages
/// 
/// Used to initiate a flexible parser object that accepts different kinds of [`Raw`] types with different
/// kind of [`From`] interfaces to buses, files, lines etc. 
/// 
/// Type `T` denotes the [`Raw`]-type, `U` is the [`Raw`]'s source type. `U` is not used in the `struct` 
/// initialization but later in its method implementation.
/// 
/// # Examples
/// 
/// ```
/// use nmea::nmea2000;
/// use nmea::nmea2000::yd;
/// 
/// let mut parser = nmea2000::Parser::<yd::Raw,String>::new();
/// ```
pub struct Parser<T,U>{
    /// Messages are stored here if they are not completely received.
    messages: HashMap<(TSrc, TPgn), Box<dyn Message>>,
    _raw_type: marker::PhantomData<T>,
    _ingest_type: marker::PhantomData<U>
}

impl<T: Raw + From<U>,U> Parser<T,U>{
    /// Returns a new [`Parser`] 
    /// 
    /// # Examples
    /// 
    /// ```
    /// use nmea::nmea2000;
    /// use nmea::nmea2000::yd;
    /// 
    /// let mut parser = nmea2000::Parser::<yd::Raw,String>::new();
    /// ```
    pub fn new() -> Self{ 
        Parser::<T,U>{
                    messages: HashMap::new(), 
                    _raw_type: marker::PhantomData, 
                    _ingest_type: marker::PhantomData
                } 
    }

    /// Parses first the source type `U` into a [`Raw`] and calls then [`Parser::parse_from_raw`] with the newly
    /// created [`Raw`] instance. Returns `Ok(Some(message))` if a complete message was received by this
    /// source.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use nmea::nmea2000;
    /// use nmea::nmea2000::yd;
    /// 
    /// let mut parser = nmea2000::Parser::<yd::Raw,String>::new();
    /// if let Some(message) = 
    ///     parser.parse("17:33:21.141 R 09F80115 A0 7D E6 18 C0 05 FB D5".to_string()).unwrap() {
    ///     //New message received
    /// }
    /// ```
    pub fn parse(&mut self, src: &U) -> Result<Option<Box<dyn Message>>,NMEA2000Error>{
        let raw = T::from(src)?;
        Ok(self.parse_from_raw(&raw))
    }

    pub fn parse_from_raw(&mut self, raw: &T) -> Option<Box<dyn Message>>{
        let mut message : Box<dyn Message>;
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
                TimeDateMessage::PGN                => Box::new(TimeDateMessage::new()),
                _ => return None
            }
        }

        // There is no error treatment here. It would be possible to use it with an Option, but for
        // debug purposes it is easier here to keep it like this and modify it in case new raw-writers
        // are implemented. During normal execution frequent PacketOutOfSequence errors occur, but are
        // ignored at this point. They seem to be normal for NMEA2000 (at least in my system)
        if raw.write(&mut message).is_err(){
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

#[derive(Error,Debug)]
pub enum NMEA2000Error{
    #[error("unknown raw format")]
    RawFormatError,
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error(transparent)]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("packet out of sequence")]
    PacketOutOfSequence,
    #[error("unexpected length of packet")]
    UnexpectedPacketLength,
}