/*
 CHATTEST PROTOCOL (port: 7357)
    Riccardo Ripanti xx/11/2019
 ===============================================================================

 SPECIFICATION:

 +----+
 |    | = 1 byte
 +----+

 Every message starts with a `code` which is an unsigned byte which specifies
 the contents of the message:

 - NAME (code 1)
      message is a name:

               +----+----+----+----+----+ - - - - - - - +
               |code|      length       |     name      |
               +----+----+----+----+----+ - - - - - - - +
                     MSB            LSB  <---length---->

      Clients send this type of message right after connecting to the server,
      putting their name inside. Server then responds with a code 2 message or
      another code 5 message where it tells to the client the name of the room
      and the name of the admin.

 - ALREADY_HERE (code 2)
      only the code is sent. It is used to tell to the client who is
      trying to connect to the server that there is already someone with his
      name.

 - MESSAGE_TO (code 3)
      the message is some text, it is used by the client to send messages

               +----+----+----+----+----+ - - - - - - - +
               |code|      length       |      text     |
               +----+----+----+----+----+ - - - - - - - +
                     MSB            LSB  <---length---->

 - MESSAGE_FROM (code 4)
      the message contains the name of the client and the text he sent:

          MSB            LSB  MSB            LSB  <--namelen-->
    +----+----+----+----+----+----+----+----+----+ - - - - - - + - - - - - +
    |0x04|      length       |     name_len      |    name     |   text    |
    +----+----+----+----+----+----+----+----+----+ - - - - - - + - - - - - +
                              <------------------length------------------->

      It is used by the server to distribute messages to the connected clients.
      `name_length` is the length of the name, name is the name of the client
      who sent the message and text is the message itself

 - WELCOME (code 5)
      This message is similar to MESSAGE_FROM but the name field is the name of
      the room which the client is connected to and text is the name of the
      admin of that room. This type of message is sent right after a client
      connects to a server
*/

use std::io::{self, ErrorKind, Read, Write};
use std::iter::FromIterator;
use std::net::TcpStream;

/// Divide a `u32` into 4 parts (one byte each, MSB first)
fn uint_to_bytes(val: u32) -> [u8; 4] {
    // (BIG ENDIAN)
    [
        (val >> 24) as u8,
        (val >> 16) as u8,
        (val >> 8) as u8,
        val as u8,
    ]
}
/// Make a `u32` from 4 bytes (MSB first)
fn bytes_to_uint(val: [u8; 4]) -> u32 {
    // (BIG ENDIAN)
    ((val[0] as u32) << 24) + ((val[1] as u32) << 16) + ((val[2] as u32) << 8) + val[3] as u32
}

/// Message `Code` used by the Chattest protocol
#[derive(PartialEq, Debug)]
pub enum Code {
    /// Name(name)
    Name(String),
    /// AlreadyHere
    AlreadyHere,
    /// MessageTo(text)
    MessageTo(String),
    /// MessageFrom(name, text)
    MessageFrom(String, String),
    /// Welcome(room, admin)
    Welcome(String, String),
}

const NAME: u8 = 1;
const ALREADY_HERE: u8 = 2;
const MESSAGE_TO: u8 = 3;
const MESSAGE_FROM: u8 = 4;
const WELCOME: u8 = 5;

/// A wrapper around the `TcpStream` that uses the Chattest protocol
pub struct BlockingStream {
    stream: TcpStream,
}

impl BlockingStream {
    pub fn new(stream: TcpStream) -> Self {
        stream.set_nonblocking(false).unwrap();
        BlockingStream { stream }
    }

    /// Reads 4 bytes from the stream and returns them as a single `u32`
    fn read_uint(&mut self) -> io::Result<u32> {
        let mut length = [0u8; 4];
        self.stream.read_exact(&mut length)?;
        Ok(bytes_to_uint(length))
    }
    /// Reads a byte from the stream and returns it
    fn read_byte(&mut self) -> io::Result<u8> {
        let mut code = [0u8; 1];
        self.stream.read_exact(&mut code)?;
        Ok(code[0])
    }
    /// Reads `size` bytes(u8) from the stream and returns a `String` made out of them
    fn read_chars(&mut self, size: usize) -> io::Result<String> {
        let mut message = vec![0u8; size];
        self.stream.read_exact(&mut message)?;
        Ok(String::from_iter(message.into_iter().map(|b| b as char)))
    }
    /// Sends a `u32` to through the stream
    fn write_uint(&mut self, val: u32) -> io::Result<()> {
        self.stream.write_all(&uint_to_bytes(val))?;
        Ok(())
    }
    /// Sends a byte to through the stream
    fn write_byte(&mut self, val: u8) -> io::Result<()> {
        self.stream.write_all(&[val])?;
        Ok(())
    }
    /// Sends a `String` through the stream
    fn write_chars(&mut self, val: String) -> io::Result<()> {
        self.stream.write_all(&val.as_bytes())?;
        Ok(())
    }
    /// Read from the stream a message and returns it
    pub fn read(&mut self) -> io::Result<Code> {
        // Read the code number
        let code = self.read_byte()?;
        match code {
            // Code Name(name) is code 1
            NAME => {
                let length = self.read_uint()? as usize;
                Ok(Code::Name(self.read_chars(length)?))
            }
            // AlreadyHere is code 2
            ALREADY_HERE => Ok(Code::AlreadyHere),
            // MessageTo(message) is code 3
            MESSAGE_TO => {
                let length = self.read_uint()? as usize;
                Ok(Code::MessageTo(self.read_chars(length)?))
            }
            // MessageFrom(name, message) is code 4
            MESSAGE_FROM => {
                // Get the length of the message
                let length = self.read_uint()? as usize;
                // Length must be at least 4 (size of the length of the name)
                if length >= 4 {
                    // Get the length of the message
                    let name_len = self.read_uint()? as usize;
                    // The size of the name can't exceed the one of the entire message
                    if name_len < length {
                        return Ok(Code::MessageFrom(
                            self.read_chars(name_len)?,              // Name
                            self.read_chars(length - name_len - 4)?, // Message
                        ));
                    }
                }
                Err(io::Error::new(
                    ErrorKind::InvalidInput,
                    "Chattest stream error: Length recived is not correct!",
                ))
            }
            WELCOME => {
                // Get the length of the message
                let length = self.read_uint()? as usize;
                // Length must be at least 4 (size of the length of the name)
                if length >= 4 {
                    // Get the length of the message
                    let room_len = self.read_uint()? as usize;
                    // The size of the name can't exceed the one of the entire message
                    if room_len < length {
                        return Ok(Code::Welcome(
                            self.read_chars(room_len)?,              // Room
                            self.read_chars(length - room_len - 4)?, // Admin
                        ));
                    }
                }
                Err(io::Error::new(
                    ErrorKind::InvalidInput,
                    "Chattest stream error: Length recived is not correct!",
                ))
            }
            // Other codes are not suppored
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Chattest stream error: code not supported!",
            )),
        }
    }

    /// Sends a `message` through the stream:
    pub fn write(&mut self, message: Code) -> io::Result<()> {
        match message {
            Code::Name(name) => {
                self.write_byte(NAME)?;
                self.write_uint(name.len() as u32)?;
                self.write_chars(name)?;
            }
            Code::AlreadyHere => {
                self.write_byte(ALREADY_HERE)?;
                self.write_uint(0)?;
            }
            Code::MessageTo(message) => {
                self.write_byte(MESSAGE_TO)?;
                self.write_uint(message.len() as u32)?;
                self.write_chars(message)?;
            }
            Code::MessageFrom(name, message) => {
                self.write_byte(MESSAGE_FROM)?;
                self.write_uint((name.len() + message.len()) as u32 + 4)?;
                self.write_uint(name.len() as u32)?;
                self.write_chars(name)?;
                self.write_chars(message)?;
            }
            Code::Welcome(room, admin) => {
                self.write_byte(WELCOME)?;
                self.write_uint((room.len() + admin.len()) as u32 + 4)?;
                self.write_uint(room.len() as u32)?;
                self.write_chars(room)?;
                self.write_chars(admin)?;
            }
        }
        self.stream.flush()?;
        Ok(())
    }

    /// Set the stream
    pub fn non_blocking(self) -> NonBlockingStream {
        NonBlockingStream::new(self.stream)
    }
}

/// A non blocking stream that only blocks on writing
pub struct NonBlockingStream {
    stream: TcpStream,
    buffer: Vec<u8>,
    bytes: usize,
    length: usize,
}

impl NonBlockingStream {
    pub fn new(stream: TcpStream) -> Self {
        stream.set_nonblocking(true).unwrap();
        NonBlockingStream {
            stream,
            buffer: vec![0],
            bytes: 0,
            length: 0,
        }
    }

    pub fn try_read(&mut self) -> io::Result<Option<Code>> {
        use std::iter;
        // Read from the stream some bytes and append them in the buffer
        match self.stream.read(&mut self.buffer[self.bytes..]) {
            Ok(bytes) => self.bytes += bytes,
            Err(error) => match error.kind() {
                ErrorKind::WouldBlock => return Ok(None),
                _ => return Err(error),
            },
        }
        println!("Buffer: {:?}", self.buffer);
        // The first byte represents the code
        if self.bytes == 1 {
            match self.buffer[0] {
                NAME => {
                    // Extend the buffer to accomodate the length of the message
                    self.buffer.extend(iter::repeat(0).take(4));
                }
                ALREADY_HERE => {
                    // Reset the values
                    self.bytes = 0;
                    self.buffer[0] = 0;
                    // Return the message code
                    return Ok(Some(Code::AlreadyHere));
                }
                MESSAGE_TO => {
                    // Extend the buffer to accomodate the length of the message
                    self.buffer.extend(iter::repeat(0).take(4));
                }
                MESSAGE_FROM => {
                    // Extend the buffer enough to store the length of the
                    // message
                    self.buffer.extend(iter::repeat(0).take(4));
                }
                WELCOME => {
                    // Extend the buffer enough to store the length of the
                    // message
                    self.buffer.extend(iter::repeat(0).take(4));
                }
                code => {
                    // Reset the values
                    self.bytes = 0;
                    self.buffer[0] = 0;
                    println!("Warning: recived {}!", code);
                }
            }
        // After another four bytes the length can be calculated
        } else if self.bytes == 5 {
            // Calculate the length
            self.length = bytes_to_uint([
                self.buffer[1],
                self.buffer[2],
                self.buffer[3],
                self.buffer[4],
            ]) as usize;
            // Extend the buffer to accomodate the rest of the message
            self.buffer.extend(iter::repeat(0).take(self.length));

        // After all the bytes have arrived the result can be returned
        } else if self.bytes == self.length + 5 {
            match self.buffer[0] {
                NAME => {
                    // Trasform the bytes into a string
                    let name = String::from_iter(self.buffer.iter().skip(5).map(|b| *b as char));
                    // Reset the values
                    self.bytes = 0;
                    self.buffer = vec![0];
                    // Return the name
                    return Ok(Some(Code::Name(name)));
                }
                MESSAGE_TO => {
                    // Trasform the bytes into a string
                    let text = String::from_iter(self.buffer.iter().skip(5).map(|b| *b as char));
                    // Reset the values
                    self.bytes = 0;
                    self.buffer = vec![0];
                    // Return the text
                    return Ok(Some(Code::MessageTo(text)));
                }
                MESSAGE_FROM => {
                    // Calculate the length of the name
                    let name_len = bytes_to_uint([
                        self.buffer[5],
                        self.buffer[6],
                        self.buffer[7],
                        self.buffer[8],
                    ]) as usize;
                    // Convert the bytes into a String and separate the name
                    // from the text
                    let name = String::from_iter(
                        self.buffer
                            .iter()
                            .skip(9)
                            .map(|b| *b as char)
                            .take(name_len),
                    );
                    let text = String::from_iter(
                        self.buffer.iter().skip(9 + name_len).map(|b| *b as char),
                    );
                    // Reset the values
                    self.bytes = 0;
                    self.buffer = vec![0];
                    // Return the name and text
                    return Ok(Some(Code::MessageFrom(name, text)));
                }
                WELCOME => {
                    // Calculate the length of the name
                    let room_len = bytes_to_uint([
                        self.buffer[5],
                        self.buffer[6],
                        self.buffer[7],
                        self.buffer[8],
                    ]) as usize;
                    // Convert the bytes into a String and separate the room's
                    // name from the admin's one
                    let room = String::from_iter(
                        self.buffer
                            .iter()
                            .skip(9)
                            .map(|b| *b as char)
                            .take(room_len),
                    );
                    let admin = String::from_iter(
                        self.buffer.iter().skip(9 + room_len).map(|b| *b as char),
                    );
                    // Reset the values
                    self.bytes = 0;
                    self.buffer = vec![0];
                    // Return the room and admin names
                    return Ok(Some(Code::Welcome(room, admin)));
                }
                code => println!("Chattest stream error: code {} not supported!", code),
            }
        }
        Ok(None)
    }

    /// Sends a `u32` to through the stream
    fn write_uint(&mut self, val: u32) -> io::Result<()> {
        self.stream.write_all(&uint_to_bytes(val))?;
        Ok(())
    }
    /// Sends a byte to through the stream
    fn write_byte(&mut self, val: u8) -> io::Result<()> {
        self.stream.write_all(&[val])?;
        Ok(())
    }
    /// Sends a `String` through the stream
    fn write_chars(&mut self, val: String) -> io::Result<()> {
        self.stream.write_all(&val.as_bytes())?;
        Ok(())
    }

    pub fn write(&mut self, message: Code) -> io::Result<()> {
        match message {
            Code::Name(name) => {
                self.write_byte(NAME)?;
                self.write_uint(name.len() as u32)?;
                self.write_chars(name)?;
            }
            Code::AlreadyHere => {
                self.write_byte(ALREADY_HERE)?;
                self.write_uint(0)?;
            }
            Code::MessageTo(message) => {
                self.write_byte(MESSAGE_TO)?;
                self.write_uint(message.len() as u32)?;
                self.write_chars(message)?;
            }
            Code::MessageFrom(name, message) => {
                self.write_byte(MESSAGE_FROM)?;
                self.write_uint((name.len() + message.len()) as u32 + 4)?;
                self.write_uint(name.len() as u32)?;
                self.write_chars(name)?;
                self.write_chars(message)?;
            }
            Code::Welcome(room, admin) => {
                self.write_byte(MESSAGE_FROM)?;
                self.write_uint((room.len() + admin.len()) as u32 + 4)?;
                self.write_uint(room.len() as u32)?;
                self.write_chars(room)?;
                self.write_chars(admin)?;
            }
        }
        self.stream.flush()?;
        Ok(())
    }

    pub fn blocking(self) -> BlockingStream {
        BlockingStream::new(self.stream)
    }
}
