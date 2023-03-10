#![warn(rust_2018_idioms)]

use futures::{io::BufReader, AsyncBufReadExt};
use std::{env, str};

use async_serial::SerialPortBuilderExt;

#[cfg(unix)]
const DEFAULT_TTY: &str = "/dev/ttyUSB0";
#[cfg(windows)]
const DEFAULT_TTY: &str = "COM1";

fn main() -> async_serial::Result<()> {
    futures::executor::block_on(async {
        let mut args = env::args();
        let tty_path = args.nth(1).unwrap_or_else(|| DEFAULT_TTY.into());

        let mut port = BufReader::new(async_serial::new(tty_path, 9600).open_native_async()?);

        loop {
            let mut line = String::new();
            port.read_line(&mut line).await?;
            println!("{}", line);
        }
    })
}
