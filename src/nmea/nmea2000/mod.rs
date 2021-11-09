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

/// Read a `Raw` packet from some type `T`
pub trait From<T>{
    /// Reads a `Raw` packet from some type `T`.
    fn from(s: &T) -> Result<Self,Box<dyn std::error::Error>> where 
        Self: Raw + Sized;
}

/// Message as interface to the frontend for `State` updating
pub trait Message<T: Raw> : FromRaw<T>{
    /// Updates a supplied state `s` with the message's information.
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
    messages: HashMap<(TSrc, TPgn), Box<dyn Message<T>>>,
    /// Unused variable that is only required in order to have a type `U` for the [`Raw`]'s
    /// implementation of the [`From`] trait.
    _phantom: marker::PhantomData<U>
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
    pub fn new() -> Self{ Parser::<T,U>{messages: HashMap::new(), _phantom: marker::PhantomData} }

    /// Parses first the source type `U` into a [`Raw`] and calls then [`Parser::parse_from_raw`] with the newly
    /// created [`Raw`] instance. Returns `Ok(Some(message))` if a complete message was received by this
    /// source packet.
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

