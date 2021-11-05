use super::types::*;

#[derive(Default)]
pub struct Packet{
    /// Time of the Packet
    pub timestamp: Timestamp,
    /// Priority
    pub prio: TPrio,
    /// Source
    pub src: TSrc,
    /// Destination
    pub dest: TDest,
    /// Databytes
    pub data: TData,
    
    /// Masks the counter value for subsequent packages
    pub counter_mask : u8,         
    /// Next packet number we expect
    pub next_packet : u8,          
    /// Remaining bytes until the packet is complete
    pub remaining_bytes : usize,   
}
impl Packet{
    /// New empty packet
    pub fn new() -> Self{
        Packet{..Default::default()}
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
