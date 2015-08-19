use std::fmt;
use std::ops::Deref;
use super::super::ParseError;
use super::Inner;
use super::Message;
use super::MessageParser;

#[derive(Debug)]
pub struct Smtpd {
	inner: Inner,
	client_s: usize,
	client_e: usize,
}

#[derive(Debug)]
pub struct SmtpdForward {
	smtpd: Smtpd,
	orig_queue_id_s: usize,
	orig_queue_id_e: usize,
	orig_client_s: usize,
	orig_client_e: usize,
}

#[derive(Debug)]
pub struct SmtpdLogin {
	smtpd: Smtpd,
	sasl_username_s: usize,
	sasl_username_e: usize,
}

pub enum SmtpdBadReason {
	Reject,
	Discard,
}

pub enum SmtpdProto {
	SMTP,
	ESMTP,
}

#[derive(Debug)]
pub struct SmtpdBad {
	inner: Inner,
	reason: SmtpdBadReason,
	message_s: usize,
	message_e: usize,
	from_s: usize,
	from_e: usize,
	to_s: usize,
	to_e: usize,
	proto: SmtpdProto,
	helo_s: usize,
	helo_e: usize,
}

impl Deref for Smtpd {
	type Target = Inner;
	fn deref(&self) -> &Inner {
		&self.inner
	}
}

impl Deref for SmtpdForward {
	type Target = Smtpd;
	fn deref(&self) -> &Smtpd {
		&self.smtpd
	}
}

impl Deref for SmtpdLogin {
	type Target = Smtpd;
	fn deref(&self) -> &Smtpd {
		&self.smtpd
	}
}

impl Deref for SmtpdBad {
	type Target = Inner;
	fn deref(&self) -> &Inner {
		&self.inner
	}
}

impl fmt::Display for SmtpdBadReason {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		let error = match self {
			&SmtpdBadReason::Reject => "Reject",
			&SmtpdBadReason::Discard => "Discard",
		};
		write!(fmt, "{}", error)
	}
}

impl fmt::Debug for SmtpdBadReason {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt::Display::fmt(self, fmt)
	}
}

impl fmt::Display for SmtpdProto {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		let error = match self {
			&SmtpdProto::SMTP => "SMTP",
			&SmtpdProto::ESMTP => "ESMTP",
		};
		write!(fmt, "{}", error)
	}
}

impl fmt::Debug for SmtpdProto {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		fmt::Display::fmt(self, fmt)
	}
}

impl Smtpd {
	pub fn client <'a>(&'a self) -> &'a str {
		&self.raw[self.client_s..self.client_e]
	}
}

impl SmtpdForward {
	pub fn orig_queue_id <'a>(&'a self) -> &'a str {
		&self.raw[self.orig_queue_id_s..self.orig_queue_id_e]
	}
	pub fn orig_client <'a>(&'a self) -> &'a str {
		&self.raw[self.orig_client_s..self.orig_client_e]
	}
}

impl SmtpdLogin {
	pub fn sasl_username <'a>(&'a self) -> &'a str {
		&self.raw[self.sasl_username_s..self.sasl_username_e]
	}
}

impl SmtpdBad {
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

impl MessageParser for Smtpd {
	fn parse(inner: Inner, start: usize) -> Result<Option<Message>, ParseError> {
		match inner.queue_id() {
			None => return Ok(None),
			Some(_) => ()
		};
		let (bad, message_s) = {
			let rest = &inner.raw[start..];
			if rest.starts_with(" discard: ") {
				(Some(SmtpdBadReason::Discard), start + 10)
			} else if rest.starts_with(" reject: ") {
				(Some(SmtpdBadReason::Reject), start + 9)
			} else {
				(None, 0)
			}
		};
		match bad {
			None => (),
			Some(reason) => {
				let (message_e, from_s, from_e, to_s, to_e, proto, helo_s, helo_e) = {
					let rest = &inner.raw[message_s..];
					let pos = match rest.find(';') {
						None => return Err(ParseError::SmtpdBadMessage),
						Some(p) => p
					};
					let rest = &rest[pos..];
					let message_e = message_s + pos;
					if !rest.starts_with("; from=<") {
						return Err(ParseError::SmtpdNoFrom);
					}
					let from_s = message_e + 8;
					let rest = &rest[8..];
					let pos = match rest.find('>') {
						None => return Err(ParseError::SmtpdBadFrom),
						Some(p) => p
					};
					let rest = &rest[pos..];
					let from_e = from_s + pos;
					let (rest, end, to_s, to_e) = {
						if rest.starts_with("> to=<") {
							let rest = &rest[6..];
							let to_s = from_e + 6;
							let pos = match rest.find('>') {
								None => return Err(ParseError::SmtpdBadTo),
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
						return Err(ParseError::SmtpdNoProto);
					}
					let rest = &rest[8..];
					let pos = match rest.find(' ') {
						None => return Err(ParseError::SmtpdBadProto),
						Some(p) => p
					};
					let proto = match &rest[..pos] {
						"SMTP" => SmtpdProto::SMTP,
						"ESMTP" => SmtpdProto::ESMTP,
						_ => return Err(ParseError::SmtpdUnknownProto)
					};
					let rest = &rest[pos..];
					if !rest.starts_with(" helo=<") {
						return Err(ParseError::SmtpdNoHelo);
					}
					let rest = &rest[7..];
					let helo_s = end + 8 + pos + 7;
					let helo_e = match rest.find('>') {
						None => return Err(ParseError::SmtpdBadHelo),
						Some(p) => helo_s + p
					};
					(message_e, from_s, from_e, to_s, to_e, proto, helo_s, helo_e)
				};
				return Ok(Some(Message::SmtpdBad { m: SmtpdBad { inner: inner, reason:reason, message_s:message_s, message_e:message_e, from_s:from_s, from_e:from_e, to_s:to_s, to_e:to_e, proto:proto, helo_s:helo_s, helo_e:helo_e } }));
			}
		};
		let (client_s, client_e, done) = {
			let rest = &inner.raw[start..];
			if  !rest.starts_with(" client=") {
				return Err(ParseError::SmtpdNonEndingOrigQueue);
			}
			let rest = &rest[8..];
			let client_s = start + 8;
			let (done, client_e) =  match rest.find(',') {
				None => (true, client_s + rest.len()),
				Some(p) => (false, client_s + p)
			};
			(client_s, client_e, done)
		};
		if done {
			return Ok(Some(Message::Smtpd { m: Smtpd { inner:inner, client_s: client_s, client_e: client_e } }));
		}
		let (orig_queue_id_s, orig_queue_id_e, orig_client_s, orig_client_e, sasl_username_s, sasl_username_e) = {
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
				(orig_queue_id_s, orig_queue_id_e, orig_client_s, orig_client_e, 0, 0)
			} else if rest.starts_with(", sasl_method=LOGIN, sasl_username=") {
				let sasl_username_s = client_e + 35;
				let sasl_username_e = inner.raw.len();
				(0, 0, 0, 0, sasl_username_s, sasl_username_e)
			} else {
				return Err(ParseError::SmtpdBadOrigQueue);
			}
		};
		let smtpd = Smtpd { inner:inner, client_s: client_s, client_e: client_e };
		if orig_client_e != 0 {
			Ok(Some(Message::SmtpdForward { m: SmtpdForward { smtpd: smtpd, orig_queue_id_s: orig_queue_id_s, orig_queue_id_e: orig_queue_id_e, orig_client_s: orig_client_s, orig_client_e:orig_client_e } }))
		} else {
			Ok(Some(Message::SmtpdLogin { m: SmtpdLogin { smtpd: smtpd, sasl_username_s:sasl_username_s, sasl_username_e:sasl_username_e } }))
		}
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

	fn parse_smtpd(s: String) -> Result<Option<Message>, ParseError> {
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
			Ok(x) => panic!("This should have been ignored ({:?})", x)
		};
		let s = "Aug  4 00:00:12 ozgurluk postfix/smtpd[25688]: warning: hostname pei-190-128-lxiii-xxx.une.net.co does not resolve to address 190.128.63.30: Name or service not known".to_string();
		match parse_smtpd(s) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => (),
			Ok(x) => panic!("This should have been ignored ({:?})", x)
		};
	}

	#[test]
	fn bad_message() {
		let s = "Aug  4 00:00:12 yuuai postfix/smtpd[3199]: 89EF32091D: discard: DATA from scm.seog.co.kr[61.36.79.99]: <DATA>: Data command Recipient list contains a blacklisted address".to_string();
		match parse_smtpd(s) {
			Err(ParseError::SmtpdBadMessage) => (),
			Err(x) => panic!("Wrong error, should have been SmtpdBadMessage {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(_) => panic!("This should have failed")
		};
	}

	#[test]
	fn no_from() {
		let s = "Jul 25 00:00:09 svoboda postfix/smtpd[5884]: 87E611409B022: reject: DATA from 99-46-141-195.lightspeed.sntcca.sbcglobal.net[99.46.141.195]: 421 4.7.1 <DATA>: Data command rejected: Tu (firstname.lastname) as envoye trop de mails recement. Merci de contacter le support s'il s'agit d'une erreur;".to_string();
		match parse_smtpd(s) {
			Err(ParseError::SmtpdNoFrom) => (),
			Err(x) => panic!("Wrong error, should have been SmtpdNoFrom {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(_) => panic!("This should have failed")
		};
	}

	#[test]
	fn bad_from() {
		let s = "Aug  4 00:00:12 yuuai postfix/smtpd[3199]: 89EF32091D: discard: DATA from scm.seog.co.kr[61.36.79.99]: <DATA>: Data command Recipient list contains a blacklisted address; from=<xyz".to_string();
		match parse_smtpd(s) {
			Err(ParseError::SmtpdBadFrom) => (),
			Err(x) => panic!("Wrong error, should have been SmtpdBadFrom {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(_) => panic!("This should have failed")
		};
	}

	#[test]
	fn bad_to() {
		let s = "Jul 25 00:00:09 svoboda postfix/smtpd[5884]: 87E611409B022: reject: DATA from 99-46-141-195.lightspeed.sntcca.sbcglobal.net[99.46.141.195]: 421 4.7.1 <DATA>: Data command rejected: Tu (firstname.lastname) as envoye trop de mails recement. Merci de contacter le support s'il s'agit d'une erreur; from=<firstname.lastname@m4x.org> to=<firstname.lastname@m4x.org".to_string();
		match parse_smtpd(s) {
			Err(ParseError::SmtpdBadTo) => (),
			Err(x) => panic!("Wrong error, should have been SmtpdBadTo {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(_) => panic!("This should have failed")
		};
	}

	#[test]
	fn no_proto() {
		let s = "Aug  4 00:00:12 yuuai postfix/smtpd[3199]: 89EF32091D: discard: DATA from scm.seog.co.kr[61.36.79.99]: <DATA>: Data command Recipient list contains a blacklisted address; from=<massnewsletter4654654xel@gmail.com>".to_string();
		match parse_smtpd(s) {
			Err(ParseError::SmtpdNoProto) => (),
			Err(x) => panic!("Wrong error, should have been SmtpdNoProto {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(_) => panic!("This should have failed")
		};
	}

	#[test]
	fn bad_proto() {
		let s = "Jul 25 00:00:09 svoboda postfix/smtpd[5884]: 87E611409B022: reject: DATA from 99-46-141-195.lightspeed.sntcca.sbcglobal.net[99.46.141.195]: 421 4.7.1 <DATA>: Data command rejected: Tu (firstname.lastname) as envoye trop de mails recement. Merci de contacter le support s'il s'agit d'une erreur; from=<firstname.lastname@m4x.org> to=<firstname.lastname@m4x.org> proto=".to_string();
		match parse_smtpd(s) {
			Err(ParseError::SmtpdBadProto) => (),
			Err(x) => panic!("Wrong error, should have been SmtpdBadProto {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(_) => panic!("This should have failed")
		};
	}

	#[test]
	fn unknown_proto() {
		let s = "Aug  4 00:00:12 yuuai postfix/smtpd[3199]: 89EF32091D: discard: DATA from scm.seog.co.kr[61.36.79.99]: <DATA>: Data command Recipient list contains a blacklisted address; from=<massnewsletter4654654xel@gmail.com> proto=XYZ ".to_string();
		match parse_smtpd(s) {
			Err(ParseError::SmtpdUnknownProto) => (),
			Err(x) => panic!("Wrong error, should have been SmtpdUnknownProto {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(_) => panic!("This should have failed")
		};
	}

	#[test]
	fn no_helo() {
		let s = "Jul 25 00:00:09 svoboda postfix/smtpd[5884]: 87E611409B022: reject: DATA from 99-46-141-195.lightspeed.sntcca.sbcglobal.net[99.46.141.195]: 421 4.7.1 <DATA>: Data command rejected: Tu (firstname.lastname) as envoye trop de mails recement. Merci de contacter le support s'il s'agit d'une erreur; from=<firstname.lastname@m4x.org> to=<firstname.lastname@m4x.org> proto=ESMTP ".to_string();
		match parse_smtpd(s) {
			Err(ParseError::SmtpdNoHelo) => (),
			Err(x) => panic!("Wrong error, should have been SmtpdNoHelo {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(_) => panic!("This should have failed")
		};
	}

	#[test]
	fn bad_helo() {
		let s = "Aug  4 00:00:12 yuuai postfix/smtpd[3199]: 89EF32091D: discard: DATA from scm.seog.co.kr[61.36.79.99]: <DATA>: Data command Recipient list contains a blacklisted address; from=<massnewsletter4654654xel@gmail.com> proto=SMTP helo=<gmail.com".to_string();
		match parse_smtpd(s) {
			Err(ParseError::SmtpdBadHelo) => (),
			Err(x) => panic!("Wrong error, should have been SmtpdBadHelo {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(_) => panic!("This should have failed")
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
	fn valid_discard() {
		let s = "Aug  4 00:00:12 yuuai postfix/smtpd[3199]: 89EF32091D: discard: DATA from scm.seog.co.kr[61.36.79.99]: <DATA>: Data command Recipient list contains a blacklisted address; from=<massnewsletter4654654xel@gmail.com> proto=SMTP helo=<gmail.com>".to_string();
		let smtpd = match parse_smtpd(s) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some(Message::SmtpdBad{m:x})) => x,
			Ok(Some(x)) => panic!("Wrong message parsed: {:?}", x)
		};
		match smtpd.reason {
			SmtpdBadReason::Discard => (),
			x => panic!("Parsed wrong reason: {}", x)
		}
		assert_eq!(smtpd.message(), "DATA from scm.seog.co.kr[61.36.79.99]: <DATA>: Data command Recipient list contains a blacklisted address");
		assert_eq!(smtpd.from(), "massnewsletter4654654xel@gmail.com");
		match smtpd.to() {
			None => (),
			Some(s) => panic!("Parsed a non existing to: {}", s)
		};
		match smtpd.proto {
			SmtpdProto::SMTP => (),
			x => panic!("Parsed wrong proto: {}", x)
		}
		assert_eq!(smtpd.helo(), "gmail.com");
		assert_eq!(fmt::format(format_args!("{:?}", smtpd)), "SmtpdBad { inner: Inner { raw: \"Aug  4 00:00:12 yuuai postfix/smtpd[3199]: 89EF32091D: discard: DATA from scm.seog.co.kr[61.36.79.99]: <DATA>: Data command Recipient list contains a blacklisted address; from=<massnewsletter4654654xel@gmail.com> proto=SMTP helo=<gmail.com>\", host_e: 21, queue_s: 22, queue_e: 29, process: Smtpd, pid: 3199, queue_id_s: 43, queue_id_e: 53 }, reason: Discard, message_s: 64, message_e: 169, from_s: 177, from_e: 211, to_s: 0, to_e: 0, proto: SMTP, helo_s: 230, helo_e: 239 }");
	}

	#[test]
	fn valid_reject() {
		let s = "Jul 25 00:00:09 svoboda postfix/smtpd[5884]: 87E611409B022: reject: DATA from 99-46-141-195.lightspeed.sntcca.sbcglobal.net[99.46.141.195]: 421 4.7.1 <DATA>: Data command rejected: Tu (firstname.lastname) as envoye trop de mails recement. Merci de contacter le support s'il s'agit d'une erreur; from=<firstname.lastname@m4x.org> to=<firstname.lastname@m4x.org> proto=ESMTP helo=<DiskStation>".to_string();
		let smtpd = match parse_smtpd(s) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some(Message::SmtpdBad{m:x})) => x,
			Ok(Some(x)) => panic!("Wrong message parsed: {:?}", x)
		};
		match smtpd.reason {
			SmtpdBadReason::Reject => (),
			x => panic!("Parsed wrong reason: {}", x)
		}
		assert_eq!(smtpd.message(), "DATA from 99-46-141-195.lightspeed.sntcca.sbcglobal.net[99.46.141.195]: 421 4.7.1 <DATA>: Data command rejected: Tu (firstname.lastname) as envoye trop de mails recement. Merci de contacter le support s'il s'agit d'une erreur");
		assert_eq!(smtpd.from(), "firstname.lastname@m4x.org");
		match smtpd.to() {
			None => panic!("Failed to parse to"),
			Some(s) => assert_eq!(s, "firstname.lastname@m4x.org")
		};
		match smtpd.proto {
			SmtpdProto::ESMTP => (),
			x => panic!("Parsed wrong proto: {}", x)
		}
		assert_eq!(smtpd.helo(), "DiskStation");
		assert_eq!(fmt::format(format_args!("{:?}", smtpd)), "SmtpdBad { inner: Inner { raw: \"Jul 25 00:00:09 svoboda postfix/smtpd[5884]: 87E611409B022: reject: DATA from 99-46-141-195.lightspeed.sntcca.sbcglobal.net[99.46.141.195]: 421 4.7.1 <DATA>: Data command rejected: Tu (firstname.lastname) as envoye trop de mails recement. Merci de contacter le support s\\\'il s\\\'agit d\\\'une erreur; from=<firstname.lastname@m4x.org> to=<firstname.lastname@m4x.org> proto=ESMTP helo=<DiskStation>\", host_e: 23, queue_s: 24, queue_e: 31, process: Smtpd, pid: 5884, queue_id_s: 45, queue_id_e: 58 }, reason: Reject, message_s: 68, message_e: 293, from_s: 301, from_e: 327, to_s: 333, to_e: 359, proto: ESMTP, helo_s: 379, helo_e: 390 }");
	}

	#[test]
	fn valid_with_orig() {
		let s = "Aug  4 00:00:08 yuuai postfix/smtpd.local[20039]: 84ED020916: client=localhost[127.0.0.1], orig_queue_id=67D8720887, orig_client=3.mo52.mail-out.ovh.net[178.33.254.192]".to_string();
		let smtpd = match parse_smtpd(s) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some(Message::SmtpdForward{m:x})) => x,
			Ok(Some(x)) => panic!("Wrong message parsed: {:?}", x)
		};
		assert_eq!(smtpd.client(), "localhost[127.0.0.1]");
		assert_eq!(smtpd.orig_queue_id(), "67D8720887");
		assert_eq!(smtpd.orig_client(), "3.mo52.mail-out.ovh.net[178.33.254.192]");
		assert_eq!(fmt::format(format_args!("{:?}", smtpd)), "SmtpdForward { smtpd: Smtpd { inner: Inner { raw: \"Aug  4 00:00:08 yuuai postfix/smtpd.local[20039]: 84ED020916: client=localhost[127.0.0.1], orig_queue_id=67D8720887, orig_client=3.mo52.mail-out.ovh.net[178.33.254.192]\", host_e: 21, queue_s: 22, queue_e: 29, process: Smtpd, pid: 20039, queue_id_s: 50, queue_id_e: 60 }, client_s: 69, client_e: 89 }, orig_queue_id_s: 105, orig_queue_id_e: 115, orig_client_s: 129, orig_client_e: 168 }");
	}

	#[test]
	fn valid_simple() {
		let s = "Aug  4 00:00:08 yuuai postfix/smtpd.local[20039]: 84ED020916: client=3.mo52.mail-out.ovh.net[178.33.254.192]".to_string();
		let smtpd = match parse_smtpd(s) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some(Message::Smtpd{m:x})) => x,
			Ok(Some(x)) => panic!("Wrong message parsed: {:?}", x)
		};
		assert_eq!(smtpd.client(), "3.mo52.mail-out.ovh.net[178.33.254.192]");
		assert_eq!(fmt::format(format_args!("{:?}", smtpd)), "Smtpd { inner: Inner { raw: \"Aug  4 00:00:08 yuuai postfix/smtpd.local[20039]: 84ED020916: client=3.mo52.mail-out.ovh.net[178.33.254.192]\", host_e: 21, queue_s: 22, queue_e: 29, process: Smtpd, pid: 20039, queue_id_s: 50, queue_id_e: 60 }, client_s: 69, client_e: 108 }");
	}

	#[test]
	fn valid_with_login() {
		let s = "Jul 25 00:00:09 svoboda postfix/smtpd[5884]: 87E611409B022: client=99-46-141-195.lightspeed.sntcca.sbcglobal.net[99.46.141.195], sasl_method=LOGIN, sasl_username=firstname.lastname".to_string();
		let smtpd = match parse_smtpd(s) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some(Message::SmtpdLogin{m:x})) => x,
			Ok(Some(x)) => panic!("Wrong message parsed: {:?}", x)
		};
		assert_eq!(smtpd.client(), "99-46-141-195.lightspeed.sntcca.sbcglobal.net[99.46.141.195]");
		assert_eq!(smtpd.sasl_username(), "firstname.lastname");
		assert_eq!(fmt::format(format_args!("{:?}", smtpd)), "SmtpdLogin { smtpd: Smtpd { inner: Inner { raw: \"Jul 25 00:00:09 svoboda postfix/smtpd[5884]: 87E611409B022: client=99-46-141-195.lightspeed.sntcca.sbcglobal.net[99.46.141.195], sasl_method=LOGIN, sasl_username=firstname.lastname\", host_e: 23, queue_s: 24, queue_e: 31, process: Smtpd, pid: 5884, queue_id_s: 45, queue_id_e: 58 }, client_s: 67, client_e: 127 }, sasl_username_s: 162, sasl_username_e: 180 }");
	}
}
