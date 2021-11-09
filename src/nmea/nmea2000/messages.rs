//! nmea2000::Message types 
use crate::state::State;
use crate::nmea::types::{TData, TDest, TPgn, TPrio, TSrc, Timestamp};
use crate::nmea::nmea2000;

use std::f64::consts::PI;
use std::cmp;

/// Creates a message type that implements the trait nmea2000::MessageData
macro_rules! message_type {
    ($type_name: ident, $pgn: expr, $bytes: expr, $fast: expr) => {
        #[derive(Default)]
        pub struct $type_name {
            /// Time of the nmea2000::Message
            pub timestamp: Timestamp,
            /// Priority
            pub prio: TPrio,
            /// Source
            pub src: TSrc,
            /// Destination
            pub dest: TDest,
            /// Databytes
            pub data: TData,
            
            /// Masks the counter value for subsequent packets
            pub counter_mask : u8,         
            /// Next message number we expect
            pub next_packet : u8,          
            /// Remaining bytes until the nmea2000::Message is complete
            pub remaining_bytes : usize,               
        }
        
        impl $type_name{
            pub const PGN: TPgn = $pgn;
            pub const BYTES: usize = $bytes;
            pub const FAST: bool = $fast;

            pub fn new() -> Self{ $type_name{..Default::default()} }
        }

        impl<T: nmea2000::Raw> nmea2000::FromRaw<T> for $type_name{
            fn is_complete(&self) -> bool {
                match self.remaining_bytes{
                    0 => true,
                    _ => false
                }
            }

            fn from_raw(&mut self, raw: &T) -> Result<(),nmea2000::MessageErr>{
                //Is this a fast message?
                //(This part is most likely optimized in the compiler and only present
                // in messages which are consisting of several raw-packets)
                if $type_name::FAST {
                    //If we are just starting this new fast package
                    if (self.next_packet == 0) && (raw.data()[0] & 0x1F == 0){
                        //Check if this packet has the same length as we expect to see
                        if $type_name::BYTES != raw.data()[1] as usize {
                            return Err(nmea2000::MessageErr::UnexpectedLength);
                        }
                        //Set values and the first 6 bytes for this package
                        self.timestamp = raw.timestamp();
                        self.src = raw.src();
                        self.dest = raw.dest();
                        self.prio = raw.prio();
                        self.counter_mask = raw.data()[0];
                        self.next_packet += 1;
                        self.remaining_bytes = $type_name::BYTES - cmp::min($type_name::BYTES,6);
                        self.data.append(&mut raw.data()[2..8_usize].to_vec());
                    } else {
                        //This packet is already begun...
                        //If the packet is the next in series
                        if self.next_packet == (self.counter_mask ^ raw.data()[0]){
                            self.data.append(&mut raw.data()[1..cmp::min(self.remaining_bytes+1,8) as usize].to_vec());
                            self.remaining_bytes -= cmp::min(self.remaining_bytes,7);
                            self.next_packet += 1;
                        } else {
                            //It seems that the previous sequence was not finished. Try to start a new sequence.
                            //Check that only bits in sequence identifier (raw.data[0] & 0b00011111) and sequence
                            //size with what we expect.
                            if ((raw.data()[0] & 0x1F == 0) && ((raw.data()[1] as usize ) == $type_name::BYTES as usize)){
                                self.timestamp = raw.timestamp();
                                self.src = raw.src();
                                self.dest = raw.dest();
                                self.prio = raw.prio();
                                self.counter_mask = raw.data()[0];
                                self.next_packet += 1;
                                self.remaining_bytes = $type_name::BYTES - cmp::min($type_name::BYTES,6);
                                self.data.clear();
                                self.data.append(&mut raw.data()[2..8_usize].to_vec());
                            } else {
                                return Err(nmea2000::MessageErr::OutOfSequence);
                            }
                        }
                    }
                } else {
                    //Just a normal packet
                    self.timestamp = raw.timestamp();
                    self.src = raw.src();
                    self.dest = raw.dest();
                    self.prio = raw.prio();
                    self.data.append(&mut raw.data().to_vec());
                }
                Ok(())
            }
        }
    }
}

message_type!(WindMessage, 130306, 8, false);
impl<T: nmea2000::Raw> nmea2000::Message<T> for WindMessage{
    fn update(&self, s: &mut State){
        s.timestamp = self.timestamp;
        s.aws = u16::from_le_bytes([self.data[1],self.data[2]]) as f32 * 0.01 * 1.943_844_6; //in knots;
        s.awa = u16::from_le_bytes([self.data[3],self.data[4]]) as f32 * 0.0001 * 360.0 / 2.0 / PI as f32;
    }
}

/*message_type!(PositionRapidUpdateMessage, 129025, 8, false);
impl<T: nmea2000::Raw> nmea2000::Message<T> for PositionRapidUpdateMessage{
    fn update(&self, s: &mut State){
        s.timestamp = self.timestamp;
        let mut lat = i32::from_le_bytes([  
            self.data[0],
            self.data[1],
            self.data[2],
            self.data[3]]) as f32;
        lat *= 0.0000001;

        let mut long = i32::from_le_bytes([
            self.data[4],
            self.data[5],
            self.data[6],
            self.data[7]]) as f32;
        long *= 0.0000001;
        s.latitude= lat;
        s.longitude = long;
    }
}

message_type!(GNSSPositionData, 129029, 43, true);
impl<T: nmea2000::Raw> nmea2000::Message<T> for GNSSPositionData{
    fn update(&self, s : &mut State){
        s.timestamp = self.timestamp;
        //Latitude
        let mut lat = i64::from_le_bytes([ 
            self.data[7],
            self.data[8],
            self.data[9],
            self.data[10],
            self.data[11],
            self.data[12],
            self.data[13],
            self.data[14]]) as f64;
        lat *= 0.0000000000000001;
        //Longitude
        let mut long = i64::from_le_bytes([ 
            self.data[15],
            self.data[16],
            self.data[17],
            self.data[18],
            self.data[19],
            self.data[20],
            self.data[21],
            self.data[22]]) as f64;
        long *= 0.0000000000000001;
        s.latitude = lat as f32;
        s.longitude = long as f32;
    }
}

message_type!(VesselHeadingMessage, 127250, 8, false);
impl<T: nmea2000::Raw> nmea2000::Message<T> for VesselHeadingMessage{
    fn update(&self, s: &mut State){
        s.timestamp = self.timestamp;
        s.hdg = u16::from_le_bytes([self.data[1],self.data[2]]) as f32 * 0.0001 * 360.0 / 2.0 / PI as f32;
    }
}

message_type!(CogSogRapidUpdateMessage, 129026, 8, false);
impl<T: nmea2000::Raw> nmea2000::Message<T> for CogSogRapidUpdateMessage{
    fn update(&self, s: &mut State){
        s.timestamp = self.timestamp;
        s.cog = u16::from_le_bytes([self.data[2],self.data[3]]) as f32 * 0.0001 * 360.0 / 2.0 / PI as f32;
        s.sog = u16::from_le_bytes([self.data[4],self.data[5]]) as f32 * 0.01 * 1.943_844_6; //in knots
    }
}

message_type!(SpeedMessage, 128259, 8, false);
impl<T: nmea2000::Raw> nmea2000::Message<T> for SpeedMessage{
    fn update(&self, s: &mut State){
        s.timestamp = self.timestamp;
        s.stw = u16::from_le_bytes([self.data[1],self.data[2]]) as f32 * 0.01 * 1.943_844_6; //in knots
    }
}

message_type!(RateOfTurnMessage, 127251, 5, false);
impl<T: nmea2000::Raw> nmea2000::Message<T> for RateOfTurnMessage{
    fn update(&self, s: &mut State){
        s.timestamp = self.timestamp;
        s.rot = i32::from_le_bytes([self.data[1],
                                    self.data[2],
                                    self.data[3],
                                    self.data[4]]) as f32 * 3.125e-08 * 360.0 / 2.0 / PI as f32;
    }
}

message_type!(AttitudeMessage, 127257, 7, false);
impl<T: nmea2000::Raw> nmea2000::Message<T> for AttitudeMessage{
    fn update(&self, s: &mut State){
        s.timestamp = self.timestamp;
        s.yaw = i16::from_le_bytes([self.data[1],self.data[2]]) as f32 * 0.0001 * 360.0 / 2.0 / PI as f32;
        s.pitch = i16::from_le_bytes([self.data[3],self.data[4]]) as f32 * 0.0001 * 360.0 / 2.0 / PI as f32;
        s.roll = i16::from_le_bytes([self.data[5],self.data[6]]) as f32 * 0.0001 * 360.0 / 2.0 / PI as f32;
    }
}

message_type!(RudderMessage, 127245, 8, false);
impl<T: nmea2000::Raw> nmea2000::Message<T> for RudderMessage{
    fn update(&self, s: &mut State){
        s.timestamp = self.timestamp;
        let value = i16::from_le_bytes([self.data[4],self.data[5]]) as f32 * 0.0001;
        //There were some invalid rudder values, so here a sanity check
        if (value <= PI as f32) && (value >= -PI as f32){
            s.rudder_angle =  value * 360.0 / 2.0 / PI as f32;
        }
    }
}*/