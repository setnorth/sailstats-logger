use crate::state::State;
use crate::packet::{Packet,PacketAccess};
use crate::types::*;

use std::f64::consts::PI;

/// Messages can update the state
pub trait Message : PacketAccess{
    fn update(&self, s: &mut State);
}

//*****************************************************************************
// Message types
//*****************************************************************************
/// Creates a message type that implements the trait PacketAccess
macro_rules! message_type {
    ($type_name: ident, $pgn: expr, $bytes: expr, $fast: expr) => {
        pub struct $type_name {
            p: Packet
        }
        impl $type_name{
            pub const PGN: TPgn = $pgn;
            pub const BYTES: usize = $bytes;
            pub const FAST: bool = $fast;
            pub fn new() -> Self{ $type_name{p: Packet::new()} }
        }
        impl PacketAccess for $type_name{
            fn packet(&self) -> &Packet{ &self.p }
            fn packet_mut(&mut self) -> &mut Packet{ &mut self.p }
            fn pgn(&self) -> TPgn{ $type_name::PGN }
            fn bytes(&self) -> usize{ $type_name::BYTES }
            fn fast(&self) -> bool{ $type_name::FAST }
        } 
    }
}

message_type!(WindMessage, 130306, 8, false);
impl Message for WindMessage{
    fn update(&self, s: &mut State){
        s.timestamp = self.p.timestamp;
        s.aws = u16::from_le_bytes([self.p.data[1],self.p.data[2]]) as f32 * 0.01 * 1.943_844_6; //in knots;
        s.awa = u16::from_le_bytes([self.p.data[3],self.p.data[4]]) as f32 * 0.0001 * 360.0 / 2.0 / PI as f32;
    }
}

message_type!(PositionRapidUpdateMessage, 129025, 8, false);
impl Message for PositionRapidUpdateMessage{
    fn update(&self, s: &mut State){
        s.timestamp = self.p.timestamp;
        let mut lat = i32::from_le_bytes([  
            self.p.data[0],
            self.p.data[1],
            self.p.data[2],
            self.p.data[3]]) as f32;
        lat *= 0.0000001;

        let mut long = i32::from_le_bytes([
            self.p.data[4],
            self.p.data[5],
            self.p.data[6],
            self.p.data[7]]) as f32;
        long *= 0.0000001;
        s.latitude= lat;
        s.longitude = long;
    }
}

message_type!(GNSSPositionData, 129029, 43, true);
impl Message for GNSSPositionData{
    fn update(&self, s : &mut State){
        s.timestamp = self.p.timestamp;
        //Latitude
        let mut lat = i64::from_le_bytes([ 
            self.p.data[7],
            self.p.data[8],
            self.p.data[9],
            self.p.data[10],
            self.p.data[11],
            self.p.data[12],
            self.p.data[13],
            self.p.data[14]]) as f64;
        lat *= 0.0000000000000001;
        //Longitude
        let mut long = i64::from_le_bytes([ 
            self.p.data[15],
            self.p.data[16],
            self.p.data[17],
            self.p.data[18],
            self.p.data[19],
            self.p.data[20],
            self.p.data[21],
            self.p.data[22]]) as f64;
        long *= 0.0000000000000001;
        s.latitude = lat as f32;
        s.longitude = long as f32;
    }
}

message_type!(VesselHeadingMessage, 127250, 8, false);
impl Message for VesselHeadingMessage{
    fn update(&self, s: &mut State){
        s.timestamp = self.p.timestamp;
        s.hdg = u16::from_le_bytes([self.p.data[1],self.p.data[2]]) as f32 * 0.0001 * 360.0 / 2.0 / PI as f32;
    }
}

message_type!(CogSogRapidUpdateMessage, 129026, 8, false);
impl Message for CogSogRapidUpdateMessage{
    fn update(&self, s: &mut State){
        s.timestamp = self.p.timestamp;
        s.cog = u16::from_le_bytes([self.p.data[2],self.p.data[3]]) as f32 * 0.0001 * 360.0 / 2.0 / PI as f32;
        s.sog = u16::from_le_bytes([self.p.data[4],self.p.data[5]]) as f32 * 0.01 * 1.943_844_6; //in knots
    }
}

message_type!(SpeedMessage, 128259, 8, false);
impl Message for SpeedMessage{
    fn update(&self, s: &mut State){
        s.timestamp = self.p.timestamp;
        s.stw = u16::from_le_bytes([self.p.data[1],self.p.data[2]]) as f32 * 0.01 * 1.943_844_6; //in knots
    }
}

message_type!(RateOfTurnMessage, 127251, 5, false);
impl Message for RateOfTurnMessage{
    fn update(&self, s: &mut State){
        s.timestamp = self.p.timestamp;
        s.rot = i32::from_le_bytes([self.p.data[1],
                                    self.p.data[2],
                                    self.p.data[3],
                                    self.p.data[4]]) as f32 * 3.125e-08 * 360.0 / 2.0 / PI as f32;
    }
}

message_type!(AttitudeMessage, 127257, 7, false);
impl Message for AttitudeMessage{
    fn update(&self, s: &mut State){
        s.timestamp = self.p.timestamp;
        s.yaw = i16::from_le_bytes([self.p.data[1],self.p.data[2]]) as f32 * 0.0001 * 360.0 / 2.0 / PI as f32;
        s.pitch = i16::from_le_bytes([self.p.data[3],self.p.data[4]]) as f32 * 0.0001 * 360.0 / 2.0 / PI as f32;
        s.roll = i16::from_le_bytes([self.p.data[5],self.p.data[6]]) as f32 * 0.0001 * 360.0 / 2.0 / PI as f32;
    }
}

message_type!(RudderMessage, 127245, 8, false);
impl Message for RudderMessage{
    fn update(&self, s: &mut State){
        s.timestamp = self.p.timestamp;
        let value = i16::from_le_bytes([self.p.data[4],self.p.data[5]]) as f32 * 0.0001;
        //There were some invalid rudder values, so here a sanity check
        if (value <= PI as f32) && (value >= -PI as f32){
            s.rudder_angle =  value * 360.0 / 2.0 / PI as f32;
        }
    }
}