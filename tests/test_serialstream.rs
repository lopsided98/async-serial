use async_serial::SerialPortBuilderExt;
use futures::{AsyncReadExt, AsyncWriteExt};
use std::time::Duration;

#[cfg(unix)]
const DEFAULT_TEST_PORT_NAMES: &str = concat!(
    env!("CARGO_TARGET_TMPDIR"),
    "/ttyUSB0;",
    env!("CARGO_TARGET_TMPDIR"),
    "/ttyUSB1"
);
#[cfg(not(unix))]
const DEFAULT_TEST_PORT_NAMES: &str = "COM10;COM11";

struct Fixture {
    #[cfg(unix)]
    process: std::process::Child,
    pub port_a: &'static str,
    pub port_b: &'static str,
}

#[cfg(unix)]
impl Drop for Fixture {
    fn drop(&mut self) {
        self.process.kill().ok();
        std::thread::sleep(Duration::from_millis(250));
        std::fs::remove_file(self.port_a).ok();
        std::fs::remove_file(self.port_b).ok();
    }
}

impl Fixture {
    #[cfg(unix)]
    pub fn new(port_a: &'static str, port_b: &'static str) -> Self {
        let args = [
            format!("PTY,link={}", port_a),
            format!("PTY,link={}", port_b),
        ];

        let process = std::process::Command::new("socat")
            .args(&args)
            .spawn()
            .expect("unable to spawn socat process");

        std::thread::sleep(Duration::from_millis(500));

        Self {
            process,
            port_a,
            port_b,
        }
    }

    #[cfg(not(unix))]
    pub async fn new(port_a: &'static str, port_b: &'static str) -> Self {
        Self { port_a, port_b }
    }
}

fn setup_virtual_serial_ports() -> Fixture {
    let port_names: Vec<&str> = std::option_env!("TEST_PORT_NAMES")
        .unwrap_or(DEFAULT_TEST_PORT_NAMES)
        .split(';')
        .collect();

    assert_eq!(port_names.len(), 2);
    Fixture::new(port_names[0], port_names[1])
}

#[test]
fn send_recv() {
    let fixture = setup_virtual_serial_ports();
    futures::executor::block_on(async move {
        let mut sender = async_serial::new(fixture.port_a, 9600)
            .open_native_async()
            .expect("unable to open serial port");
        let mut receiver = async_serial::new(fixture.port_b, 9600)
            .open_native_async()
            .expect("unable to open serial port");

        let message = b"This is a test message";
        sender
            .write_all(message)
            .await
            .expect("unable to write test message");

        let mut buf = [0u8; 32];
        receiver
            .read_exact(&mut buf[..message.len()])
            .await
            .expect("unable to read test message");

        assert_eq!(&buf[..message.len()], message);
    })
}
