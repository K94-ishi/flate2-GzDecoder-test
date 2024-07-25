use std::fs::File;
use std::io::{stdout, BufRead, BufReader, BufWriter, Write};
use std::error::Error;
use flate2::read::MultiGzDecoder;


// gzipファイルのパスを受け取る。
// ファイルのオープンに失敗した時にはファイル名とエラー内容を表示。
fn open_reading_gzip(filename: &str) -> BufReader<MultiGzDecoder<File>> {
    let file = File::open(filename).unwrap_or_else(|err| {
        panic!("Cannnot open file '{}', Error: {}", filename, err);
    });
    let decoder = MultiGzDecoder::new(file);
    BufReader::new(decoder)
}

fn main() -> Result<(), Box<dyn Error>> {
    let filename = "test-multi.txt.gz";
    // 読み込むgzipファイルを開き、バッファリングして読み込むためのBufReaderを準備
    let reader = open_reading_gzip(filename);
    // バッファリングして標準出力に書き出すためのBufwriterを準備
    let out = stdout();
    let mut writer = BufWriter::new(out.lock());
    // ファイルを行ごとに読み出す。
    let mut counter_lines: u64 = 0;
    for line in reader.lines() {
        counter_lines += 1;
        // 行の取り出しに失敗した時にはファイル名、その行が何行目か、エラー内容を表示する。
        let line = line.unwrap_or_else(|err|{
            panic!("Cannnot reading the {}th line of {}, Error: {}", counter_lines, filename, err);
        });
        // 標準出力に書き出し
        writer.write_all((line + "\n").as_bytes())?;
    }
    writer.flush()?;
    Ok(())
}