use phpass::error::Error;
use phpass::PhPass;

pub fn cracker(hash: &str) -> Result<String, Error> {
   PhPass::try_from(hash)?.verify("partylikearockstar");

   Ok(String::from("partylikearockstar"))

}