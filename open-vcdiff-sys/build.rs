extern crate gcc;
extern crate tempdir;

use std::fs;
use std::process;

fn main() {
    gcc::compile_library("libzlib-adler32.a", &["open-vcdiff/src/zlib/adler32.c"]);

    gcc::Config::new()
        .include("open-vcdiff/src")
        .file("src/glue.cc")
        .compile("libopen-vcdiff-glue.a");

    let mut config = gcc::Config::new();

    config.cpp(true);
    config.include("open-vcdiff/src");
    config.include("src");
    config.include("src/zlib");
    config.flag("-Wno-deprecated-declarations");

    if include_exists("ext/rope") {
        config.define("HAVE_EXT_ROPE", Some("1"));
    }

    if include_exists("malloc.h") {
        config.define("HAVE_MALLOC_H", Some("1"));
    }

    if include_exists("sys/mman.h") {
        config.define("HAVE_SYS_MMAN_H", Some("1"));
    }

    if include_exists("sys/stat.h") {
        config.define("HAVE_SYS_STAT_H", Some("1"));
    }

    if include_exists("sys/time.h") {
        config.define("HAVE_SYS_TIME_H", Some("1"));
    }

    if include_exists("unistd.h") {
        config.define("HAVE_UNISTD_H", Some("1"));
    }

    if include_exists("windows.h") {
        config.define("HAVE_WINDOWS_H", Some("1"));
    }

    config.file("open-vcdiff/src/addrcache.cc");
    config.file("open-vcdiff/src/blockhash.cc");
    config.file("open-vcdiff/src/codetable.cc");
    config.file("open-vcdiff/src/decodetable.cc");
    config.file("open-vcdiff/src/encodetable.cc");
    config.file("open-vcdiff/src/headerparser.cc");
    config.file("open-vcdiff/src/instruction_map.cc");
    config.file("open-vcdiff/src/jsonwriter.cc");
    config.file("open-vcdiff/src/logging.cc");
    config.file("open-vcdiff/src/varint_bigendian.cc");
    config.file("open-vcdiff/src/vcdecoder.cc");
    config.file("open-vcdiff/src/vcdiffengine.cc");
    config.file("open-vcdiff/src/vcencoder.cc");

    config.compile("libopen-vcdiff.a");
}

fn include_exists(path: &str) -> bool {
    use std::io::Write;

    let tool = gcc::Config::new().cpp(true).get_compiler();
    let dir = tempdir::TempDir::new("open-vcdiff-sys").unwrap();

    let header_path = dir.path().join("test.h");

    {
        let mut header_file = fs::File::create(&header_path).unwrap();

        writeln!(header_file, "#include <{}>", path).unwrap();
    }

    let result = tool.to_command().arg("-E").arg(header_path)
        .stderr(process::Stdio::null())
        .stdout(process::Stdio::null())
        .status().unwrap();

    result.success()
}
