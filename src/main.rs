use std::{fs, thread};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use simple_http_server::ThreadPool;

fn main() {
    // Initialize the TCP listener
    let listener = match TcpListener::bind("127.0.0.1:80") {
        Ok(listener) => listener,
        Err(error) => panic!("Problem binding to address: {:?}", error),
    };

    // Initialize a thread pool with 4 threads
    let pool = ThreadPool::new(4);

    // Iterate over incoming open connection (or streams) and handle each one in turn - take 4 at most
    // The server handles each stream by reading the request then preparing and sending a response
    // and then closing the connection
    for stream in listener.incoming().take(4) {
        // A stream (or connection) is actually a connection attempt, meaning that connection may
        // not be successful for a number of reasons mostly OS related
        let stream = stream.unwrap_or_else(|error| {
            panic!("Problem processing incoming connection: {:?}", error)
        });

        pool.execute(|| {
            handle_connection(stream);
        });
    }
    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    // Read the request and prepare the appropriate response
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();



    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "index.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "index.html")
        },
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    let contents = match fs::read_to_string(filename) {
        Ok(file) => file,
        Err(error) => panic!("Problem reading file: {:?}", error),
    };
    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    // Send back the response
    stream.write_all(response.as_bytes()).unwrap_or_else(|error| {
        panic!("Problem sending response: {:?}", error)
    });
}
