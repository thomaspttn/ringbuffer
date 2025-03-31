// ringbuffer implementation in Rust

pub struct RingBuffer {
    buffer: Vec<u8>,
    head: usize,
    tail: usize,
    size: usize,
}

pub enum FlushResult {
    Ok(Vec<u8>),
    Err,
}

impl RingBuffer {
    pub fn new(size: usize) -> Self {
        RingBuffer {
            buffer: Vec::with_capacity(size),
            head: 0,
            tail: 0,
            size,
        }
    }

    pub fn push(&mut self, item: u8) {
        if self.buffer.len() < self.size {
            self.buffer.push(item);
        } else {
            self.buffer[self.head] = item;
        }
        self.head = (self.head + 1) % self.size;
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

    pub fn log_message(&mut self, message: &[u8]) {
        // Log message to the ring buffer
        for byte in message {
            self.push(*byte);
        }
    }

    // add in CRC-8 checksum
    fn crc8(&self, slice: &[u8]) -> u8 {
        let mut crc = 0;
        for byte in slice {
            crc ^= byte;
        }
        crc
    }

    pub fn log_message_with_crc(&mut self, message: &[u8]) {
        // Log message to the ring buffer
        for byte in message {
            self.push(*byte);
        }

        // add in CRC-8 checksum
        let crc = self.crc8(message);
        self.push(crc);

        // add in a terminator u8acter
        self.push(b'\0');
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
        let crc = self.crc8(&message);
        if crc == self.pop().unwrap_or(0) {
            FlushResult::Ok(message)
        } else {
            FlushResult::Err
        }
    }
}

fn main() {
    // goal: log messages to a ring buffer. every N ticks, flush the ring buffer and check the
    // CRC-8 checksum

    let mut ring_buffer = RingBuffer::new(256);

    // messages are vecs of chars as u8
    let messages = [
        vec![b'H', b'e', b'l', b'l', b'o'],
        vec![b'W', b'o', b'r', b'l', b'd'],
        vec![b'R', b'u', b's', b't'],
        vec![b'R', b'i', b'n', b'g', b'B', b'u', b'f', b'f', b'e', b'r'],
        vec![
            b'R', b'u', b's', b't', b'c', b'r', b'y', b'c', b'h', b'e', b'c', b'k',
        ],
    ];

    // iter = tick
    for i in 0..100 {
        // every 20 ticks, flip a bit in the message to be logged and log it
        let message = &messages[i % messages.len()];
        if i % 20 == 0 {
            let mut flipped_message = message.clone();
            flipped_message[0] ^= 1 << (i % 8);
            ring_buffer.log_message_with_crc(&flipped_message);
        } else if i % 3 == 0 {
            ring_buffer.log_message_with_crc(message);
        }

        // every 10 ticks, flush the ring buffer and check the CRC-8 checksum
        if i % 10 == 0 {
            match ring_buffer.flush_message_with_crc_check() {
                FlushResult::Ok(message) => {
                    println!("Flushed message: {:?}", message);
                }
                FlushResult::Err => {
                    println!("Flushed message: CRC-8 checksum error");
                }
            }
        }
    }
}
