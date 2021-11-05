use crate::state::State;
use crate::types::*;

use std::f64::consts::PI;

/// Messages can update the state and must contain data
pub trait Message : MessageData{
    fn update(&self, s: &mut State);
}

/// Properties of a message
pub trait MessageData{
    /// Byte length
    fn bytes(&self) -> usize;
    /// Parameter group
    fn pgn(&self) -> TPgn;
    /// Indicator if they are "fast" messages
    fn fast(&self) -> bool;
    /// Check if they are complete
    fn complete(&self) -> bool;
    
    fn timestamp(&self) -> Timestamp;
    fn mut_timestamp(&mut self) -> &mut Timestamp;

    fn prio(&self) -> TPrio;
    fn mut_prio(&mut self) -> &mut TPrio;

    fn src(&self) -> TSrc;
    fn mut_src(&mut self) -> &mut TSrc;

    fn dest(&self) -> TDest;
    fn mut_dest(&mut self) -> &mut TDest;
    
    fn data(&self) -> &TData;
    fn mut_data(&mut self) -> &mut TData;

    fn counter_mask(&self) -> u8;
    fn mut_counter_mask(&mut self) -> &mut u8;

    fn next_packet(&self) -> u8;
    fn mut_next_packet(&mut self) -> &mut u8;
    
    fn remaining_bytes(&self) -> usize;
    fn mut_remaining_bytes(&mut self) -> &mut usize;
}

//*****************************************************************************
// Message types
//*****************************************************************************

/// Creates a message type that implements the trait PacketAccess
macro_rules! message_type {
    ($type_name: ident, $pgn: expr, $bytes: expr, $fast: expr) => {
        #[derive(Default)]
        pub struct $type_name {
            //p: Packet,

            /// Time of the Message
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
            /// Remaining bytes until the Message is complete
            pub remaining_bytes : usize,               
        }
        impl $type_name{
            pub const PGN: TPgn = $pgn;
            pub const BYTES: usize = $bytes;
            pub const FAST: bool = $fast;

            pub fn new() -> Self{ $type_name{..Default::default()} }
        }
        impl MessageData for $type_name{
            fn bytes(&self) -> usize{ $type_name::BYTES }
            fn pgn(&self) -> TPgn{ $type_name::PGN }
            fn fast(&self) -> bool{ $type_name::FAST }
            fn complete(&self) -> bool{ self.remaining_bytes == 0 }

            fn timestamp(&self) -> Timestamp{ self.timestamp }
            fn mut_timestamp(&mut self) -> &mut Timestamp { &mut self.timestamp }
        
            fn prio(&self) -> TPrio { self.prio }
            fn mut_prio(&mut self) -> &mut TPrio { &mut self.prio }
        
            fn src(&self) -> TSrc { self.src }
            fn mut_src(&mut self) -> &mut TSrc { &mut self.src }
        
            fn dest(&self) -> TDest { self.dest }
            fn mut_dest(&mut self) -> &mut TDest { &mut self.dest }
            
            fn data(&self) -> &TData { &self.data }
            fn mut_data(&mut self) -> &mut TData { &mut self.data }
        
            fn counter_mask(&self) -> u8 { self.counter_mask }
            fn mut_counter_mask(&mut self) -> &mut u8 {&mut self.counter_mask }
        
            fn next_packet(&self) -> u8 { self.next_packet }
            fn mut_next_packet(&mut self) -> &mut u8 { &mut self.next_packet }
            
            fn remaining_bytes(&self) -> usize { self.remaining_bytes }
            fn mut_remaining_bytes(&mut self) -> &mut usize { &mut self.remaining_bytes }
        }
    }
}

message_type!(WindMessage, 130306, 8, false);
impl Message for WindMessage{
    fn update(&self, s: &mut State){
        s.timestamp = self.timestamp;
        s.aws = u16::from_le_bytes([self.data[1],self.data[2]]) as f32 * 0.01 * 1.943_844_6; //in knots;
        s.awa = u16::from_le_bytes([self.data[3],self.data[4]]) as f32 * 0.0001 * 360.0 / 2.0 / PI as f32;
    }
}

message_type!(PositionRapidUpdateMessage, 129025, 8, false);
impl Message for PositionRapidUpdateMessage{
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
impl Message for GNSSPositionData{
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
impl Message for VesselHeadingMessage{
    fn update(&self, s: &mut State){
        s.timestamp = self.timestamp;
        s.hdg = u16::from_le_bytes([self.data[1],self.data[2]]) as f32 * 0.0001 * 360.0 / 2.0 / PI as f32;
    }
}

message_type!(CogSogRapidUpdateMessage, 129026, 8, false);
impl Message for CogSogRapidUpdateMessage{
    fn update(&self, s: &mut State){
        s.timestamp = self.timestamp;
        s.cog = u16::from_le_bytes([self.data[2],self.data[3]]) as f32 * 0.0001 * 360.0 / 2.0 / PI as f32;
        s.sog = u16::from_le_bytes([self.data[4],self.data[5]]) as f32 * 0.01 * 1.943_844_6; //in knots
    }
}

message_type!(SpeedMessage, 128259, 8, false);
impl Message for SpeedMessage{
    fn update(&self, s: &mut State){
        s.timestamp = self.timestamp;
        s.stw = u16::from_le_bytes([self.data[1],self.data[2]]) as f32 * 0.01 * 1.943_844_6; //in knots
    }
}

message_type!(RateOfTurnMessage, 127251, 5, false);
impl Message for RateOfTurnMessage{
    fn update(&self, s: &mut State){
        s.timestamp = self.timestamp;
        s.rot = i32::from_le_bytes([self.data[1],
                                    self.data[2],
                                    self.data[3],
                                    self.data[4]]) as f32 * 3.125e-08 * 360.0 / 2.0 / PI as f32;
    }
}

message_type!(AttitudeMessage, 127257, 7, false);
impl Message for AttitudeMessage{
    fn update(&self, s: &mut State){
        s.timestamp = self.timestamp;
        s.yaw = i16::from_le_bytes([self.data[1],self.data[2]]) as f32 * 0.0001 * 360.0 / 2.0 / PI as f32;
        s.pitch = i16::from_le_bytes([self.data[3],self.data[4]]) as f32 * 0.0001 * 360.0 / 2.0 / PI as f32;
        s.roll = i16::from_le_bytes([self.data[5],self.data[6]]) as f32 * 0.0001 * 360.0 / 2.0 / PI as f32;
    }
}

message_type!(RudderMessage, 127245, 8, false);
impl Message for RudderMessage{
    fn update(&self, s: &mut State){
        s.timestamp = self.timestamp;
        let value = i16::from_le_bytes([self.data[4],self.data[5]]) as f32 * 0.0001;
        //There were some invalid rudder values, so here a sanity check
        if (value <= PI as f32) && (value >= -PI as f32){
            s.rudder_angle =  value * 360.0 / 2.0 / PI as f32;
        }
    }
}