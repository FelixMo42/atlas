use std::{
    fs,
    io::prelude::*,
    net::{TcpListener, TcpStream},
    str::from_utf8,
    thread,
};

pub fn start() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            thread::spawn(|| handle_connection(Request::new(stream)));
        } else {
            println!("ERR stream connection failed");
        }
    }
}

struct Request {
    stream: TcpStream,
    buffer: Vec<u8>,
    length: usize,
    read: usize,
}

impl Request {
    fn new(stream: TcpStream) -> Self {
        return Request {
            stream,
            buffer: vec![],
            length: 0,
            read: 0,
        };
    }

    fn has_line(&self) -> bool {
        from_utf8(&self.buffer[self.read..]).unwrap().contains('\n')
    }

    fn next_line(&mut self) -> &str {
        while !self.has_line() {
            self.buffer.append(&mut vec![0; 128]);
            let len = self.stream.read(&mut self.buffer[self.length..]).unwrap();
            self.length += len;
        }

        let str = from_utf8(&self.buffer[self.read..]).unwrap();
        let line = str.lines().next().unwrap();
        self.read += line.len() + 2;
        return line;
    }

    fn next_n(&mut self, length: usize) -> &str {
        if self.length - self.read < length {
            let new_length = usize::max(self.buffer.len(), self.read + length);
            self.buffer.resize(new_length, 0);
            let len = self.stream.read(&mut self.buffer[self.length..]).unwrap();
            self.length += len;
        }

        return from_utf8(&self.buffer[self.read..self.read + length]).unwrap();
    }

    fn header(&mut self) -> &str {
        return self.next_line();
    }

    fn content(&mut self) -> &str {
        let mut content_length = 0;

        loop {
            let line = self.next_line();

            if line.is_empty() {
                break;
            }

            let header_name = "Content-Length: ";
            if line.starts_with(header_name) {
                content_length = line[header_name.len()..].parse().unwrap();
            }
        }

        return self.next_n(content_length);
    }
}

fn handle_connection(mut request: Request) {
    let (status_line, contents) = match request.header() {
        "GET / HTTP/1.1" => {
            let content = fs::read_to_string("pub/index.html").unwrap();
            ("HTTP/1.1 200 OK", content)
        }
        "POST /parse HTTP/1.1" => {
            let content = request.content().to_string();
            match crate::wasm::compile(content.as_str()) {
                Ok(wat) => ("HTTP/1.1 200 OK", from_utf8(&wat).unwrap().to_string()),
                Err(..) => ("HTTP/1.1 200 OK", "error".to_string()),
            }
        }
        _ => {
            let content = fs::read_to_string("pub/404.html").unwrap();
            ("HTTP/1.1 404 NOT FOUND", content)
        }
    };

    // request.content();

    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    request.stream.write_all(response.as_bytes()).unwrap();
}
