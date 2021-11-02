use super::types::*;

#[derive(Debug)]
pub struct Packet{
    pub timestamp: Timestamp,
    pub prio: TPrio,
    pub src: TSrc,
    pub dest: TDest,
    pub data: TData,
    //  ----
    pub counter_mask : u8,          //Masks the counter value for subsequent packages
    pub next_packet : u8,           //Keeps track of the packets we received so far
    pub remaining_bytes : usize,    //Remaining bytes to read from raw
}
impl Packet{
    pub fn new() -> Self{
        Packet{
            timestamp: (0,0,0.0),
            prio: 0,
            src: 0,
            dest: 0,
            data: TData::new(),
            counter_mask: 0x00,
            next_packet: 0,
            remaining_bytes: 0,
        }
    }
}

/// Messages have a packet that can be accessed
pub trait PacketAccess{
    fn packet(&self) -> &Packet;
    fn packet_mut(&mut self) -> &mut Packet;
    fn bytes(&self) -> usize;
    fn pgn(&self) -> TPgn;
    fn fast(&self) -> bool;
    fn complete(&self) -> bool{ self.packet().remaining_bytes == 0 }
}
