//! nmea2000 Message types 
use crate::nmea::types::{TData, TDest, TPgn, TPrio, TSrc, Timestamp};
use crate::nmea::nmea2000;

use crate::nmea::Float::*;
use crate::nmea::MessageValue;
use crate::nmea::MessageValue::*;

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

        impl nmea2000::MessageData for $type_name{
            #[inline(always)]
            fn timestamp(&self) -> Timestamp {self.timestamp}
            #[inline(always)]
            fn timestamp_mut(&mut self) -> &mut Timestamp {&mut self.timestamp}
            #[inline(always)]
            fn src(&self) -> TSrc {self.src}
            #[inline(always)]
            fn src_mut(&mut self) -> &mut TSrc {&mut self.src}
            #[inline(always)]
            fn dest(&self) -> TDest {self.dest}
            #[inline(always)]
            fn dest_mut(&mut self) -> &mut TDest {&mut self.dest}
            #[inline(always)]
            fn prio(&self) -> TPrio {self.prio}
            #[inline(always)]
            fn prio_mut(&mut self) -> &mut TPrio {&mut self.prio}
            #[inline(always)]
            fn data(&self) -> &TData {&self.data}
            #[inline(always)]
            fn data_mut(&mut self) -> &mut TData {&mut self.data}

            fn pgn(&self) -> TPgn {$type_name::PGN}
            #[inline(always)]
            fn bytes(&self) -> usize {$type_name::BYTES}
            #[inline(always)]
            fn is_fast(&self) -> bool {$type_name::FAST}
            #[inline(always)]
            fn is_complete(&self) -> bool{
                match self.remaining_bytes{
                    0 => true,
                    _ => false
                }
            }

            fn counter_mask(&self) -> u8 {self.counter_mask}
            #[inline(always)]
            fn counter_mask_mut(&mut self) -> &mut u8 {&mut self.counter_mask}
            #[inline(always)]
            fn next_packet(&self) -> u8 {self.next_packet}
            #[inline(always)]
            fn next_packet_mut(&mut self) -> &mut u8 {&mut self.next_packet}
            #[inline(always)]
            fn remaining_bytes(&self) -> usize {self.remaining_bytes}
            #[inline(always)]
            fn remaining_bytes_mut(&mut self) -> &mut usize {&mut self.remaining_bytes}
        }
    }
}

message_type!(WindMessage, 130306, 8, false);
impl nmea2000::Message for WindMessage{
    fn values(&self) -> Vec<MessageValue>{
        let aws = u16::from_le_bytes([self.data[1],self.data[2]]) as f32 * 0.01;
        let awa = u16::from_le_bytes([self.data[3],self.data[4]]) as f32 * 0.0001;
        vec![WindSpeed(F16(aws)), 
             WindAngle(F16(awa)),
             Timestamp(self.timestamp)]
    }
}

message_type!(PositionRapidUpdateMessage, 129025, 8, false);
impl nmea2000::Message for PositionRapidUpdateMessage{
    ///Latitude & longitude 
    fn values(&self) -> Vec<MessageValue>{
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

        vec![Latitude(F32(lat)), 
             Longitude(F32(long)),
             Timestamp(self.timestamp)]
    }
}

message_type!(GNSSPositionData, 129029, 43, true);
impl nmea2000::Message for GNSSPositionData{
    ///Latitude and longitude in degrees
    fn values(&self) -> Vec<MessageValue>{
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
        vec![Latitude(F64(lat)), 
             Longitude(F64(long)),
             Timestamp(self.timestamp)]
    }    
}

message_type!(VesselHeadingMessage, 127250, 8, false);
impl nmea2000::Message for VesselHeadingMessage{
    ///Heading value in rad
    fn values(&self) -> Vec<MessageValue>{
        let hdg = u16::from_le_bytes([self.data[1],self.data[2]]) as f32 * 0.0001;
        vec![Heading(F16(hdg)),
            Timestamp(self.timestamp)]
    }
}

message_type!(CogSogRapidUpdateMessage, 129026, 8, false);
impl nmea2000::Message for CogSogRapidUpdateMessage{
    ///Course over ground in rad, speed over ground in m/s
    fn values(&self) -> Vec<MessageValue>{
        let cog = u16::from_le_bytes([self.data[2],self.data[3]]) as f32 * 0.0001;
        let sog = u16::from_le_bytes([self.data[4],self.data[5]]) as f32 * 0.01;
        vec![CourseOverGround(F16(cog)), 
             SpeedOverGround(F16(sog)),
             Timestamp(self.timestamp)]
    }
}

message_type!(SpeedMessage, 128259, 8, false);
impl nmea2000::Message for SpeedMessage{
    ///Speed through water in m/s
    fn values(&self) -> Vec<MessageValue>{
        let stw = u16::from_le_bytes([self.data[1],self.data[2]]) as f32 * 0.01;
        vec![SpeedThroughWater(F16(stw)),
             Timestamp(self.timestamp)]
    }
}

message_type!(RateOfTurnMessage, 127251, 5, false);
impl nmea2000::Message for RateOfTurnMessage{
    ///Rate of turn in radians/s
    fn values(&self) -> Vec<MessageValue>{
        let rot = i32::from_le_bytes([self.data[1],
                                      self.data[2],
                                      self.data[3],
                                      self.data[4]]) as f32 * 3.125e-08;
        vec![RateOfTurn(F32(rot)),
             Timestamp(self.timestamp)]
    }
}

message_type!(AttitudeMessage, 127257, 7, false);
impl nmea2000::Message for AttitudeMessage{
    ///Yaw, pitch & roll in radians
    fn values(&self) -> Vec<MessageValue>{
        let yaw = i16::from_le_bytes([self.data[1],self.data[2]]) as f32 * 0.0001;
        let pitch = i16::from_le_bytes([self.data[3],self.data[4]]) as f32 * 0.0001;
        let roll = i16::from_le_bytes([self.data[5],self.data[6]]) as f32 * 0.0001;
        vec![Yaw(F16(yaw)),
             Pitch(F16(pitch)),
             Roll(F16(roll)),
             Timestamp(self.timestamp)]
    }
}

message_type!(RudderMessage, 127245, 8, false);
impl nmea2000::Message for RudderMessage{
    ///Rudder angle in radians
    fn values(&self) -> Vec<MessageValue>{
        let ra = i16::from_le_bytes([self.data[4],self.data[5]]) as f32 * 0.0001;
        vec![RudderAngle(F16(ra)),
             Timestamp(self.timestamp)]
    }
}