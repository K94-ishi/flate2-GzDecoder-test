# Use MultiGzDecoder Instead of GzDecoder When Decoding Gzip Files with flate2 in Rust

[日本語ページ（Japanese Page）](https://qiita.com/K94-ishi/items/75741f7d4c5ab26ba923)

## 1. Introduction

This article suggests using [MultiGzDecoder](https://docs.rs/flate2/latest/flate2/read/struct.MultiGzDecoder.html#) instead of [GzDecoder](https://docs.rs/flate2/latest/flate2/read/struct.GzDecoder.html) for decoding and reading gzip files in Rust.

I'll share some code for opening a gzip file and a brief explanation of the gzip structure.

The Rust version used is 1.79.0.

---

In both the [intro of the flate2 documentation](https://docs.rs/flate2/latest/flate2/index.html#about-multi-member-gzip-files) and the pages for each struct, it is stated that:

- GzDecoder reads only the first member in the file
- MultiGzDecoder reads all members in the file

Embarrassingly, I didn't understand what a member was and failed when trying to decode a gzip file containing multiple members with GzDecoder. 

I've done some research on this for myself, so I've summarized it here.

## 2. Sample Code

Here's the code to open a gzip file, decode it while buffering, and write the text to the standard output.   
(This assumes reading and writing large files. Please let me know if there are any issues or areas for improvement.)

```rust
use std::fs::File;
use std::io::{stdout, BufRead, BufReader, BufWriter, Write};
use std::error::Error;
use flate2::read::MultiGzDecoder;

// Receives the path to the gzip file.
// If opening the file fails, it displays the filename and error.
fn open_reading_gzip(filename: &str) -> BufReader<MultiGzDecoder<File>> {
    let file = File::open(filename).unwrap_or_else(|err| {
        panic!("Cannot open file '{}', Error: {}", filename, err);
    });
    let decoder = MultiGzDecoder::new(file);
    BufReader::new(decoder)
}

fn main() -> Result<(), Box<dyn Error>> {
    // The path to the file being processed is hard-coded.
    let filename = "./test-multi.txt.gz";
    // Open the gzip file for reading, and prepare a BufReader.
    let reader = open_reading_gzip(filename);
    // Prepare a BufWriter for buffering and writing to the standard output.
    let out = stdout();
    let mut writer = BufWriter::new(out.lock());
    // Read the file line by line.
    let mut counter_lines: u64 = 0;
    for line in reader.lines() {
        counter_lines += 1;
        // If reading a line fails, display the filename, the line number, and the error.
        let line = line.unwrap_or_else(|err| {
            panic!("Cannot read the {}th line of {}, Error: {}", counter_lines, filename, err);
        });
        // Write to the standard output.
        writer.write_all((line + "\n").as_bytes())?;
    }
    writer.flush()?;
    Ok(())
}
```

## 3. About gzip

Here, I have summarized the parts of [RFC 1952](https://datatracker.ietf.org/doc/html/rfc1952) that describe gzip structure and checked them against sample data.

### 3-1. Creating Sample Files

Prepare sample files.

```zsh
echo -e "11 12\n21 22" > test1.txt 
echo -e "31 32\n41 42" > test2.txt
# Concatenate the two text files and then gzip compress them.
cat test{1,2}.txt | gzip -c > test-single.txt.gz 
# Gzip compress the two text files separately and then concatenate the gzip files.
gzip -k test{1,2}.txt
cat test{1,2}.txt.gz > test-multi.txt.gz
```

The prepared `test-multi.txt.gz` and `test-single.txt.gz` are different files, but their contents are the same when decompressed.

- Decompress and check the text

```zsh
% gzcat test-single.txt.gz  # use zcat for bash
11 12
21 22
31 32
41 42
% gzcat test-multi.txt.gz  # use zcat for bash
11 12
21 22
31 32
41 42
```

- Check as binary files displayed in hexadecimal

```zsh
% od -tx1 test-single.txt.gz
0000000    1f  8b  08  00  9d  79  9c  66  00  03  33  34  54  30  34  e2
0000020    32  32  54  30  32  e2  32  36  54  30  36  e2  32  31  54  30
0000040    31  e2  02  00  a3  93  dc  4a  18  00  00  00                
0000054
% od -tx1 test-multi.txt.gz
0000000    1f  8b  08  08  8e  5f  9b  66  00  03  74  65  73  74  31  2e
0000020    74  78  74  00  33  34  54  30  34  e2  32  32  54  30  32  e2
0000040    02  00  e8  e0  b9  57  0c  00  00  00  1f  8b  08  08  54  61
# Up to 0c  00  00  00 on the third line is the first member (the part of test1.txt.gz).
# From 1f  8b  08  08 on the third line is the second member (the part of test2.txt.gz).
0000060    9b  66  00  03  74  65  73  74  32  2e  74  78  74  00  33  36
0000100    54  30  36  e2  32  31  54  30  31  e2  02  00  5e  c9  a0  47
0000120    0c  00  00  00                                                
0000124
# Also check test1.txt.gz and test2.txt.gz
% od -tx1 test1.txt.gz      
0000000    1f  8b  08  08  8e  5f  9b  66  00  03  74  65  73  74  31  2e
0000020    74  78  74  00  33  34  54  30  34  e2  32  32  54  30  32  e2
0000040    02  00  e8  e0  b9  57  0c  00  00  00                        
0000052
% od -tx1 test2.txt.gz      
0000000    1f  8b  08  08  54  61  9b  66  00  03  74  65  73  74  32  2e
0000020    74  78  74  00  33  36  54  30  36  e2  32  31  54  30  31  e2
0000040    02  00  5e  c9  a0  47  0c  00  00  00                        
0000052
```

&rarr; Concatenating test1.txt.gz and test2.txt.gz indeed creates test-multi.txt.gz

### 3-2. Confirmation of the Description in [RFC 1952](https://datatracker.ietf.org/doc/html/rfc1952)

- Notation: One division represents a size of one byte.

``` text
	 +--------+
	 | 1 byte |
	 +--------+
```

- Structure of a gzip file (Comments starting with "# ..." are added for this article)

``` text
      A gzip file consists of a series of "members" (compressed data
      sets).  The format of each member is specified in the following
      section.  The members simply appear one after another in the file,
      with no additional information before, between, or after them.
```

``` text
      Each member has the following structure:

	# Header
         +---+---+---+---+---+---+---+---+---+---+
         |ID1|ID2|CM |FLG|     MTIME     |XFL|OS | (more-->)
         +---+---+---+---+---+---+---+---+---+---+
           
      (if FLG.FNAME set)

         +=========================================+
         |...original file name, zero-terminated...| (more-->)
         +=========================================+
         
    # If specified, various metadata continues,
    # but the header of the sample files we are checking is only as shown above.

	# Compressed data block
         +=======================+
         |...compressed blocks...| (more-->)
         +=======================+

	# Footer
           0   1   2   3   4   5   6   7
         +---+---+---+---+---+---+---+---+
         |     CRC32     |     ISIZE     |
         +---+---+---+---+---+---+---+---+
```

### 3-3. Cross-checking the Sample Files with the Description in [RFC 1952](https://datatracker.ietf.org/doc/html/rfc1952)

Checking the contents of `test1.txt.gz`.

``` zsh
% od -tx1 test1.txt.gz      
0000000    1f  8b  08  08  8e  5f  9b  66  00  03  74  65  73  74  31  2e
0000020    74  78  74  00  33  34  54  30  34  e2  32  32  54  30  32  e2
0000040    02  00  e8  e0  b9  57  0c  00  00  00                        
0000052
```

[Header]

- The first 2 bytes: 1f 8b
  "ID1" + "ID2"  
  ID1 = 1f, ID2 = 8b, indicating this is a gzip file.
- 3rd byte: 08
  "CM"
  Compression Method, representing the compression method. CM = 08 means "deflate".
- 4th byte: 08
  "FLG"  
  Uses 5 out of 8 bits to hold various information about the original file. FLG = 08 means "the flag indicating the original file name is ON".
  Also, `test-single.txt.gz` was compressed from standard input, so this flag is OFF, resulting in FLG = 00.
- 5th to 8th bytes: 8e 5f 9b 66
  "MTIME"  
  Modification TIME, the last modification time. For files compressed from standard input, this is the compression time.
- 9th byte: 00
  "XFL"  
  eXtra FLags, extended flags.
- 10th byte: 03
  "OS"  
  The type of file system where the compression was performed. OS = 03 means Unix.
- 11th to 20th bytes: 74 65 73 74 31 2e 74 78 74 00
  "original file name, zero-terminated"  
  The original file name before compression, ending with 00.
  Displaying "test.txt" in hexadecimal results in "74 65 73 74 31 2e 74 78 74".
  Confirmed using the [ASCII Code Converter｜Base Converter - Calculation Site](https://www.calc-site.com/bases/ascii).

[Footer]

- Last 4 bytes: 0c 00 00 00
  "ISIZE"
  Input SIZE, the size of the file before compression (the remainder when divided by 2^32).
  0x0000000c = 12
	- Check with the wc -c command
	``` zsh
	% wc -c test1.txt   
	      12 test1.txt
	```
- 5th to 8th bytes from the end: e8 e0 b9 57
  "CRC32"
  CRC32 checksum for detecting corruption or tampering.

## 4. Confirming the Operation of the Sample Code

The path of the gzip file to be expanded is hardcoded in the main() function.

``` rust
	# The first line of the main() function
    let filename = "./test-multi.txt.gz";
```

- Expand `./test-multi.txt.gz` using the sample code with MultiGzDecoder

``` zsh
% cargo run
   Compiling gzip_test v0.1.0 (/...path.../gzip_test)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.15s
     Running `target/debug/gzip_test`
11 12
21 22
31 32
41 42
```

&rarr; The entire file (both members) is decoded.

- Replace MultiGzDecoder with GzDecoder and rerun

``` zsh
# Using the GNU version of the sed command as gsed.
% gsed -i "s/MultiGzDecoder/GzDecoder/g" src/main.rs
% cargo run
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.01s
     Running `target/debug/gzip_test`
11 12
21 22 
```

&rarr; Only part of the file (the first member) is decoded.

- Change the target file from `./test-multi.txt.gz` to `./test-single.txt.gz` in the implementation using GzDecoder and rerun

``` zsh
% gsed -i "s/test-multi\.txt\.gz/test-single.txt.gz/g" src/main.rs
% cargo run
   Compiling gzip_test v0.1.0 (/Users/keiichi/Work/test/gzip_test)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.73s
     Running `target/debug/gzip_test`
11 12
21 22
31 32
41 42
```

&rarr; Since there is only one member, the entire file is decoded.

## 5. Summary

- gzip files can consist of one member or multiple members.
- GzDecoder in Rust's flate2 decodes only one member.
- When opening gzip files in Rust's flate2, use MultiGzDecoder.
- It's good to at least know what kind of files you are processing.

P.S.
This is my first time writing a summary article like this.
I would greatly appreciate any feedback.

## A. References

- Related to gzip  
[RFC 1952](https://datatracker.ietf.org/doc/html/rfc1952)  
[gzip（Wikipedia）](https://ja.wikipedia.org/wiki/Gzip)  
[Gzipについて調べてみた](https://qiita.com/takeru0911/items/903ac8b5d94660640af6)  
[TAR32.DLL フォーマット説明ファイル](http://openlab.ring.gr.jp/tsuneo/soft/tar32_2/tar32_2/sdk/TAR_FMT.TXT)  
[Go 言語と RFC から gzip の仕組みを紐解く](https://blog.8tak4.com/post/169064070956/principle-of-gzip-golang)  
[gzip圧縮されたデータの展開方法いろいろ](https://qiita.com/mpyw/items/eb6ef5e444c2250361b5)  

- Related to flate2  
[flate2（Official Documents）](https://docs.rs/flate2/1.0.30/flate2/)  
[Rustでfastq/fastq.gzを読み書きする](https://illumination-k.dev/techblog/post/198bf005-b169-4992-8ad5-667f594d84e0)  
[Rust-BioでGzip圧縮されたFASTAを読み込む](https://menseki.hatenablog.jp/entry/2021/09/08/000000)

- rustのI/O関係  
[What is the difference between write_all and flush in io::Write trait?](https://stackoverflow.com/questions/73620289/what-is-the-difference-between-write-all-and-flush-in-iowrite-trait)  
[Rustで高速な標準出力](https://keens.github.io/blog/2017/10/05/rustdekousokunahyoujunshutsuryoku/)  
[Rustファイル操作勉強スレ](https://zenn.dev/r0227n/scraps/2b9de10d44cd06)
