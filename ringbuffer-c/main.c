#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct ringbuffer {
    char *buffer;
    int head;
    int tail;
    int size;
} ringbuffer_t;

uint8_t crc8(const char *data, size_t len) {
    uint8_t crc = 0x00;
    for (size_t i = 0; i < len; i++) {
        crc ^= (uint8_t)data[i];
        for (int j = 0; j < 8; j++) {
            if (crc & 0x80)
                crc = (crc << 1) ^ 0x07; // 0x07 is the CRC-8 poly
            else
                crc <<= 1;
        }
    }
    return crc;
}

// initialize ring buffer
void rb_init(ringbuffer_t *rb, int size) {
    rb->buffer = (char *)malloc(size * sizeof(char));
    rb->head = 0;
    rb->tail = 0;
    rb->size = size; 
}

// free buffer
void rb_free(ringbuffer_t *rb) {
    free(rb->buffer);
}

// check if buffer is full
int rb_is_full(ringbuffer_t *rb) {
    return (rb->head + 1) % rb->size == rb->tail;
}

// check if buffer is empty
int rb_is_empty(ringbuffer_t *rb) {
    return rb->head == rb->tail;
}

// push single char to buffer
int rb_push(ringbuffer_t *rb, char data) {
    if (rb_is_full(rb)) return 0; // fail if full
    rb->buffer[rb->head] = data;
    rb->head = (rb->head + 1) % rb->size;
    return 1;
}

// pop single char from buffer
int rb_pop(ringbuffer_t *rb, char *data) {
    if (rb_is_empty(rb)) return 0; // fail if empty
    *data = rb->buffer[rb->tail];
    rb->tail = (rb->tail + 1) % rb->size;
    return 1;
}

// log a full message into buffer
void rb_log_message(ringbuffer_t *rb, const char *msg) {
    for (size_t i = 0; i < strlen(msg); i++) {
        if (!rb_push(rb, msg[i])) break; // stop if full
    }
}

// log a message with CRC
void rb_log_message_crc(ringbuffer_t *rb, const char *msg) {
    uint8_t crc = crc8(msg, strlen(msg));
    for (size_t i = 0; i < strlen(msg); i++) {
        rb_push(rb, msg[i]);
    }
    rb_push(rb, crc); // append CRC at end
}


// flush buffer to stdout (simulate UART flush)
void rb_flush_to_stdout(ringbuffer_t *rb) {
    char c;
    while (rb_pop(rb, &c)) {
        putchar(c);
    }
}

int main() {
    ringbuffer_t rb;
    rb_init(&rb, 64);

    rb_log_message(&rb, "System initialized...\n");
    rb_log_message(&rb, "Sensor failed at T=123ms\n");

    printf("Flushing buffer:\n");
    rb_flush_to_stdout(&rb);

    rb_free(&rb);
    return 0;
}
