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

## To do
- Refactor tests
- Add tests for extracted modules
- Check performance and tweak
- Add docs
- Provide a streaming interface?
- Validate the values & bit counts given to HuffmanTable
- Prevent panics when buffer exceeds 32 bits
