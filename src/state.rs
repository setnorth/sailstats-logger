//! State of the navigational data.
use std::fmt;
use crate::types::*;

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
impl State {
    /// Create new empty State
    pub fn new() -> State{
        State{..Default::default()}
    }
    /// Print the headline for a CSV document containig all fields seperated by `;`
    pub fn headline() -> String{
        String::from("time;awa;aws;latitude;longitude;hdg;cog;sog;stw;rot;pitch;yaw;roll;rudder_angle")
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
