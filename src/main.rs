//#![allow(dead_code,unused_imports)]
mod state;
mod udpstream;
mod nmea;

use crate::state::State;
use crate::udpstream::UdpStream;
use crate::nmea::nmea2000;

use std::fs::File;
use std::io::{BufReader, BufRead, BufWriter, Write};
use std::time::{Instant};

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "SailStats Logger v0.1.0a", 
            about = "NMEA 2000 logger for navigational messages in YDWG-Raw format.")]
struct Opt{
    /// Input filename
    #[structopt(short="f", long="file", name="INPUT", parse(from_os_str))]
    input_file: Option<PathBuf>,
    
    /// Listen to port for incoming packets [default: 1457]
    #[structopt(short, long, conflicts_with="INPUT")]
    port: Option<u16>,

    /// Interval at which status line is printed in milliseconds when listening for packets
    #[structopt(short, long, default_value="10")]
    interval: u128,

    /// Output filename
    #[structopt(short="o", long="output", name="OUTPUT", parse(from_os_str))]
    output_file: Option<PathBuf>
}

///****************************************************************************
/// Main
///****************************************************************************
fn main() -> std::io::Result<()> {
    let opt = Opt::from_args();

    let in_stream: Box<dyn std::io::Read>;
    let out_stream: Box<dyn std::io::Write>;
    let reading_from_file: bool;
    let writing_to_file: bool;
    
    //Input args
    if let Some(f) = opt.input_file{
        in_stream = Box::new(File::open(f.to_str().unwrap())?);
        reading_from_file = true;
    } else{
        in_stream = Box::new(UdpStream::open(
            format!("0.0.0.0:{}",
                match opt.port {
                    Some(port) => port.to_string(),
                    None => "1457".to_string(),
                }))?);
        reading_from_file = false;
    }

    //Output args
    if let Some(f) = opt.output_file{
        out_stream = Box::new(File::create(f.to_str().unwrap())?);
        writing_to_file = true;
    }else{
        out_stream = Box::new(std::io::stdout());
        writing_to_file = false;
    }

    let reader = BufReader::new(in_stream);
    let mut writer = BufWriter::new(out_stream);

    let mut parser = nmea2000::Parser2::<nmea2000::yd::Raw>::new();
    let mut state = State::new();

    //Write the headline
    writer.write_all(format!("{}\n",State::headline()).as_bytes())?;
    
    //If we are writing to stdout flush immediately
    if !writing_to_file{ writer.flush()?; } 

    //Start timer for the print out interval
    let mut time : Instant = Instant::now();
    for line in reader.lines(){
        if let Some(message) = parser.parse_string(&line?).unwrap(){
            message.update(&mut state);
            if time.elapsed().as_millis() >= opt.interval || reading_from_file {
                writer.write_all(
                    format!("{}\n", state)
                    .as_bytes())?;
                if !writing_to_file{ writer.flush()?; }
                time = Instant::now();
            }
        }
    }
    Ok(())
}
