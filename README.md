# Huffman Compression

A huffman coding compression library in Rust.

## What's this all about then?

I need this for my Rust implementation of an Ultima Online game server and thought it would be fun to write it as a separate crate.

Huffman coding is a method for lossless data compression. Variable length codes are assigned to each type of input, with shorter codes assigned to more frequent inputs and longer ones to less frequent inputs. The overall length of the encoded data should therefore be shorter.

Ultima Online uses Huffman coding to compress the bytes in the network packets between the game server and game clients. For each of the 256 possible bytes a packet can include, a value with a unique prefix is assigned. These values are looked up for each byte in the original packet and written in their place to construct the compressed packet. The receiving code can decompress the packet because the prefixes are unique and so walking left to right through the packet data it can determine what the original bytes were. For example, here's how the 4 most common bytes in Ultima Online packets are encoded:

| original byte | original bits | compressed value |
|---------------|---------------|------------------|
|      0x00     |    00000000   |       00         |
|      0x01     |    00000001   |       11111      |
|      0x40     |    01000000   |       01010      |
|      0x03     |    00000011   |       100010     |

An uncompressed packet containing `[0x01, 0x03, 0x00, 0x40, 0x03]` would therefore be compressed to the following stream of bits (underscores to distinguish each compressed value):

```
11111_100010_00_01010_100010
```

Total length of the packet has been reduced from 5 x 8 = 40 bits to 24 bits.

Rearranging into bytes gives the compressed packet:

```
[11111100, 01000010, 10100010]
```

or in hex:

```
[0xFC, 0x42, 0xA2]
```

To decompress, the receiving code needs to traverse a pre constructed tree built according to the same encodings as the data was compressed with:
```
          start
       _____|_____
      /           \
     0             1
    / \           / \
   /   \         0   1
0x00    1       /     \
       /       0       1
      0       /         \
       \     0           1
        1     \           \
       /       1          0x01
      /       /
   0x40    0x03
```

Starting from the top, the code should take each of the compressed bits one by one and go left (0) or right (1) depending on its value. In the compressed packet from the example above the first bit is 1, so the code should go *right*. The next 4 bits are also 1s, so the code would continue branching right until it reached the leaf node of `0x01`, the original uncompressed byte that corresponds to these first five 1 bits. It would then write this decompressed byte and carry on to the next bit, starting from the top of the tree again.

## How to use this crate

### Getting started

First construct a `HuffmanTable` which represents the encoding rules:

```rust
let values: [u32; 256] = [
    0b1111, 0b0111, 0b1011, 0b110,
    // snip
];

let bit_counts: [u8; 256] = [
    4, 4, 4, 4,
    // snip
];

let table = HuffmanTable { values, bit_counts };
```

`values` are the compressed bits that should be written for corresponding uncompressed bytes. `bit_counts` are the number of bits that should be written for each encoded value. The correct value and bit_count for each uncompressed byte are looked up by using that byte as an index to look up the elements in the two arrays.

For example, given an uncompressed byte of 0x03:
  - The value, `0b110`, lies at index 3 (i.e. `0x03`) of `values`
  - The bit count, `4`,  lies at index 3 (again, `0x03`) of `bit_counts`
  - The final compressed bits are the value after is has been left padded with 0s until the bit_count is reached. In this case that means `0110` is written.

The example above shows only the first 4 elements for each array but in reality you will need to populate all 256.

Next create a `Huffman`, passing it the table:

```rust
let mut huffman = Huffman::new(table, None); // <-- the None is for the termination code, see further down
```

Now, send bytes to be compressed and an output vec for the compressed bits to be pushed to:

```rust
let uncompressed_bytes = vec![0x00, 0x01, 0x02, 0x03];

let mut output = Vec::new();

huffman.compress(uncompressed_bytes, &mut output);
```

Now we can see that `output` has been populated with the compressed bits, separated into bytes: 

```rust
// as binary with underscores separating the original compressed bit groupings:
assert_eq!(output, vec![0b1111_0111, 0b1011_0110]);

// or as hex:
assert_eq!(output, vec![0xF7, 0xB6]);
```

### Byte boundaries and termination codes

If the compressed bits do not align with a byte boundary like they do in the example above, the crate will pad with zeroes:

```rust
// using the same table as in the previous example

let uncompressed_bytes = [0x00, 0x01, 0x02];

huffman.compress(uncompressed_bytes, &mut output);

// the compressed bits will now be 0b1111_0111_1011. This is only one and a half bytes, so
// four zeroes are added to the end to make up to the next byte boundary:
assert_eq!(output, vec![0b1111_0111, 0b1011_0000]);
```

This is fine so long as the zeroes added for padding do not clash with one of the compressed values for a byte. For example if a 0x04 byte had a compressed value of `0b00` and bit count of 2, the last four bits in the example above would decompress to two 0x04 bytes which is not what we want.

To avoid this, it is possible to append a termination code to the compressed bits which will signal to the consuming code that any bits following do not represent compressed bytes and should be ignored:

```rust
// this time, set up the Huffman with a TerminalCode:

let terminal_code = TerminalCode { value: 0b001, bit_count: 3 };
let huffman = Huffman::new(table, Some(terminal_code));

// compress as normal:

let uncompressed_bytes = [0x00, 0x01, 0x02];
huffman.compress(uncompressed_bytes, &mut output);

// now the termination code is appended to the output before padding with zeroes:

assert_eq!(output, vec![0b1111_0111, 0b1011_001_0]);
//                                           ^
//                                  termination code
```

## To do
- Refactor tests
- Add tests for extracted modules
- Check performance and tweak
- Add docs
- Provide a streaming interface?
- Validate the values & bit counts given to HuffmanTable
- Validate the termination code is not present in the table
- Prevent panics when buffer exceeds 32 bits
