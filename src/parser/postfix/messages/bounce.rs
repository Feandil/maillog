use std::ops::Deref;
use super::super::ParseError;
use super::Inner;
use super::Message;
use super::MessageParser;

#[derive(Debug)]
pub struct Bounce {
	inner: Inner,
	child_queue_id_s: usize,
	child_queue_id_e: usize,
}

impl Deref for Bounce {
	type Target = Inner;
	fn deref(&self) -> &Inner {
		&self.inner
	}
}

impl Bounce {
	pub fn child_queue_id <'a>(&'a self) -> &'a str {
		&self.raw[self.child_queue_id_s..self.child_queue_id_e]
	}

}
impl MessageParser for Bounce {
	fn parse(inner: Inner, start: usize) -> Result<Option<Message>, ParseError> {
		let (child_queue_id_s, child_queue_id_e) = {
			let rest = &inner.raw[start..];
			if !rest.starts_with(" sender non-delivery notification: ") {
				return Err(ParseError::BounceBad);
			}
			let rest = &rest[35..];
			let child_queue_id_s = start + 35;
			if rest.bytes().any(|b| ('0' as u8 > b || b > '9' as u8) && ('A' as u8 > b || b > 'F' as u8)) {
				return Err(ParseError::BounceBadQueueID);
			}
			let child_queue_id_e = inner.raw.len();
			(child_queue_id_s, child_queue_id_e)
		};
		Ok(Some(Message::Bounce { m: Bounce { inner: inner, child_queue_id_s: child_queue_id_s, child_queue_id_e: child_queue_id_e } }))
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

	fn parse_bounce(s: String) -> Result<Option<Message>, ParseError> {
		let conf = ParserConfig { process_noise: vec!["clamsmtpd".to_string()] };
		let (inner, start) = match Inner::parse(&conf, s) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some((x,y))) => (x,y)
		};
		Bounce::parse(inner, start)
	}

	#[test]
	fn bad() {
		let s ="Aug  4 00:03:15 yuuai postfix/bounce[24350]: 7C091208A3:".to_string();
		match parse_bounce(s) {
			Err(ParseError::BounceBad) => (),
			Err(x) => panic!("Wrong error, should have been BounceBad {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(_) => panic!("This should have failed")
		};
	}

	#[test]
	fn bad_queue_id() {
		let s ="Aug  4 00:03:15 yuuai postfix/bounce[24350]: 7C091208A3: sender non-delivery notification: X".to_string();
		match parse_bounce(s) {
			Err(ParseError::BounceBadQueueID) => (),
			Err(x) => panic!("Wrong error, should have been BounceBad {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(_) => panic!("This should have failed")
		};
	}

	#[test]
	fn valid() {
		let s = "Aug  4 00:03:15 yuuai postfix/bounce[24350]: 7C091208A3: sender non-delivery notification: A270E20915".to_string();
		let bounce = match parse_bounce(s) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some(Message::Bounce{m:x})) => x,
			Ok(Some(x)) => panic!("Wrong message parsed: {:?}", x)
		};
		assert_eq!(bounce.child_queue_id(), "A270E20915");
		assert_eq!(fmt::format(format_args!("{:?}", bounce)), "Bounce { inner: Inner { raw: \"Aug  4 00:03:15 yuuai postfix/bounce[24350]: 7C091208A3: sender non-delivery notification: A270E20915\", host_e: 21, queue_s: 22, queue_e: 29, process: Bounce, pid: 24350, queue_id_s: 45, queue_id_e: 55 }, child_queue_id_s: 91, child_queue_id_e: 101 }");
	}
}
