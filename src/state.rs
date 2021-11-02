use std::fmt;

/// State that keeps the latest values
pub struct State{
    pub timestamp : (u8,u8,f32),
    pub awa : f32,
    pub aws : f32,
    pub latitude : f32,
    pub longitude : f32,
    pub hdg : f32,
    pub cog : f32,
    pub sog : f32,
    pub stw : f32,
    pub rot : f32,
    pub pitch : f32,
    pub yaw : f32,
    pub roll : f32,
    pub rudder_angle : f32,
}
impl State {
    pub fn new() -> State{
        State{
            timestamp : (0, 0, 0.0),
            awa : 0.0,
            aws : 0.0,
            latitude : 0.0,
            longitude : 0.0,
            hdg : 0.0,
            cog : 0.0,
            sog : 0.0,
            stw : 0.0,
            rot : 0.0,
            pitch : 0.0,
            yaw : 0.0,
            roll : 0.0,
            rudder_angle : 0.0
        }
    }
    pub fn headline() -> String{
        String::from("time;awa;aws;latitude;longitude;hdg;cog;sog;stw;rot;pitch;yaw;roll;rudder_angle")
    }
}
impl fmt::Display for State{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
        write!(f,
            "{:02}:{:02}:{:0>6.3};{:.1};{:.2};{};{};{:.2};{:.2};{:.2};{:.2};{:.2};{:.2};{:.2};{:.2};{:.2}",
            self.timestamp.0, self.timestamp.1, self.timestamp.2,self.awa,self.aws,
            self.latitude,self.longitude,self.hdg,self.cog,self.sog,self.stw,
            self.rot,self.pitch,self.yaw,self.roll,self.rudder_angle)
    }
}
