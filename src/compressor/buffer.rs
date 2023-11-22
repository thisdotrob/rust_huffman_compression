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
