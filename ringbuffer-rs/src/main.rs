// ringbuffer implementation in Rust

pub struct RingBuffer {
    buffer: Vec<u8>,
    head: usize,
    tail: usize,
    size: usize,
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

    pub fn log_message(&mut self, message: Vec<u8>) {
        // Log message to the ring buffer
        for byte in message {
            self.push(byte);
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

    pub fn log_message_with_crc(&mut self, message: Vec<u8>) {
        // Log message to the ring buffer
        for byte in &message {
            self.push(*byte);
        }

        // add in CRC-8 checksum
        let crc = self.crc8(&message);
        self.push(crc);

        // add in a terminator u8acter
        self.push(b'\0');
    }

    pub fn flush_message_with_crc_check(&mut self) -> Result<Vec<u8>, ()> {
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
            Ok(message)
        } else {
            Err(())
        }
    }
}

fn main() {
    // create a ringbuffer
    let mut ringbuffer = RingBuffer::new(10);

    // push some u8 data to the ringbuffer
    ringbuffer.push(b'a');
    ringbuffer.push(b'b');
    ringbuffer.push(b'c');

    // pop some data from the ringbuffer
    println!("{:?}", ringbuffer.pop()); // Some(1)

    // check if the ringbuffer is empty
    println!("{:?}", ringbuffer.is_empty()); // false

    // check if the ringbuffer is full
    println!("{:?}", ringbuffer.is_full()); // false
}
