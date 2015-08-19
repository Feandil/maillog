use std::ops::Deref;
use super::super::ParseError;
use super::Inner;
use super::Message;
use super::MessageParser;

#[derive(Debug)]
pub struct Cleanup {
	inner: Inner,
	message_id_s: usize,
	message_id_e: usize,
	resent: bool,
}

impl Deref for Cleanup {
	type Target = Inner;
	fn deref(&self) -> &Inner {
		&self.inner
	}
}

impl Cleanup {
	pub fn message_id <'a>(&'a self) -> &'a str {
		&self.raw[self.message_id_s..self.message_id_e]
	}

}
impl MessageParser for Cleanup {
	fn parse(inner: Inner, start: usize) -> Result<Option<Message>, ParseError> {
		{
			match inner.queue_id() {
				Some(_) => (),
				None => {
					let rest = &inner.raw[start..];
					if rest.starts_with(" warning:") {
						return Ok(None)
					}
				}
			};

		}
		let (message_id_s, message_id_e, resent) = {
			let rest = &inner.raw[start..];
			let (rest, message_id_s, resent) = {
				if rest.starts_with(" message-id=") {
					(&rest[12..], start + 12, false)
				} else if rest.starts_with(" resent-message-id=") {
					(&rest[19..], start + 19, true)
				} else {
					return Err(ParseError::CleanupNoMessageID);
				}
			};
			let mut message_id_s = message_id_s;
			let mut message_id_e = rest.len();
			if &rest[message_id_e-1..message_id_e] == "\n" {
				message_id_e = message_id_e - 1;
			}
			if &rest[message_id_e-1..message_id_e] == ">" {
				message_id_e = message_id_e - 1;
			}
			message_id_e = message_id_s + message_id_e;
			if message_id_e > message_id_s {
				if &inner.raw[message_id_s..message_id_s+1] == "<" {
					message_id_s = message_id_s + 1;
				}
			}
			(message_id_s, message_id_e, resent)
		};
		Ok(Some(Message::Cleanup { m: Cleanup { inner: inner, message_id_s: message_id_s, message_id_e: message_id_e, resent: resent } }))
	}
}


#[cfg(test)]
mod tests {
	use std::fmt;
	use super::*;
	use super::super::Inner;
	use super::super::Message;
	use super::super::MessageParser;
	use super::super::super::ParserConfig;
	use super::super::super::ParseError;

	fn parse_cleanup(s: String) -> Result<Option<Message>, ParseError> {
		let conf = ParserConfig { process_noise: vec!["clamsmtpd".to_string()] };
		let (inner, start) = match Inner::parse(&conf, s) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some((x,y))) => (x,y)
		};
		Cleanup::parse(inner, start)
	}

	#[test]
	fn ignored() {
		let s = "Aug  4 04:28:18 ozgurluk postfix-in/cleanup[24617]: warning: bounce: removed spurious C8A031E05FB log".to_string();
		match parse_cleanup(s) {
			Err(x) => panic!("Failed to parse {}", x),
			Ok(None) => (),
			Ok(_) => panic!("This should have been ignored"),
		};
	}

	#[test]
	fn no_orig_client() {
		let s ="Aug  4 00:00:01 yuuai postfix-in/cleanup[22502]: A071220883: message-id".to_string();
		match parse_cleanup(s) {
			Err(ParseError::CleanupNoMessageID) => (),
			Err(x) => panic!("Wrong error, should have been CleanupNoMessageID {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(_) => panic!("This should have failed")
		};
	}

	#[test]
	fn valid() {
		let s = "Aug  4 00:00:01 yuuai postfix-in/cleanup[22502]: A071220883: message-id=<20150803220001.5E2AA52093C@mail2.les-moocs-gmf.fr".to_string();
		let cleanup = match parse_cleanup(s) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some(Message::Cleanup{m:x})) => x,
			Ok(Some(x)) => panic!("Wrong message parsed: {:?}", x)
		};
		assert_eq!(cleanup.message_id(), "20150803220001.5E2AA52093C@mail2.les-moocs-gmf.fr");
		assert_eq!(cleanup.resent, false);
		let s = "Aug  4 00:00:01 yuuai postfix-in/cleanup[22502]: A071220883: message-id=20150803220001.5E2AA52093C@mail2.les-moocs-gmf.fr".to_string();
		let cleanup = match parse_cleanup(s) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some(Message::Cleanup{m:x})) => x,
			Ok(Some(x)) => panic!("Wrong message parsed: {:?}", x)
		};
		assert_eq!(cleanup.message_id(), "20150803220001.5E2AA52093C@mail2.les-moocs-gmf.fr");
		assert_eq!(cleanup.resent, false);
		let s = "Aug  4 00:03:09 yuuai postfix-in/cleanup[22656]: 40A67208A3: resent-message-id=<PbhLmifNtVG.A.mh.ZU-vVB@bendel>".to_string();
		let cleanup = match parse_cleanup(s) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some(Message::Cleanup{m:x})) => x,
			Ok(Some(x)) => panic!("Wrong message parsed: {:?}", x)
		};
		assert_eq!(cleanup.message_id(), "PbhLmifNtVG.A.mh.ZU-vVB@bendel");
		assert_eq!(cleanup.resent, true);
		let s = "Aug  4 00:00:01 yuuai postfix-in/cleanup[22502]: A071220883: message-id=<20150803220001.5E2AA52093C@mail2.les-moocs-gmf.fr>".to_string();
		let cleanup = match parse_cleanup(s) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some(Message::Cleanup{m:x})) => x,
			Ok(Some(x)) => panic!("Wrong message parsed: {:?}", x)
		};
		assert_eq!(cleanup.message_id(), "20150803220001.5E2AA52093C@mail2.les-moocs-gmf.fr");
		assert_eq!(cleanup.resent, false);
		assert_eq!(fmt::format(format_args!("{:?}", cleanup)), "Cleanup { inner: Inner { raw: \"Aug  4 00:00:01 yuuai postfix-in/cleanup[22502]: A071220883: message-id=<20150803220001.5E2AA52093C@mail2.les-moocs-gmf.fr>\", host_e: 21, queue_s: 22, queue_e: 32, process: Cleanup, pid: 22502, queue_id_s: 49, queue_id_e: 59 }, message_id_s: 73, message_id_e: 122, resent: false }");
	}
}
