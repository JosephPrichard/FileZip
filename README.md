# Zipper
Simple lossless file compression and archival format implemented in Rust. Zipper utilizes huffman coding to compress files, generally achieving a 55-65% compression ratio for the average text file. The compression works on any file type, but it works on text file types (txt, json, xml) the best. The archival format is inspired by TAR but is custom designed for simplicity. Project involved implementing the huffman coding algorithm, memory safe binary trees, and bit-layered reader/writers. Zipper contains no external depndencies and was primarily created to learn rust standard library.

## Compression Format
Each compressed file is broken into two segments: the tree segment and the compressed data segment. The tree segment is laid out using depth first traversal. An internal node is represented with a 0 bit, and a leaf node with a 1 bit. A leaf node is followed by the byte the bit code decompresses into. The compressed data segment simply contains a bit sequence of each original byte compressed using the aforementioned tree.

## Archival Format
The archive file is broken up into two segments: the file header segment and the file data segment. The file header segment contains a block for each file in the archive. Each block contains a null-terminated relative path, the bit sizes of the tree and compressed data, the pre compression byte size, and the file offset which acts as a pointer to the actual compressed data stored in the file data segment. The file data segment contains each compressed file stored as a bit stream. The archive two segments are separated by control code GS, and each file header is separated by control code RS.

## Usage

### Compress
Compresses each file into an archive using the compression strategy described above. Recursively adds sub-directories to archive.

../path/to/zipper.exe -c ../path/to/directory ../path/to/file.txt

### Decompress
Decompresses the archive into the stored directory strcture using the decompression strategy desribed above.

../path/to/zipper.exe -d ../path/to/archive.zipr

### List
Lists the sizes, compression ratios, and relative file name of any files in the archive. 

../path/to/zipper.exe -l ../path/to/archive.zipr

## Example
![image](https://user-images.githubusercontent.com/58538077/216788857-6a19c6a2-5770-4857-8392-3fd70f22594b.png)

