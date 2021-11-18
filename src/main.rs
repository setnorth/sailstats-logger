//#![allow(dead_code,unused_imports)]
mod state;
mod udpstream;
mod nmea;

use crate::state::State;
use crate::udpstream::UdpStream;
use crate::nmea::nmea2000;

use std::fs::File;
use std::io::{BufReader, BufRead, BufWriter, Write};
use std::path::PathBuf;
use std::thread;
use std::sync::{Arc,Mutex};
use std::time::Duration;

use structopt::StructOpt;
use anyhow::{Context, Result};

#[derive(Debug, StructOpt)]
#[structopt(name = "SailStats Logger v0.1.0a", 
            about = "NMEA logger for navigational messages.")]
struct Opt{
    /// Input filename
    #[structopt(short="f", long="file", name="INPUT", parse(from_os_str))]
    input_file: Option<PathBuf>,
    
    /// Listen to port for incoming packets [default: 1457]
    #[structopt(short, long, conflicts_with="INPUT")]
    port: Option<u16>,

    /// Interval at which status line is printed in milliseconds when listening for packets
    #[structopt(short, long, default_value="250")]
    interval: u64,

    /// Output filename
    #[structopt(short="o", long="output", name="OUTPUT", parse(from_os_str))]
    output_file: Option<PathBuf>,
    
    /// Use date values that are propagated on the NMEA bus (default is system time, except when reading from file)
    #[structopt(short, long)]
    nmea_date: bool,
}

fn read_thread<T,U>(
    reader: BufReader<T>, 
    parser: &mut nmea2000::Parser<U,String>, 
    state: Arc<Mutex<State>>) -> Result<()>
    where
    T: std::io::Read,
    U: nmea::nmea2000::Raw + nmea::nmea2000::From<String> + Send,
    {
        for line in reader.lines(){
            if let Some(message) = parser.parse(&line.context("error processing line")?).context("error parsing line")?{
                state.lock().unwrap().update(message);
            }
        }
        Ok(())
}

fn write_thread<T: Write>(
    writer: &mut BufWriter<T>, 
    state: Arc<Mutex<State>>,
    interval: u64,
    writing_to_file: bool) -> Result<()>
    {
        //Write the headline
        writer.write_all(format!("{}\n",State::headline()).as_bytes()).context("unable to write headline")?;
        
        //If we are writing to stdout flush immediately
        if !writing_to_file{ writer.flush().context("unable to flush output")?; } 
        
        //Main writing loop
        loop{
            let s = state.lock().unwrap();
            writer.write_all(
                format!("{}", s)
                .as_bytes()).context("error writing output")?;
            drop(s);
            if !writing_to_file{ writer.flush().context("unable to flush output")?; }
            thread::sleep(Duration::from_millis(interval));
        }
}

fn main() -> Result<()> {
    /**************************************************************************
     * Program arguments
     **************************************************************************/
    let opt = Opt::from_args();
    let in_stream: Box<dyn std::io::Read+Send>;
    let out_stream: Box<dyn std::io::Write+Send>;
    let reading_from_file: bool;
    let writing_to_file: bool;
    let mut nmea_date: bool = opt.nmea_date; // Can be overwritten if reading from file
    
    //Input args
    if let Some(f) = opt.input_file{
        in_stream = Box::new(File::open(
                                    f.to_str().unwrap())
                                     .with_context(|| format!("unable to open {}",f.to_str().unwrap()))?
                                    );
        reading_from_file = true;
        nmea_date = true;
    } else{
        let port = match opt.port {
                    Some(port) => port.to_string(),
                    None => "1457".to_string(),
                };
        let address = format!("0.0.0.0:{}",port);
        in_stream = Box::new(
                        UdpStream::open(address.clone()).with_context(|| format!("could not open UDP listener on {}",address))?
                    );
        reading_from_file = false;
    }

    //Output args
    if let Some(f) = opt.output_file{
        out_stream = Box::new(
            File::create(f.to_str().unwrap()).with_context(|| format!("could not create file {}", f.to_str().unwrap()))?
        );
        writing_to_file = true;
    }else{
        out_stream = Box::new(std::io::stdout());
        writing_to_file = false;
    }

    /**************************************************************************
     * Main Program logic
     **************************************************************************/
    let reader = BufReader::new(in_stream);
    let mut writer = BufWriter::new(out_stream);

    let mut parser = nmea2000::Parser::<nmea2000::yd::Raw,String>::new();
    let mut state = State::new(nmea_date);

    if !reading_from_file{
        let state_arc = Arc::new(Mutex::new(state));

        let writer_state = Arc::clone(&state_arc);
        let writer_handle = thread::spawn(move || 
            write_thread(&mut writer, writer_state, opt.interval, writing_to_file)
        );

        let reader_state = Arc::clone(&state_arc);
        let reader_handle = thread::spawn(move ||
            read_thread(reader, &mut parser, reader_state)
        );
    
        writer_handle.join().unwrap()?;
        reader_handle.join().unwrap()?;
    }else{
        //Write the headline
        writer.write_all(format!("{}\n",State::headline()).as_bytes()).context("unable to write headline")?;
    
        //If we are writing to stdout flush immediately
        if !writing_to_file{ 
            writer.flush().context("unable to flush output")?; 
        } 

        for line in reader.lines(){
            if let Some(message) = parser.parse(&line.context("error processing line")?).context("error parsing line")?{
                state.update(message);
                    writer.write_all(
                        format!("{}", state)
                        .as_bytes()).context("error writing output")?;
                    if !writing_to_file{ 
                        writer.flush().context("unable to flush output")?; 
                    }
            }
        }
    }
    Ok(())
}
