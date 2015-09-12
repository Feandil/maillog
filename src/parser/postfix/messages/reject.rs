use std::fmt;
use std::ops::Deref;
use super::super::ParseError;
use super::Inner;
use super::Message;

pub enum RejectReason {
	Reject,
	Discard,
	Warn,
}

pub enum RejectProto {
	SMTP,
	ESMTP,
}

#[derive(Debug)]
pub struct Reject {
	inner: Inner,
	pub reason: RejectReason,
	message_s: usize,
	message_e: usize,
	from_s: usize,
	from_e: usize,
	to_s: usize,
	to_e: usize,
	pub proto: RejectProto,
	helo_s: usize,
	helo_e: usize,
}

impl Deref for Reject {
	type Target = Inner;
	fn deref(&self) -> &Inner {
		&self.inner
	}
}

impl fmt::Display for RejectReason {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		let error = match self {
			&RejectReason::Reject => "Reject",
			&RejectReason::Discard => "Discard",
			&RejectReason::Warn => "Warn",
		};
		write!(fmt, "{}", error)
	}
}

impl fmt::Debug for RejectReason {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt::Display::fmt(self, fmt)
	}
}

impl fmt::Display for RejectProto {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		let error = match self {
			&RejectProto::SMTP => "SMTP",
			&RejectProto::ESMTP => "ESMTP",
		};
		write!(fmt, "{}", error)
	}
}

impl fmt::Debug for RejectProto {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt::Display::fmt(self, fmt)
	}
}

impl Reject {
	pub fn message <'a>(&'a self) -> &'a str {
		&self.raw[self.message_s..self.message_e]
	}
	pub fn from <'a>(&'a self) -> &'a str {
		&self.raw[self.from_s..self.from_e]
	}
	pub fn to <'a>(&'a self) -> Option<&'a str> {
		if self.to_e != 0 {
			Some(&self.raw[self.to_s..self.to_e])
		} else {
			None
		}
	}
	pub fn helo <'a>(&'a self) -> &'a str {
		&self.raw[self.helo_s..self.helo_e]
	}
}

impl Reject {
	pub fn parse(inner: Inner, start: usize, reason: RejectReason) -> Result<Option<Message>, ParseError> {
		let (message_s, message_e, from_s, from_e, to_s, to_e, proto, helo_s, helo_e) = {
			let message_s = match reason {
				RejectReason::Discard => start + 10,
				RejectReason::Reject => start + 9,
				RejectReason::Warn => start + 7,
			};
			let rest = &inner.raw[message_s..];
			let pos = match rest.find(';') {
				None => return Err(ParseError::RejectBadMessage),
				Some(p) => p
			};
			let rest = &rest[pos..];
			let message_e = message_s + pos;
			if !rest.starts_with("; from=<") {
				return Err(ParseError::RejectNoFrom);
			}
			let from_s = message_e + 8;
			let rest = &rest[8..];
			let pos = match rest.find('>') {
				None => return Err(ParseError::RejectBadFrom),
				Some(p) => p
			};
			let rest = &rest[pos..];
			let from_e = from_s + pos;
			let (rest, end, to_s, to_e) = {
				if rest.starts_with("> to=<") {
					let rest = &rest[6..];
					let to_s = from_e + 6;
					let pos = match rest.find('>') {
						None => return Err(ParseError::RejectBadTo),
						Some(p) => p
					};
					let rest = &rest[pos..];
					let to_e = to_s + pos;
					(rest, to_e, to_s, to_e)
				} else {
					(rest, from_e, 0, 0)
				}
			};
			if !rest.starts_with("> proto=") {
				return Err(ParseError::RejectNoProto);
			}
			let rest = &rest[8..];
			let pos = match rest.find(' ') {
				None => return Err(ParseError::RejectBadProto),
				Some(p) => p
			};
			let proto = match &rest[..pos] {
				"SMTP" => RejectProto::SMTP,
				"ESMTP" => RejectProto::ESMTP,
				_ => return Err(ParseError::RejectUnknownProto)
			};
			let rest = &rest[pos..];
			if !rest.starts_with(" helo=<") {
				return Err(ParseError::RejectNoHelo);
			}
			let rest = &rest[7..];
			let helo_s = end + 8 + pos + 7;
			let helo_e = match rest.find('>') {
				None => return Err(ParseError::RejectBadHelo),
				Some(p) => helo_s + p
			};
			(message_s, message_e, from_s, from_e, to_s, to_e, proto, helo_s, helo_e)
		};
		Ok(Some(Message::Reject { m: Reject { inner: inner, reason:reason, message_s:message_s, message_e:message_e, from_s:from_s, from_e:from_e, to_s:to_s, to_e:to_e, proto:proto, helo_s:helo_s, helo_e:helo_e } }))
	}
}


#[cfg(test)]
mod tests {
	use super::*;
	use super::super::Inner;
	use super::super::Message;
	use super::super::MessageParser;
	use super::super::super::ParserConfig;
	use super::super::super::ParseError;

	fn parse_reject(s: String, reason: RejectReason) -> Result<Option<Message>, ParseError> {
		let conf = ParserConfig { process_noise: vec!["clamsmtpd".to_string()] };
		let (inner, start) = match Inner::parse(&conf, s) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some((x,y))) => (x,y)
		};
		Reject::parse(inner, start, reason)
	}

	#[test]
	fn bad_message() {
		let s = "Aug  4 00:00:12 yuuai postfix/smtpd[3199]: 89EF32091D: discard: DATA from scm.seog.co.kr[61.36.79.99]: <DATA>: Data command Recipient list contains a blacklisted address".to_string();
		match parse_reject(s, RejectReason::Discard) {
			Err(ParseError::RejectBadMessage) => (),
			Err(x) => panic!("Wrong error, should have been RejectBadMessage {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(_) => panic!("This should have failed")
		};
	}

	#[test]
	fn no_from() {
		let s = "Jul 25 00:00:09 svoboda postfix/smtpd[5884]: 87E611409B022: reject: DATA from 99-46-141-195.lightspeed.sntcca.sbcglobal.net[99.46.141.195]: 421 4.7.1 <DATA>: Data command rejected: Tu (firstname.lastname) as envoye trop de mails recement. Merci de contacter le support s'il s'agit d'une erreur;".to_string();
		match parse_reject(s, RejectReason::Reject) {
			Err(ParseError::RejectNoFrom) => (),
			Err(x) => panic!("Wrong error, should have been RejectNoFrom {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(_) => panic!("This should have failed")
		};
	}

	#[test]
	fn bad_from() {
		let s = "Aug  4 00:00:12 yuuai postfix/smtpd[3199]: 89EF32091D: discard: DATA from scm.seog.co.kr[61.36.79.99]: <DATA>: Data command Recipient list contains a blacklisted address; from=<xyz".to_string();
		match parse_reject(s, RejectReason::Discard) {
			Err(ParseError::RejectBadFrom) => (),
			Err(x) => panic!("Wrong error, should have been RejectBadFrom {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(_) => panic!("This should have failed")
		};
	}

	#[test]
	fn bad_to() {
		let s = "Jul 25 00:00:09 svoboda postfix/smtpd[5884]: 87E611409B022: reject: DATA from 99-46-141-195.lightspeed.sntcca.sbcglobal.net[99.46.141.195]: 421 4.7.1 <DATA>: Data command rejected: Tu (firstname.lastname) as envoye trop de mails recement. Merci de contacter le support s'il s'agit d'une erreur; from=<firstname.lastname@m4x.org> to=<firstname.lastname@m4x.org".to_string();
		match parse_reject(s, RejectReason::Reject) {
			Err(ParseError::RejectBadTo) => (),
			Err(x) => panic!("Wrong error, should have been RejectBadTo {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(_) => panic!("This should have failed")
		};
	}

	#[test]
	fn no_proto() {
		let s = "Aug  4 00:00:12 yuuai postfix/smtpd[3199]: 89EF32091D: discard: DATA from scm.seog.co.kr[61.36.79.99]: <DATA>: Data command Recipient list contains a blacklisted address; from=<massnewsletter4654654xel@gmail.com>".to_string();
		match parse_reject(s, RejectReason::Discard) {
			Err(ParseError::RejectNoProto) => (),
			Err(x) => panic!("Wrong error, should have been RejectNoProto {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(_) => panic!("This should have failed")
		};
	}

	#[test]
	fn bad_proto() {
		let s = "Jul 25 00:00:09 svoboda postfix/smtpd[5884]: 87E611409B022: reject: DATA from 99-46-141-195.lightspeed.sntcca.sbcglobal.net[99.46.141.195]: 421 4.7.1 <DATA>: Data command rejected: Tu (firstname.lastname) as envoye trop de mails recement. Merci de contacter le support s'il s'agit d'une erreur; from=<firstname.lastname@m4x.org> to=<firstname.lastname@m4x.org> proto=".to_string();
		match parse_reject(s, RejectReason::Reject) {
			Err(ParseError::RejectBadProto) => (),
			Err(x) => panic!("Wrong error, should have been RejectBadProto {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(_) => panic!("This should have failed")
		};
	}

	#[test]
	fn unknown_proto() {
		let s = "Aug  4 00:00:12 yuuai postfix/smtpd[3199]: 89EF32091D: discard: DATA from scm.seog.co.kr[61.36.79.99]: <DATA>: Data command Recipient list contains a blacklisted address; from=<massnewsletter4654654xel@gmail.com> proto=XYZ ".to_string();
		match parse_reject(s, RejectReason::Discard) {
			Err(ParseError::RejectUnknownProto) => (),
			Err(x) => panic!("Wrong error, should have been RejectUnknownProto {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(_) => panic!("This should have failed")
		};
	}

	#[test]
	fn no_helo() {
		let s = "Jul 25 00:00:09 svoboda postfix/smtpd[5884]: 87E611409B022: reject: DATA from 99-46-141-195.lightspeed.sntcca.sbcglobal.net[99.46.141.195]: 421 4.7.1 <DATA>: Data command rejected: Tu (firstname.lastname) as envoye trop de mails recement. Merci de contacter le support s'il s'agit d'une erreur; from=<firstname.lastname@m4x.org> to=<firstname.lastname@m4x.org> proto=ESMTP ".to_string();
		match parse_reject(s, RejectReason::Reject) {
			Err(ParseError::RejectNoHelo) => (),
			Err(x) => panic!("Wrong error, should have been RejectNoHelo {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(_) => panic!("This should have failed")
		};
	}

	#[test]
	fn bad_helo() {
		let s = "Aug  4 00:00:12 yuuai postfix/smtpd[3199]: 89EF32091D: discard: DATA from scm.seog.co.kr[61.36.79.99]: <DATA>: Data command Recipient list contains a blacklisted address; from=<massnewsletter4654654xel@gmail.com> proto=SMTP helo=<gmail.com".to_string();
		match parse_reject(s, RejectReason::Discard) {
			Err(ParseError::RejectBadHelo) => (),
			Err(x) => panic!("Wrong error, should have been RejectBadHelo {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(_) => panic!("This should have failed")
		};
	}

	// Valid tests are made in the different callers
}
