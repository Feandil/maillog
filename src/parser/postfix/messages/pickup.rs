use std::ops::Deref;
use super::super::config::ParserConfig;
use super::super::errors::ParseError;
use super::super::inner::Inner;

#[derive(Debug)]
pub struct Pickup {
	inner: Inner,
	pub uid: u32,
	from_s: usize,
	from_e: usize,
}

impl Deref for Pickup {
	type Target = Inner;
	fn deref(&self) -> &Inner {
		&self.inner
	}
}

impl Pickup {
	pub fn from <'a>(&'a self) -> &'a str {
		&self.raw[self.from_s..self.from_e]
	}
	pub fn parse(config: ParserConfig, s: String) -> Result<Option<Pickup>, ParseError> {
		let (inner, start) = match Inner::parse(config, s) {
			Err(x) => return Err(x),
			Ok(None) => return Ok(None),
			Ok(Some((x,y))) => (x,y)
		};
		let (uid, from_s, from_e) = {
			let rest = &inner.raw[start..];
			if  !rest.starts_with(" uid=") {
				return Err(ParseError::PickupBadUID);
			}
			let rest = &rest[5..];
			let pos = match rest.find(' ') {
				None => return Err(ParseError::PickupBadUID),
				Some(p) => p
			};
			let uid = match rest[..pos].parse::<u32>() {
				Err(_) => return Err(ParseError::PickupBadUID),
				Ok(val) => val
			};
			let rest = &rest[pos+1..];
			let pos = start + 5 + pos + 1;
			if !rest.starts_with("from=") {
				return Err(ParseError::PickupBadFrom);
			}
			let from_s = if &rest[5..6] == "<" { pos + 6 } else { pos + 5 };
			let mut end = rest.len();
			if &rest[end-1..end] == "\n" { end = end - 1; }
			if &rest[end-1..end] == ">" { end = end - 1; }
			(uid, from_s, pos + end)
		};

		Ok(Some(Pickup {inner: inner, uid: uid, from_s: from_s, from_e: from_e}))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use super::super::super::inner::Inner;
	use super::super::super::config::ParserConfig;
	use super::super::super::errors::ParseError;

	fn parse_pickup(s: String) -> Result<Option<Pickup>, ParseError> {
		let conf = ParserConfig { process_noise: vec!["clamsmtpd".to_string()] };
		Pickup::parse(conf, s)
	}

	#[test]
	fn parse_valid() {
		let s = "Sep  3 00:00:03 yuuai postfix/pickup[12797]: 12C172090B: uid=106 from=<root@example.com>".to_string();
		let pick = match parse_pickup(s) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some(x)) => x
		};
		assert_eq!(pick.uid, 106);
		assert_eq!(pick.from(), "root@example.com");
		let s = "Sep  3 00:00:03 yuuai postfix/pickup[12797]: 12C172090B: uid=1024 from=root@example.com".to_string();
		let pick = match parse_pickup(s) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some(x)) => x
		};
		assert_eq!(pick.uid, 1024);
		assert_eq!(pick.from(), "root@example.com");
	}

	#[test]
	fn parse_broken_uid() {
		let s = "Sep  3 00:00:03 yuuai postfix/pickup[12797]: 12C172090B: uid".to_string();
		let pick = match parse_pickup(s) {
			Err(ParseError::PickupBadUID) => (),
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some(x)) => panic!("This should not have worked: the uid is bad")
		};
		let s = "Sep  3 00:00:03 yuuai postfix/pickup[12797]: 12C172090B: uid=".to_string();
		let pick = match parse_pickup(s) {
			Err(ParseError::PickupBadUID) => (),
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some(x)) => panic!("This should not have worked: the uid is bad")
		};
		let s = "Sep  3 00:00:03 yuuai postfix/pickup[12797]: 12C172090B: uid= ".to_string();
		let pick = match parse_pickup(s) {
			Err(ParseError::PickupBadUID) => (),
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some(x)) => panic!("This should not have worked: the uid is bad")
		};
		let s = "Sep  3 00:00:03 yuuai postfix/pickup[12797]: 12C172090B: uid=xxx ".to_string();
		let pick = match parse_pickup(s) {
			Err(ParseError::PickupBadUID) => (),
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some(x)) => panic!("This should not have worked: the uid is bad")
		};
	}

	#[test]
	fn parse_broken_from() {
		let s = "Sep  3 00:00:03 yuuai postfix/pickup[12797]: 12C172090B: uid=106 from".to_string();
		let pick = match parse_pickup(s) {
			Err(ParseError::PickupBadFrom) => (),
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some(x)) => panic!("This should not have worked: the uid is bad")
		};
	}
}

