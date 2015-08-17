use std::ops::Deref;
use super::super::ParseError;
use super::Inner;

#[derive(Debug)]
pub struct Cleanup {
	inner: Inner,
	message_id_s: usize,
	message_id_e: usize,
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
	pub fn parse(inner: Inner, start: usize) -> Result<Option<Cleanup>, ParseError> {
		let (message_id_s, message_id_e) = {
			let rest = &inner.raw[start..];
			if  !rest.starts_with(" message-id=") {
				return Err(ParseError::CleanupNoMessageID);
			}
			let rest = &rest[12..];
			let mut message_id_s = start + 12;
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
			(message_id_s, message_id_e)
		};
		Ok(Some(Cleanup {inner: inner, message_id_s: message_id_s, message_id_e:message_id_e}))
	}
}


#[cfg(test)]
mod tests {
	use std::fmt;
	use super::*;
	use super::super::Inner;
	use super::super::super::ParserConfig;
	use super::super::super::ParseError;

	fn parse_cleanup(s: String) -> Result<Option<Cleanup>, ParseError> {
		let conf = ParserConfig { process_noise: vec!["clamsmtpd".to_string()] };
		let (inner, start) = match Inner::parse(&conf, s) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some((x,y))) => (x,y)
		};
		Cleanup::parse(inner, start)
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
			Ok(Some(x)) => x
		};
		assert_eq!(cleanup.message_id(), "20150803220001.5E2AA52093C@mail2.les-moocs-gmf.fr");
		let s = "Aug  4 00:00:01 yuuai postfix-in/cleanup[22502]: A071220883: message-id=20150803220001.5E2AA52093C@mail2.les-moocs-gmf.fr".to_string();
		let cleanup = match parse_cleanup(s) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some(x)) => x
		};
		assert_eq!(cleanup.message_id(), "20150803220001.5E2AA52093C@mail2.les-moocs-gmf.fr");
		let s = "Aug  4 00:00:01 yuuai postfix-in/cleanup[22502]: A071220883: message-id=<20150803220001.5E2AA52093C@mail2.les-moocs-gmf.fr>".to_string();
		let cleanup = match parse_cleanup(s) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some(x)) => x
		};
		assert_eq!(cleanup.message_id(), "20150803220001.5E2AA52093C@mail2.les-moocs-gmf.fr");
		assert_eq!(fmt::format(format_args!("{:?}", cleanup)), "Cleanup { inner: Inner { raw: \"Aug  4 00:00:01 yuuai postfix-in/cleanup[22502]: A071220883: message-id=<20150803220001.5E2AA52093C@mail2.les-moocs-gmf.fr>\", host_e: 21, queue_s: 22, queue_e: 32, process: Cleanup, pid: 22502, queue_id_s: 49, queue_id_e: 59 }, message_id_s: 73, message_id_e: 122 }");
	}
}
