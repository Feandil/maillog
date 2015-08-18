use std::ops::Deref;
use super::super::ParseError;
use super::Inner;

#[derive(Debug)]
pub struct Smtpd {
	inner: Inner,
	client_s: usize,
	client_e: usize,
	orig_queue_id_s: usize,
	orig_queue_id_e: usize,
	orig_client_s: usize,
	orig_client_e: usize,
	sasl_username_s: usize,
	sasl_username_e: usize,
}

impl Deref for Smtpd {
	type Target = Inner;
	fn deref(&self) -> &Inner {
		&self.inner
	}
}

impl Smtpd {
	pub fn client <'a>(&'a self) -> &'a str {
		&self.raw[self.client_s..self.client_e]
	}
	pub fn orig_queue_id <'a>(&'a self) -> Option<&'a str> {
		if self.orig_queue_id_e != 0 {
			Some(&self.raw[self.orig_queue_id_s..self.orig_queue_id_e])
		} else {
			None
		}
	}
	pub fn orig_client <'a>(&'a self) -> Option<&'a str> {
		if self.orig_client_e != 0 {
			Some(&self.raw[self.orig_client_s..self.orig_client_e])
		} else {
			None
		}
	}
	pub fn sasl_username <'a>(&'a self) -> Option<&'a str> {
		if self.sasl_username_e != 0 {
			Some(&self.raw[self.sasl_username_s..self.sasl_username_e])
		} else {
			None
		}
	}
	pub fn parse(inner: Inner, start: usize) -> Result<Option<Smtpd>, ParseError> {
		match inner.queue_id() {
			None => return Ok(None),
			Some(_) => ()
		};
		let (client_s, client_e, orig_queue_id_s, orig_queue_id_e, orig_client_s, orig_client_e, sasl_username_s, sasl_username_e) = {
			let rest = &inner.raw[start..];
			if  !rest.starts_with(" client=") {
				return Err(ParseError::PickupBadUID);
			}
			let rest = &rest[8..];
			let client_s = start + 8;
			let (done, client_e) =  match rest.find(',') {
				None => (true, client_s + rest.len()),
				Some(p) => (false, client_s + p)
			};
			if done {
				(client_s, client_e, 0, 0, 0, 0, 0, 0)
			} else {
				let rest = &inner.raw[client_e ..];
				if rest.starts_with(", orig_queue_id=") {
					let orig_queue_id_s = client_e + 16;
					let rest = &rest[16..];
					let orig_queue_id_e = match rest.find(',') {
						None => return Err(ParseError::SmtpdNonEndingOrigQueue),
						Some(p) => orig_queue_id_s + p,
					};
					let rest = &inner.raw[orig_queue_id_e..];
					if !rest.starts_with(", orig_client=") {
						return Err(ParseError::SmtpdNoOrigClient);
					}
					let orig_client_s = orig_queue_id_e + 14;
					let orig_client_e = inner.raw.len();
					(client_s, client_e, orig_queue_id_s, orig_queue_id_e, orig_client_s, orig_client_e, 0, 0)
				} else if rest.starts_with(", sasl_method=LOGIN, sasl_username=") {
					let sasl_username_s = client_e + 35;
					let sasl_username_e = inner.raw.len();
					(client_s, client_e, 0, 0, 0, 0, sasl_username_s, sasl_username_e)
				} else {
					return Err(ParseError::SmtpdBadOrigQueue);
				}
			}

		};
		Ok(Some(Smtpd {inner: inner, client_s: client_s, client_e: client_e, orig_queue_id_s: orig_queue_id_s, orig_queue_id_e: orig_queue_id_e, orig_client_s: orig_client_s, orig_client_e:orig_client_e, sasl_username_s:sasl_username_s, sasl_username_e:sasl_username_e}))
	}
}


#[cfg(test)]
mod tests {
	use std::fmt;
	use super::*;
	use super::super::Inner;
	use super::super::super::ParserConfig;
	use super::super::super::ParseError;

	fn parse_smtpd(s: String) -> Result<Option<Smtpd>, ParseError> {
		let conf = ParserConfig { process_noise: vec!["clamsmtpd".to_string()] };
		let (inner, start) = match Inner::parse(&conf, s) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some((x,y))) => (x,y)
		};
		Smtpd::parse(inner, start)
	}

	#[test]
	fn no_queue_id() {
		let s = "Aug  4 00:00:09 yuuai postfix/smtpd[20518]: disconnect from mx1[129.104.30.34]".to_string();
		match parse_smtpd(s) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => (),
			Ok(x) => panic!("This should have been ignored ({})", fmt::format(format_args!("{:?}", x)))
		};
		let s = "Aug  4 00:00:12 ozgurluk postfix/smtpd[25688]: warning: hostname pei-190-128-lxiii-xxx.une.net.co does not resolve to address 190.128.63.30: Name or service not known".to_string();
		match parse_smtpd(s) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => (),
			Ok(x) => panic!("This should have been ignored ({})", fmt::format(format_args!("{:?}", x)))
		};
	}

	#[test]
	fn bad_orig_queue() {
		let s ="Aug  4 00:00:08 yuuai postfix/smtpd.local[20039]: 84ED020916: client=localhost[127.0.0.1], orig_queue_id".to_string();
		match parse_smtpd(s) {
			Err(ParseError::SmtpdBadOrigQueue) => (),
			Err(x) => panic!("Wrong error, should have been SmtpdBadOrigQueue {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(_) => panic!("This should have failed")
		};
	}

	#[test]
	fn non_ending_orig_queue() {
		let s ="Aug  4 00:00:08 yuuai postfix/smtpd.local[20039]: 84ED020916: client=localhost[127.0.0.1], orig_queue_id=".to_string();
		match parse_smtpd(s) {
			Err(ParseError::SmtpdNonEndingOrigQueue) => (),
			Err(x) => panic!("Wrong error, should have been SmtpdNonEndingOrigQueue {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(_) => panic!("This should have failed")
		};
	}

	#[test]
	fn no_orig_client() {
		let s ="Aug  4 00:00:08 yuuai postfix/smtpd.local[20039]: 84ED020916: client=localhost[127.0.0.1], orig_queue_id=67D8720887,".to_string();
		match parse_smtpd(s) {
			Err(ParseError::SmtpdNoOrigClient) => (),
			Err(x) => panic!("Wrong error, should have been SmtpdNoOrigClient {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(_) => panic!("This should have failed")
		};
	}

	#[test]
	fn valid_with_orig() {
		let s = "Aug  4 00:00:08 yuuai postfix/smtpd.local[20039]: 84ED020916: client=localhost[127.0.0.1], orig_queue_id=67D8720887, orig_client=3.mo52.mail-out.ovh.net[178.33.254.192]".to_string();
		let smtpd = match parse_smtpd(s) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some(x)) => x
		};
		assert_eq!(smtpd.client(), "localhost[127.0.0.1]");
		match smtpd.orig_queue_id() {
			None => panic!("Incorrectly parsed the orig_queue_id"),
			Some(s) => assert_eq!(s, "67D8720887")
		};
		match smtpd.orig_client() {
			None => panic!("Incorrectly parsed the orig_queue_id"),
			Some(s) => assert_eq!(s, "3.mo52.mail-out.ovh.net[178.33.254.192]")
		};
		match smtpd.sasl_username() {
			None => (),
			Some(s) => panic!("Parsed a non existant sasl_username {}", s)
		};
		assert_eq!(fmt::format(format_args!("{:?}", smtpd)), "Smtpd { inner: Inner { raw: \"Aug  4 00:00:08 yuuai postfix/smtpd.local[20039]: 84ED020916: client=localhost[127.0.0.1], orig_queue_id=67D8720887, orig_client=3.mo52.mail-out.ovh.net[178.33.254.192]\", host_e: 21, queue_s: 22, queue_e: 29, process: Smtpd, pid: 20039, queue_id_s: 50, queue_id_e: 60 }, client_s: 69, client_e: 89, orig_queue_id_s: 105, orig_queue_id_e: 115, orig_client_s: 129, orig_client_e: 168, sasl_username_s: 0, sasl_username_e: 0 }");
	}

	#[test]
	fn valid_simple() {
		let s = "Aug  4 00:00:08 yuuai postfix/smtpd.local[20039]: 84ED020916: client=3.mo52.mail-out.ovh.net[178.33.254.192]".to_string();
		let smtpd = match parse_smtpd(s) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some(x)) => x
		};
		assert_eq!(smtpd.client(), "3.mo52.mail-out.ovh.net[178.33.254.192]");
		match smtpd.orig_queue_id() {
			None => (),
			Some(s) => panic!("Parsed a non existing orig_queue_id {}", s)
		};
		match smtpd.orig_client() {
			None => (),
			Some(s) => panic!("Parsed a non existing orig_client {}", s)
		};
		match smtpd.sasl_username() {
			None => (),
			Some(s) => panic!("Parsed a non existant sasl_username {}", s)
		};
	}

	#[test]
	fn valid_with_login() {
		let s = "Jul 25 00:00:09 svoboda postfix/smtpd[5884]: 87E611409B022: client=99-46-141-195.lightspeed.sntcca.sbcglobal.net[99.46.141.195], sasl_method=LOGIN, sasl_username=firstname.lastname".to_string();
		let smtpd = match parse_smtpd(s) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some(x)) => x
		};
		assert_eq!(smtpd.client(), "99-46-141-195.lightspeed.sntcca.sbcglobal.net[99.46.141.195]");
		match smtpd.orig_queue_id() {
			None => (),
			Some(s) => panic!("Parsed a non existing orig_queue_id {}", s)
		};
		match smtpd.orig_client() {
			None => (),
			Some(s) => panic!("Parsed a non existing orig_client {}", s)
		};
		match smtpd.sasl_username() {
			None => panic!("Failed to parse the sasl_username"),
			Some(s) => assert_eq!(s, "firstname.lastname")
		};
	}
}
