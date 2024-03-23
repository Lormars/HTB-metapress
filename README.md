# HTB-metapress
An almost one-click solution to the HTB metapress machine built in rust.

## Usage
Just clone the repository, build it with cargo, and run. (Make sure you already connected to HTB and are able to access the machine.)
The script should then run succesfully, take you all the way to the last step before getting the root flag.

Due to my limited familiarity to russh, I cannot get the root flag through rust because of the password chanllenge when I attempt to 
`su -`, so I would really appreciate it if anyone knows how and share with me.

## Credit
The solution is built largely on the following two blogs:
https://0xdf.gitlab.io/2023/04/29/htb-metatwo.html
https://medium.com/@KonradDaWo/hackthebox-metatwo-writeup-59135896c890

If you do not understand any step, you can just check these blogs.

Happy Hacking!
