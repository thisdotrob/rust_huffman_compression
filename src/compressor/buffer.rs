pub struct CompressorBuffer {
    compressed_bits: u32,
    compressed_bit_count: u8,
}

impl CompressorBuffer {
    pub fn new() -> Self {
        Self {
            compressed_bits: 0,
            compressed_bit_count: 0,
        }
    }

    pub fn write_bits(&mut self, value: u32, bit_count: u8) {
        self.compressed_bits = self.compressed_bits << bit_count;
        self.compressed_bits = self.compressed_bits | value;
        self.compressed_bit_count = self.compressed_bit_count + bit_count;
    }

    pub fn read_byte(&mut self) -> Option<u8> {
        if self.compressed_bit_count < 8 {
            return None;
        }

        self.compressed_bit_count = self.compressed_bit_count - 8;

        let byte = self.compressed_bits >> self.compressed_bit_count;

        let mask = if self.compressed_bit_count > 0 {
            u32::MAX >> (32 - self.compressed_bit_count)
        } else {
            0
        };

        self.compressed_bits = self.compressed_bits & mask;

        Some(byte as u8) // what impact on performance does this casting have?
    }

    pub fn byte_boundary_offset(&self) -> u8 {
        self.compressed_bit_count % 8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_has_a_constructor_that_intialises_an_empty_buffer() {
        let mut buffer = CompressorBuffer::new();
        assert_eq!(buffer.read_byte(), None);
    }

    #[test]
    fn it_can_write_bits() {
        let mut buffer = CompressorBuffer::new();
        let value: u32 = 0xBBB;
        let bit_count = 12;
        buffer.write_bits(value, bit_count);
    }

    #[test]
    fn it_can_read_bytes() {
        let mut buffer = CompressorBuffer::new();
        let value = 0b1;
        let bit_count = 8;
        buffer.write_bits(value, bit_count);
        assert_eq!(buffer.read_byte(), Some(0b00000001))
    }

    #[test]
    fn it_consumes_a_byte_when_read_byte_is_called() {
        let mut buffer = CompressorBuffer::new();
        let value = 0b1;
        let bit_count = 8;
        buffer.write_bits(value, bit_count);
        buffer.read_byte();
        assert_eq!(buffer.read_byte(), None)
    }

    #[test]
    fn it_returns_none_when_trying_to_read_bytes_before_8_bits_have_been_written() {
        let mut buffer = CompressorBuffer::new();
        let value = 0b101010;
        let bit_count = 6;
        buffer.write_bits(value, bit_count);
        assert_eq!(buffer.read_byte(), None)
    }

    #[test]
    fn it_can_read_bytes_in_between_writing_bits() {
        let mut buffer = CompressorBuffer::new();
        let value = 0b101010;
        let bit_count = 6;
        buffer.write_bits(value, bit_count);
        assert_eq!(buffer.read_byte(), None);
        buffer.write_bits(value, bit_count);
        assert_eq!(buffer.read_byte(), Some(0b10101010));
        buffer.write_bits(value, bit_count);
        assert_eq!(buffer.read_byte(), Some(0b10101010))
    }

    #[test]
    fn it_can_read_multiple_bytes_in_a_row() {
        let mut buffer = CompressorBuffer::new();
        let value = 0b10101010;
        let bit_count = 8;
        buffer.write_bits(value, bit_count);
        buffer.write_bits(value, bit_count);
        buffer.write_bits(value, bit_count);
        assert_eq!(buffer.read_byte(), Some(0b10101010));
        assert_eq!(buffer.read_byte(), Some(0b10101010));
        assert_eq!(buffer.read_byte(), Some(0b10101010));
        assert_eq!(buffer.read_byte(), None);
    }

    #[test]
    fn it_returns_the_byte_boundary_offset() {
        let mut buffer = CompressorBuffer::new();
        let value = 0b1;
        let bit_count = 3;
        buffer.write_bits(value, bit_count);
        assert_eq!(buffer.byte_boundary_offset(), 3);
        buffer.write_bits(value, bit_count);
        assert_eq!(buffer.byte_boundary_offset(), 6);
        buffer.write_bits(value, bit_count);
        assert_eq!(buffer.byte_boundary_offset(), 1);
    }

    #[test]
    #[should_panic(expected = "attempt to shift left with overflow")]
    fn it_panics_when_attempting_to_write_a_single_32_bit_value() {
        let mut buffer = CompressorBuffer::new();
        let value = 0xFFFFFFFF;
        let bit_count = 32;
        buffer.write_bits(value, bit_count);
    }

    #[test]
    #[should_panic(expected = "attempt to shift right with overflow")]
    fn it_panics_on_read_byte_when_buffer_exceeds_32_bits() {
        let mut buffer = CompressorBuffer::new();
        let value = 0xFFFFFFF;
        let bit_count = 28;
        buffer.write_bits(value, bit_count);
        buffer.write_bits(value, bit_count);
        buffer.read_byte();
    }
}
