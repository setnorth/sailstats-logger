//! State of the navigational data.
use std::fmt;
use crate::nmea::types::Timestamp;
use crate::nmea::nmea2000;
use crate::nmea::{MessageValue,Float};
use std::f64::consts::PI;

/// Keeps the latest values of the navigational data.
#[derive(Default)]
pub struct State{
    /// Time of latest update to the state
    pub timestamp : Timestamp,
    /// Apparent wind angle in degrees
    pub awa : f32,
    /// Apparent wind speed in knots
    pub aws : f32,
    /// Latitude
    pub latitude : f32,
    /// Longitude
    pub longitude : f32,
    /// Heading in degrees
    pub hdg : f32,
    /// Course over ground in degrees
    pub cog : f32,
    /// Speed over ground in knots
    pub sog : f32,
    /// Speed through water in knots
    pub stw : f32,
    /// Rate of turn in degrees/s
    pub rot : f32,
    /// Pitch angle in degrees
    pub pitch : f32,
    /// Yaw angle in degrees
    pub yaw : f32,
    /// Roll angle in degrees, i.e., heel angle
    pub roll : f32,
    /// Angle of rudder deflection in degrees
    pub rudder_angle : f32,
}

/// Helper function to convert between radians and degrees
#[inline(always)]
fn to_degrees(val: f32) -> f32{
    val * 360.0 / 2.0 / PI as f32
}

/// Helper function to convert between m/s and knots
#[inline(always)]
fn to_knots(val: f32) -> f32{
    val * 1.943_844_6
}

impl State {
    /// Create new empty State
    pub fn new() -> State{
        State{..Default::default()}
    }
    /// Print the headline for a CSV document containig all fields seperated by `;`
    pub fn headline() -> String{
        String::from("time;awa;aws;latitude;longitude;hdg;cog;sog;stw;rot;pitch;yaw;roll;rudder_angle")
    }
    /// Update the state with a nmea message value
    pub fn update(&mut self, message: Box<dyn nmea2000::Message>){
        for entry in message.values(){
            match entry{
                MessageValue::Timestamp(t) => self.timestamp = t,
                MessageValue::WindSpeed(Float::F16(aws)) => self.aws = to_knots(aws),
                MessageValue::WindAngle(Float::F16(awa)) => self.awa = to_degrees(awa),
                MessageValue::Latitude(Float::F32(lat)) => self.latitude = lat,
                MessageValue::Longitude(Float::F32(long)) => self.longitude = long,
                MessageValue::Latitude(Float::F64(lat)) => self.latitude = lat as f32,
                MessageValue::Longitude(Float::F64(long)) => self.longitude = long as f32,
                MessageValue::Heading(Float::F16(hdg)) => self.hdg = to_degrees(hdg),
                MessageValue::CourseOverGround(Float::F16(cog)) => self.cog = to_degrees(cog),
                MessageValue::SpeedOverGround(Float::F16(sog)) => self.sog = to_knots(sog),
                MessageValue::SpeedThroughWater(Float::F16(stw)) => self.stw = to_knots(stw),
                MessageValue::RateOfTurn(Float::F32(rot)) => self.rot = to_degrees(rot),
                MessageValue::Yaw(Float::F16(yaw)) => self.yaw = to_degrees(yaw),
                MessageValue::Pitch(Float::F16(pitch)) => self.pitch = to_degrees(pitch),
                MessageValue::Roll(Float::F16(roll)) => self.roll = to_degrees(roll),
                //sanity check if plausible value for rudder angle
                MessageValue::RudderAngle(Float::F16(ra)) => if(ra <= PI as f32) && (ra >= -PI as f32){
                                                                self.rudder_angle = to_degrees(ra);
                                                             }
                _ => unimplemented!(),
            }
        }
    }
}

/// Display state implementation for CSV document with separator `;`
impl fmt::Display for State{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        write!(f,
            "{:02}:{:02}:{:0>6.3};{:.1};{:.2};{};{};{:.2};{:.2};{:.2};{:.2};{:.2};{:.2};{:.2};{:.2};{:.2}",
            self.timestamp.0, self.timestamp.1, self.timestamp.2,self.awa,self.aws,
            self.latitude,self.longitude,self.hdg,self.cog,self.sog,self.stw,
            self.rot,self.pitch,self.yaw,self.roll,self.rudder_angle)
    }
}
