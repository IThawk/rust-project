//! Simple HTTPS GET client based on hyper-rustls
//!
//! First parameter is the mandatory URL to GET.
//! Second parameter is an optional path to CA store.
// #![deny(warnings)]

extern crate futures;
extern crate hyper;
extern crate hyper_rustls;
extern crate rustls;
extern crate tokio;
extern crate reqwest;
extern crate tempdir;
use futures::{Future, Stream};
use self::hyper::{client, Uri};
use std::str::FromStr;
use std::{env, fs, io};
use std::result::Result::Err;
use std::string::String;
use std::option::Option::{Some,None};
use std::result::Result::Ok;
use std::result::Result;
use std::io::Write;
use tempdir::TempDir;

pub fn reqwest_demo1_main() {
    main();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
//    let tmp_dir = TempDir::new("example")?;
    let target = "https://codeload.github.com/ctz/hyper-rustls/zip/master";
//    let mut response = reqwest::get(target)?;
    let mut response = reqwest::Client::builder().danger_accept_invalid_hostnames(true).build()
        .unwrap().get(target).send()?;

    let mut dest = {
        let fname = response
            .url()
            .path_segments()
            .and_then(|segments| {
                println!("-------{:?}",segments);
                segments.last()})
            .and_then(|name| if name.is_empty() { None } else { Some(name) })
            .unwrap_or("tmp.bin");

//        println!("file to download: '{}'", fname);
        let fnames = format!("{}/{}","D:/",fname);
        println!("will be located under: '{:?}'", fnames);

        //这个地方就会报错(如果类型不对的话,就会报错)
//        fs::File::create("D:/a.zip")?

        fs::File::create(fnames)?
    };
    std::io::copy(&mut response, &mut dest)?;

    println!("下载完成了");
    Ok(())
}
