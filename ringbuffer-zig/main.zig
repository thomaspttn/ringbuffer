const std = @import("std");

const RingBuffer = struct {
    buffer: []u8,
    head: usize,
    tail: usize,
    size: usize,

    pub fn init(allocator: *std.mem.Allocator, size: usize) !RingBuffer {
        const buf = try allocator.alloc(u8, size);
        return RingBuffer{
            .buffer = buf,
            .head = 0,
            .tail = 0,
            .size = size,
        };
    }

    pub fn deinit(self: *RingBuffer, allocator: *std.mem.Allocator) void {
        allocator.free(self.buffer);
    }

    pub fn is_full(self: *RingBuffer) bool {
        return (self.head + 1) % self.size == self.tail;
    }

    pub fn is_empty(self: *RingBuffer) bool {
        return self.head == self.tail;
    }

    pub fn push(self: *RingBuffer, byte: u8) !void {
        if (self.is_full()) return error.BufferFull;
        self.buffer[self.head] = byte;
        self.head = (self.head + 1) % self.size;
    }

    pub fn pop(self: *RingBuffer) !u8 {
        if (self.is_empty()) return error.BufferEmpty;
        const byte = self.buffer[self.tail];
        self.tail = (self.tail + 1) % self.size;
        return byte;
    }

    pub fn crc8(data: []const u8) u8 {
        var crc: u8 = 0;
        for (data) |b| {
            crc ^= b;
        }
        return crc;
    }

    pub fn pushSlice(self: *RingBuffer, slice: []const u8) !void {
        for (slice) |b| {
            try self.push(b);
        }
    }

    const error = error{
        BufferFull,
        BufferEmpty,
    };



};

pub fn main() !void {
    const allocator = std.heap.page_allocator;
    var rb = try RingBuffer.init(allocator, 32);
    defer rb.deinit(allocator);

    const msg = "boot ok";
    try rb.pushSlice(msg);
    const crc = RingBuffer.crc8(msg);
    try rb.push(crc);
    try rb.push('\n');

    // simulate flush: read all until \n, then check CRC
    var collected = [_]u8{0} ** 32;
    var i: usize = 0;
    while (true) {
        const byte = try rb.pop();
        if (byte == '\n') break;
        collected[i] = byte;
        i += 1;
    }

    const crc_actual = collected[i - 1];
    const msg_slice = collected[0..i - 1];
    const crc_calc = RingBuffer.crc8(msg_slice);

    if (crc_actual == crc_calc) {
        std.debug.print("[OK  ] {}\n", .{msg_slice});
    } else {
        std.debug.print("[FAIL] {}\n", .{msg_slice});
    }
}
