pub mod nmea2000;
pub mod types;


pub enum Float{
    F16(f32),
    F32(f32),
    F64(f64)
}

/// Value of a NMEA message
pub enum MessageValue{
    WindAngle(Float),
    WindSpeed(Float),
    Latitude(Float),
    Longitude(Float),
    Heading(Float),
    CourseOverGround(Float),
    SpeedOverGround(Float),
    SpeedThroughWater(Float),
    RateOfTurn(Float),
    Yaw(Float),
    Pitch(Float),
    Roll(Float),
    RudderAngle(Float),
    Timestamp(types::Timestamp),
    Date(u16), //Days since 1.1.1970
    Time(f32), //Seonds since midnight
    LocalOffset(i16) //Local offset in minutes
}