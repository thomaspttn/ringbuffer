// ringbuffer implementation in Rust

pub struct RingBuffer {
    buffer: Vec<u8>,
    head: usize,
    tail: usize,
    size: usize,
    max_flush_size: usize,
}

pub enum PushResult {
    Ok,
    Err(String),
}

pub enum FlushResult {
    Ok(Vec<u8>),
    Err(String),
}

impl RingBuffer {
    pub fn new(size: usize) -> Self {
        RingBuffer {
            buffer: vec![0; size],
            head: 0,
            tail: 0,
            size,
            max_flush_size: 32,
        }
    }

    pub fn push(&mut self, item: u8) -> PushResult {
        if self.is_full() {
            return PushResult::Err("Buffer is full".to_string());
        }

        self.buffer[self.head] = item;
        self.head = (self.head + 1) % self.size;
        PushResult::Ok
    }

    pub fn pop(&mut self) -> Option<u8> {
        if self.head == self.tail {
            None
        } else {
            let item = self.buffer[self.tail];
            self.tail = (self.tail + 1) % self.size;
            Some(item)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head == self.tail
    }

    pub fn is_full(&self) -> bool {
        (self.head + 1) % self.size == self.tail
    }

    // add in CRC-8 checksum
    fn crc8(&self, slice: &[u8]) -> u8 {
        let mut crc = 0;
        for byte in slice {
            crc ^= byte;
        }
        crc
    }

    // get the size of the next message in the buffer
    fn get_next_message_size(&self) -> Option<usize> {
        for (size, i) in (0..self.size).enumerate() {
            if self.buffer[(self.tail + i) % self.size] == b'\0' {
                return Some(size);
            }
        }
        None
    }

    pub fn log_message_with_crc(&mut self, message: &[u8]) -> PushResult {
        // Log message to the ring buffer
        for &byte in message {
            match self.push(byte) {
                PushResult::Ok => {}
                PushResult::Err(_) => return PushResult::Err("Error logging message".to_string()),
            }
        }

        // add in CRC-8 checksum
        let crc = self.crc8(message);
        match self.push(crc) {
            PushResult::Ok => {}
            PushResult::Err(_) => return PushResult::Err("Error logging message".to_string()),
        }

        // add in a terminator u8acter
        match self.push(b'\0') {
            PushResult::Ok => PushResult::Ok,
            PushResult::Err(_) => PushResult::Err("Error logging message".to_string()),
        }
    }

    pub fn flush_message_with_crc_check(&mut self) -> FlushResult {
        let mut message = Vec::new();
        while let Some(byte) = self.pop() {
            if byte == b'\0' {
                break;
            }
            message.push(byte);
        }
        // check CRC-8 checksum
        let crc_read = message.pop().unwrap_or(0) as char;
        let crc_calc = self.crc8(&message) as char;
        if crc_calc == crc_read {
            FlushResult::Ok(message)
        } else {
            FlushResult::Err("CRC-8 checksum failed".to_string())
        }
    }

    pub fn dma_flush_with_crc_check(&mut self) -> FlushResult {
        let mut bytes_sent = 0;
        let mut message = Vec::new();

        // goal: pop COMPLETE messages until we're out of messages or we've sent max_flush_size
        // bytes. don't forget the CRC check

        while bytes_sent < self.max_flush_size && !self.is_empty() {
            println!("message: {:?}", message);
            // get the size of the next message in the buffer
            let message_size = match self.get_next_message_size() {
                Some(size) => size,
                None => break,
            };

            // pop the message and CRC-8 checksum
            for _ in 0..message_size {
                if let Some(byte) = self.pop() {
                    message.push(byte);
                } else {
                    return FlushResult::Err("Error flushing message".to_string());
                }
            }

            // check CRC-8 checksum
            let crc_read = message.pop().unwrap_or(0) as char;
            let crc_calc = self.crc8(&message) as char;
            if crc_calc != crc_read {
                return FlushResult::Err("CRC-8 checksum failed".to_string());
            }
            bytes_sent += message_size;
        }
        FlushResult::Ok(message)
    }
}

// split into characters and convert to u8, return should be 2d u8 vec
pub fn create_log_messages(messages: &[&str]) -> Vec<Vec<u8>> {
    messages
        .iter()
        .map(|message| message.chars().map(|c| c as u8).collect())
        .collect()
}

fn main() {
    // goal: log messages to a ring buffer. every N ticks, flush the ring buffer and check the
    // CRC-8 checksum
    let mut ring_buffer = RingBuffer::new(256);

    let messages = create_log_messages(&[
        "Hello, world!",
        "This is a test message.",
        "This is a longer test message.",
        "This is a very long test message that is longer than the others.",
    ]);

    // iter = tick
    for i in 0..100 {
        // every 20 ticks, flip a bit in the message to be logged and log it
        let message = &messages[i % messages.len()];
        //if i % 20 == 0 {
        //    let mut flipped_message = message.clone();
        //    flipped_message[0] ^= 1 << (i % 8);
        //    ring_buffer.log_message_with_crc(&flipped_message);
        if i % 3 == 0 {
            ring_buffer.log_message_with_crc(message);
        }

        // every 10 ticks, flush the ring buffer and check the CRC-8 checksum
        if i % 10 == 0 {
            match ring_buffer.dma_flush_with_crc_check() {
                FlushResult::Ok(message) => {
                    println!(
                        "Flushed message: {:?}",
                        message.iter().map(|&b| b as char).collect::<String>()
                    );
                }
                FlushResult::Err(e) => {
                    println!("Error flushing message: {}", e);
                }
            }
        }
    }
}
