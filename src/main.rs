use std::{
    net::{
        TcpListener,
        UdpSocket,
        SocketAddr
    },
    thread,
    io::{
        Result,
        Write,
        Error,
        ErrorKind,
        BufRead,
        BufReader
    },
    env::args,
    fs::File,
    sync::{
        Arc,
        Mutex
    }, time::SystemTime
};

struct Quote {
    quote: String,
    name: String
}

const SECONDS_IN_DAY: usize = 86400;

fn send_quote<T: Write>(client: &mut T, quotes: &[Quote]) -> Result<()> {
    let time = SystemTime::now();
    let diff = time.duration_since(SystemTime::UNIX_EPOCH)
        .expect("this program is best designed for use after January 1, 1970");
    let day = (diff.as_secs() as usize) / SECONDS_IN_DAY;
    let idx = day % quotes.len();
    let quote_of_the_day = &quotes[idx];
    write!(client, "\"{}\"\n\n\t-- {}\n", quote_of_the_day.quote, quote_of_the_day.name)?;
    Ok(())
}

struct UdpConn<'a>(&'a UdpSocket, SocketAddr);

impl Write for UdpConn<'_> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.0.send_to(buf, self.1)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = args().collect();
    if args.len() != 2 && args.len() != 3 {
        eprintln!("syntax: {} <quotes file> <port (default: 10017)>", args[0]);
        return Err(Error::from(ErrorKind::NotFound));
    }
    let port = if args.len() == 3 { args[2].parse::<u16>().expect("invalid port number") } else { 10017 };

    let quotes_file = &args[1];
    let quotes_file = File::open(quotes_file)?;
    let quotes_file = BufReader::new(quotes_file);
    let mut quotes = Vec::new();
    
    for line in quotes_file.lines() {
        match line {
            Ok(line) => {
                let split: Vec<&str> = line.split('|').collect();
                if split.len() != 2 {
                    eprintln!("invalid line in quotes file:\n{}", line);
                    return Err(Error::from(ErrorKind::InvalidData));
                }
                quotes.push(Quote { quote: split[0].to_string(), name: split[1].to_string() });
            },
            Err(err) => {
                eprintln!("could not read quotes file");
                return Err(err);
            }
        }
    }

    let quotes = Arc::new(Mutex::new(quotes));

    let tcp_quotes = quotes.clone();
    let tcp_thread = thread::spawn(move || {
        let l = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 0], port)))?;
        println!("listening on TCP {}", l.local_addr().expect("could not read local addr"));
        for stream in l.incoming() {
            match stream {
                Ok(mut stream) => {
                    let quotes = tcp_quotes.lock().expect("quotes mutex poisoned");
                    match send_quote(&mut stream, &quotes) {
                        Ok(()) => (),
                        Err(err) => eprintln!("TCP request: {}", err)
                    };
                    
                },
                Err(err) => eprintln!("TCP accept: {}", err)
            };
        }
        Result::Ok(())
    });
    
    let udp_quotes = quotes.clone();
    let udp_thread = thread::spawn(move || -> Result<()> {
        let l = UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0, 0, 0, 0, 0], port)))?;
        println!("listening on UDP {}", l.local_addr().expect("could not read local addr"));
        loop {
            let mut buf = [0u8; 0];
            let (_, return_addr) = match l.recv_from(&mut buf) {
                Ok(res) => res,
                Err(err) => {
                    eprintln!("UDP inbound: {}", err);
                    continue;
                }
            };
            let mut conn = UdpConn(&l, return_addr);
            let quotes = udp_quotes.lock().expect("quotes mutex poisoned");
            match send_quote(&mut conn, &quotes) {
                Ok(()) => (),
                Err(err) => eprintln!("UDP outbound: {}", err)
            };   
        }
    });

    match tcp_thread.join() {
        Ok(res) => res?,
        Err(err) => panic!("TCP thread panicked: {:?}", err)
    };
    match udp_thread.join() {
        Ok(res) => res?,
        Err(err) => panic!("UDP thread panicked: {:?}", err)
    };
    Ok(())
}
