use std::ops::Deref;
use super::super::ParseError;
use super::Inner;
use super::Message;
use super::MessageParser;

#[derive(Debug)]
pub struct Qmgr {
	inner: Inner,
	from_s: usize,
	from_e: usize,
	size: u64,
	nrcpt: u32,
}

#[derive(Debug)]
pub struct QmgrRemoved {
	inner: Inner,
}

#[derive(Debug)]
pub struct QmgrExpired {
	inner: Inner,
	from_s: usize,
	from_e: usize,
}

impl Deref for Qmgr {
	type Target = Inner;
	fn deref(&self) -> &Inner {
		&self.inner
	}
}

impl Deref for QmgrRemoved {
	type Target = Inner;
	fn deref(&self) -> &Inner {
		&self.inner
	}
}

impl Deref for QmgrExpired {
	type Target = Inner;
	fn deref(&self) -> &Inner {
		&self.inner
	}
}

impl Qmgr {
	pub fn from <'a>(&'a self) -> Option<&'a str> {
		if self.from_e != 0 {
			Some(&self.raw[self.from_s..self.from_e])
		} else {
			None
		}
	}
}

impl QmgrExpired {
	pub fn from <'a>(&'a self) -> Option<&'a str> {
		if self.from_e != 0 {
			Some(&self.raw[self.from_s..self.from_e])
		} else {
			None
		}
	}
}

impl MessageParser for Qmgr {
	fn parse(inner: Inner, start: usize) -> Result<Option<Message>, ParseError> {
		let removed = {
			let rest = &inner.raw[start..];
			rest.starts_with(" removed")
		};
		if removed {
			return Ok(Some(Message::QmgrRemoved { m: QmgrRemoved { inner: inner } }));
		}
		let (from_s, from_e) = {
			let rest = &inner.raw[start..];
			if !rest.starts_with(" from=<") {
				return Err(ParseError::QmgrNoFrom);
			}
			let rest = &rest[7..];
			let from_s = start + 7;
			let from_e = match rest.find('>') {
				None => return Err(ParseError::QmgrBadFrom),
				Some(p) => from_s + p
			};
			(from_s, from_e)
		};
		let expired = {
			inner.raw[from_e..].starts_with(">, status=expired, returned to sender")
		};
		if expired {
			return Ok(Some(Message::QmgrExpired { m: QmgrExpired {inner: inner, from_s: from_s, from_e: from_e } }));
		}
		let (size, nrcpt) = {
			let rest = &inner.raw[from_e..];
			if !rest.starts_with(">, size=") {
				return Err(ParseError::QmgrNoSize);
			}
			let rest = &rest[8..];
			let len = match rest.find(',') {
				None => return Err(ParseError::QmgrBadSize),
				Some(p) => p
			};
			let size = match rest[..len].parse::<u64>() {
				Err(_) => return Err(ParseError::QmgrSizeNotInt),
				Ok(val) => val
			};
			let rest = &rest[len..];
			if !rest.starts_with(", nrcpt=") {
				return Err(ParseError::QmgrNoNrcpt);
			}
			let rest = &rest[8..];
			let len = match rest.find(' ') {
				None => return Err(ParseError::QmgrBadNrcpt),
				Some(p) => p
			};
			if !rest[len..].starts_with(" (queue active)") {
				return Err(ParseError::QmgrNotActive);
			}
			let nrcpt = match rest[..len].parse::<u32>() {
				Err(_) => return Err(ParseError::QmgrNrcptNotInt),
				Ok(val) => val
			};
			(size, nrcpt)
		};
		Ok(Some(Message::Qmgr { m: Qmgr {inner: inner, from_s: from_s, from_e: from_e, size: size, nrcpt: nrcpt } }))
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

	fn parse_qmgr(s: String) -> Result<Option<Message>, ParseError> {
		let conf = ParserConfig { process_noise: vec!["clamsmtpd".to_string()] };
		let (inner, start) = match Inner::parse(&conf, s) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some((x,y))) => (x,y)
		};
		Qmgr::parse(inner, start)
	}

	#[test]
	fn no_from() {
		let s ="Jul 25 00:00:06 svoboda postfix/qmgr[32099]: DF83C1409B04F: from=".to_string();
		match parse_qmgr(s) {
			Err(ParseError::QmgrNoFrom) => (),
			Err(x) => panic!("Wrong error, should have been QmgrNoFrom {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(_) => panic!("This should have failed")
		};
	}

	#[test]
	fn bad_from() {
		let s ="Jul 25 00:00:06 svoboda postfix/qmgr[32099]: DF83C1409B04F: from=<".to_string();
		match parse_qmgr(s) {
			Err(ParseError::QmgrBadFrom) => (),
			Err(x) => panic!("Wrong error, should have been QmgrBadFrom {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(_) => panic!("This should have failed")
		};
	}

	#[test]
	fn no_size() {
		let s ="Jul 25 00:00:06 svoboda postfix/qmgr[32099]: DF83C1409B04F: from=<>, size".to_string();
		match parse_qmgr(s) {
			Err(ParseError::QmgrNoSize) => (),
			Err(x) => panic!("Wrong error, should have been QmgrNoSize {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(_) => panic!("This should have failed")
		};
	}

	#[test]
	fn bad_size() {
		let s ="Jul 25 00:00:06 svoboda postfix/qmgr[32099]: DF83C1409B04F: from=<>, size=)".to_string();
		match parse_qmgr(s) {
			Err(ParseError::QmgrBadSize) => (),
			Err(x) => panic!("Wrong error, should have been QmgrBadSize {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(_) => panic!("This should have failed")
		};
	}

	#[test]
	fn size_not_int() {
		let s ="Jul 25 00:00:06 svoboda postfix/qmgr[32099]: DF83C1409B04F: from=<>, size=Xyz,".to_string();
		match parse_qmgr(s) {
			Err(ParseError::QmgrSizeNotInt) => (),
			Err(x) => panic!("Wrong error, should have been QmgrSizeNotInt {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(_) => panic!("This should have failed")
		};
	}

	#[test]
	fn no_nrcpt() {
		let s ="Jul 25 00:00:06 svoboda postfix/qmgr[32099]: DF83C1409B04F: from=<>, size=0, nrcpt".to_string();
		match parse_qmgr(s) {
			Err(ParseError::QmgrNoNrcpt) => (),
			Err(x) => panic!("Wrong error, should have been QmgrNoNrcpt {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(_) => panic!("This should have failed")
		};
	}

	#[test]
	fn bad_nrcpt() {
		let s ="Jul 25 00:00:06 svoboda postfix/qmgr[32099]: DF83C1409B04F: from=<>, size=0, nrcpt=".to_string();
		match parse_qmgr(s) {
			Err(ParseError::QmgrBadNrcpt) => (),
			Err(x) => panic!("Wrong error, should have been QmgrBadNrcpt {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(_) => panic!("This should have failed")
		};
	}

	#[test]
	fn nrcpt_not_active() {
		let s ="Jul 25 00:00:06 svoboda postfix/qmgr[32099]: DF83C1409B04F: from=<>, size=0, nrcpt= ".to_string();
		match parse_qmgr(s) {
			Err(ParseError::QmgrNotActive) => (),
			Err(x) => panic!("Wrong error, should have been QmgrNotActive {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(_) => panic!("This should have failed")
		};
	}

	#[test]
	fn nrcpt_not_int() {
		let s ="Jul 25 00:00:06 svoboda postfix/qmgr[32099]: DF83C1409B04F: from=<>, size=0, nrcpt=Xyz (queue active)".to_string();
		match parse_qmgr(s) {
			Err(ParseError::QmgrNrcptNotInt) => (),
			Err(x) => panic!("Wrong error, should have been QmgrNrcptNotInt {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(_) => panic!("This should have failed")
		};
	}

	#[test]
	fn valid_active() {
		let s = "Jul 25 00:00:01 svoboda postfix/qmgr[32099]: 77A8F1409B022: from=<validation@polytechnique.org>, size=665, nrcpt=1 (queue active)".to_string();
		let qmgr = match parse_qmgr(s) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some(Message::Qmgr{m:x})) => x,
			Ok(Some(x)) => panic!("Wrong message parsed: {:?}", x)
		};
		match qmgr.from() {
			None => panic!("From not found"),
			Some(f) => assert_eq!(f, "validation@polytechnique.org")
		};
		assert_eq!(qmgr.size, 665);
		assert_eq!(qmgr.nrcpt, 1);
		assert_eq!(fmt::format(format_args!("{:?}", qmgr)), "Qmgr { inner: Inner { raw: \"Jul 25 00:00:01 svoboda postfix/qmgr[32099]: 77A8F1409B022: from=<validation@polytechnique.org>, size=665, nrcpt=1 (queue active)\", host_e: 23, queue_s: 24, queue_e: 31, process: Qmgr, pid: 32099, queue_id_s: 45, queue_id_e: 58 }, from_s: 66, from_e: 94, size: 665, nrcpt: 1 }");
	}

	#[test]
	fn valid_removed() {
		let s = "Jul 25 00:00:03 svoboda postfix/qmgr[32099]: 77A8F1409B022: removed".to_string();
		let qmgr = match parse_qmgr(s) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some(Message::QmgrRemoved{m:x})) => x,
			Ok(Some(x)) => panic!("Wrong message parsed: {:?}", x)
		};
		assert_eq!(fmt::format(format_args!("{:?}", qmgr)), "QmgrRemoved { inner: Inner { raw: \"Jul 25 00:00:03 svoboda postfix/qmgr[32099]: 77A8F1409B022: removed\", host_e: 23, queue_s: 24, queue_e: 31, process: Qmgr, pid: 32099, queue_id_s: 45, queue_id_e: 58 } }");
	}

	#[test]
	fn valid_expired() {
		let s = "Jul 25 00:08:51 yuuai postfix/qmgr[4146]: BB3B220B19: from=<>, status=expired, returned to sender".to_string();
		let qmgr = match parse_qmgr(s) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some(Message::QmgrExpired{m:x})) => x,
			Ok(Some(x)) => panic!("Wrong message parsed: {:?}", x)
		};
		match qmgr.from() {
			None => panic!("From not found"),
			Some(f) => assert_eq!(f, "")
		};
		assert_eq!(fmt::format(format_args!("{:?}", qmgr)), "QmgrExpired { inner: Inner { raw: \"Jul 25 00:08:51 yuuai postfix/qmgr[4146]: BB3B220B19: from=<>, status=expired, returned to sender\", host_e: 21, queue_s: 22, queue_e: 29, process: Qmgr, pid: 4146, queue_id_s: 42, queue_id_e: 52 }, from_s: 60, from_e: 60 }");
	}
}
