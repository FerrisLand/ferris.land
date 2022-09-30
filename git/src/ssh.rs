use std::{sync::Arc, time::Duration, process::{Stdio, Command}, io::Read};

use rocket::futures;
use russh::{server::{Config, Session, self, Auth}, ChannelId, CryptoVec};
use russh_keys::key::KeyPair;

pub async fn launch() {
    let mut config = Config::default();
    config.keys.push(KeyPair::generate_ed25519().unwrap());
    config.connection_timeout = Some(std::time::Duration::from_secs(10));
    config.auth_rejection_time = Duration::from_secs(3);
    russh::server::run(Arc::new(config), &"0.0.0.0:2222".parse().unwrap(), Server).await.unwrap();
}

struct Server;

impl server::Server for Server {
    type Handler = Handler;

    fn new_client(&mut self, peer_addr: Option<std::net::SocketAddr>) -> Handler {
        eprintln!("NEW CLIENT: {peer_addr:?}");
        Handler
    }
}

struct Handler;

impl server::Handler for Handler {
    type Error = anyhow::Error;
    type FutureAuth = futures::future::Ready<Result<(Self, server::Auth), anyhow::Error>>;
    type FutureUnit = futures::future::Ready<Result<(Self, Session), anyhow::Error>>;
    type FutureBool = futures::future::Ready<Result<(Self, Session, bool), anyhow::Error>>;

    fn auth_none(self, user: &str) -> Self::FutureAuth {
        // git access is provided through the git user
        if user == "git" {
            return self.finished_auth(server::Auth::Accept);
        }

        self.finished_auth(Auth::Reject {
            proceed_with_methods: None,
        })
    }

    fn data(self, _: ChannelId, data: &[u8], session: Session) -> Self::FutureUnit {
        eprintln!("data received: {data:?}");
        self.finished(session)
    }

    fn extended_data(
        self,
        _: ChannelId,
        _: u32,
        data: &[u8],
        session: Session,
    ) -> Self::FutureUnit {
        eprintln!("data received: {data:?}");
        self.finished(session)
    }

    fn exec_request(self, channel: ChannelId, data: &[u8], mut session: Session) -> Self::FutureUnit {
        let data = std::str::from_utf8(data).unwrap();
        let cmd: Vec<_> = data.split(' ').collect();
        println!("received cmd: {cmd:?}");

        if cmd.len() == 2 {
            match cmd[0] {
                "git-upload-pack" => {
                    // FIXME: un-hardcode
                    println!("git-upload-pack received...");
                    let mut child = Command::new("git-upload-pack")
                        .arg("./repos/rust.git")
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .spawn()
                        .expect("cannot spawn git-upload-pack");
                    let stdout = child.stdout.take().unwrap();

                    let mut leading_zeros: u8 = 0;
                    let mut encountered_non_zero = false;
                    // FIXME: oh my god why are we encrypting every byte separately
                    for byte in stdout.bytes().map(|b| b.unwrap()) {
                        match byte {
                            b'\n' => {
                                leading_zeros = 0;
                                encountered_non_zero = false;
                                println!("newline")
                            },
                            b'0' if !encountered_non_zero => leading_zeros += 1,
                            _ => encountered_non_zero = true,
                        }

                        session.data(channel, CryptoVec::from_slice(&[byte]));

                        if leading_zeros == 4 {
                            break;
                        }
                    }

                    println!("packfile sent!");
                },
                cmd => {
                    println!("unknown ssh command: {cmd}")
                },
            }
        }

        self.finished(session)
    }

    #[allow(unused_mut)]
    fn finished_auth(mut self, auth: Auth) -> Self::FutureAuth {
        futures::future::ready(Ok((self, auth)))
    }

    fn finished_bool(self, b: bool, s: Session) -> Self::FutureBool {
        futures::future::ready(Ok((self, s, b)))
    }

    fn finished(self, s: Session) -> Self::FutureUnit {
        futures::future::ready(Ok((self, s)))
    }
}
