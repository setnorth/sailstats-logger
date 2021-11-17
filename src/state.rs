//! State of the navigational data.
use crate::nmea::types::Timestamp;
use crate::nmea::nmea2000;
use crate::nmea::{MessageValue,Float};

use std::f64::consts::PI;
use std::fmt;
use std::time::SystemTime;

use chrono::{Datelike,NaiveDateTime};

/// Keeps the latest values of the navigational data.
pub struct State{
    /// Date & time from system
    pub date_time : chrono::NaiveDateTime,
    /// Days since January 1 1970
    pub days : u16,
    /// Seconds since midnight
    pub seconds: f32,
    /// Local offset in minutes
    pub localoffset: i16,
    /// Timestamp of latest update to the state
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

    /// Flag if we should use the date that is propagated by
    /// the NMEA bus instead of systime. This is useful if the 
    /// device on which the logger is running on does not have a 
    /// reliable clock installed.
    pub nmea_date : bool,
    /// Flag if we have received a date/time value completely,
    /// i.e., we know that when we have read "localoffset".
    pub got_nmea_date: bool
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

/// Helper function to convert days, seconds and offset to a NaiveDateTime
#[inline(always)]
fn to_date_time(days: u16, seconds: f32, localoffset: i16) -> NaiveDateTime{
    NaiveDateTime::from_timestamp(days as i64 * 86_400 
                                  + seconds as i64
                                  + (localoffset * 60) as i64,0)
}

impl State {
    /// Create new empty State
    pub fn new(nmea_date: bool) -> State{
        State{
            date_time: NaiveDateTime::from_timestamp(0,0),
            days: 0,
            seconds: 0.0,
            localoffset: 0,
            timestamp: (0,0,0.0),
            awa: 0.0,
            aws: 0.0,
            latitude: 0.0,
            longitude: 0.0,
            hdg: 0.0,
            cog: 0.0,
            sog: 0.0,
            stw: 0.0,
            rot: 0.0,
            pitch: 0.0,
            yaw: 0.0,
            roll: 0.0,
            rudder_angle: 0.0,
            nmea_date: nmea_date,
            got_nmea_date: false, 
        }
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
                MessageValue::Date(d) => {  self.days = d;
                                            self.date_time = to_date_time(self.days, self.seconds, self.localoffset) ; 
                                         }
                MessageValue::Time(t) => {  self.seconds = t;
                                            self.date_time = to_date_time(self.days, self.seconds, self.localoffset) ; 
                                         }
                MessageValue::LocalOffset(o) => {   self.localoffset = o;
                                                    self.date_time = to_date_time(self.days, self.seconds, self.localoffset) ; 
                                                    self.got_nmea_date = true;
                                                }
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
        if self.nmea_date{
            //Check if we can write out something, i.e., if we have read some nmea date
            if self.got_nmea_date{
                write!(f,
                    "{:04}-{:02}-{:02} {:02}:{:02}:{:0>6.3};{:.1};{:.2};{};{};{:.2};{:.2};{:.2};{:.2};{:.2};{:.2};{:.2};{:.2};{:.2}\n",
                    self.date_time.year(),self.date_time.month(),self.date_time.day(),self.timestamp.0, self.timestamp.1, self.timestamp.2,self.awa,self.aws,
                    self.latitude,self.longitude,self.hdg,self.cog,self.sog,self.stw,
                    self.rot,self.pitch,self.yaw,self.roll,self.rudder_angle)
            }else{
                Ok(())
            }
        }else{
            let t = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
            let date_time = NaiveDateTime::from_timestamp(t.as_secs() as i64,0);
            write!(f,
                "{:04}-{:02}-{:02} {:02}:{:02}:{:0>6.3};{:.1};{:.2};{};{};{:.2};{:.2};{:.2};{:.2};{:.2};{:.2};{:.2};{:.2};{:.2}\n",
                date_time.year(),date_time.month(),date_time.day(),self.timestamp.0, self.timestamp.1, self.timestamp.2,self.awa,self.aws,
                self.latitude,self.longitude,self.hdg,self.cog,self.sog,self.stw,
                self.rot,self.pitch,self.yaw,self.roll,self.rudder_angle)
        }
    }
}