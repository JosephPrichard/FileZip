# Zipper
Simple lossless file compression and archival format implemented in Rust. It utilizes huffman coding to compress files, generally achieving a 55-65% compression ratio for the average text file. The compression works on any file type, but it works on text file types (txt, json, xml) the best. The archival format is inspired by TAR but is custom designed for simplicity. Project involved implementing the huffman coding algorithm, memory safe binary trees, and bit-layered reader/writers. The project contains only rayon as a depndency and was primarily created to learn rust standard library.

## Compression Format
Each compressed file is broken into two segments: the tree segment and the compressed data segment. The tree segment is laid out using depth first traversal. An internal node is represented with a 0 bit, and a leaf node with a 1 bit. A leaf node is followed by the byte the bit code decompresses into. The compressed data segment simply contains a bit sequence of each original byte compressed using the aforementioned tree.

## Archival Format
The archive file is broken up into two segments: the file header segment and the file data segment. The file header segment contains a block for each file in the archive. Each block contains a null-terminated relative path, the bit sizes of the tree and compressed data, the pre compression byte size, and the file offset which acts as a pointer to the actual compressed data stored in the file data segment. The file data segment contains each compressed file stored as a bit stream. The archive two segments are separated by control code GS, and each file header is separated by control code RS.

## Usage

### Compress
Compresses each file into an archive using the compression strategy described above. Recursively adds sub-directories to archive.

```../path/to/zipper.exe -c ../path/to/directory ../path/to/file.txt```

### Decompress
Decompresses the archive into the stored directory strcture using the decompression strategy desribed above.

```../path/to/zipper.exe -d ../path/to/archive.zipr```

### List
Lists the sizes, compression ratios, and relative file name of any files in the archive. 

```../path/to/zipper.exe -l ../path/to/archive.zipr```

### Multihreading
Adds multithreaded parallelism using Rayon's threadpool. Uses the max number of cores as the thread pool count, although this isn't guaranteed to actually use all cores in the system (unless you have no other processes being executed).

```../path/to/zipper.exe -c -mt ../path/to/directory```

## Example
![image](https://github.com/JosephPrichard/Zipper/assets/58538077/6d8365f2-a7a8-45b1-96d3-632350a65184)


