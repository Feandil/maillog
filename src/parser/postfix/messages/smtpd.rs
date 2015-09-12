use std::fmt;
use std::ops::Deref;
use super::super::ParseError;
use super::Inner;
use super::Message;
use super::MessageParser;
use super::Reject;
use super::RejectReason;

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

pub enum SmtpdMethod {
	Plain,
	Login,
}

#[derive(Debug)]
pub struct SmtpdLogin {
	smtpd: Smtpd,
	method: SmtpdMethod,
	sasl_username_s: usize,
	sasl_username_e: usize,
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

impl fmt::Display for SmtpdMethod {
	fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		let error = match self {
			&SmtpdMethod::Login => "LOGIN",
			&SmtpdMethod::Plain => "PLAIN",
		};
		write!(fmt, "{}", error)
	}
}

impl fmt::Debug for SmtpdMethod {
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

impl MessageParser for Smtpd {
	fn parse(inner: Inner, start: usize) -> Result<Option<Message>, ParseError> {
		match inner.queue_id() {
			None => return Ok(None),
			Some(_) => ()
		};
		let bad = {
			let rest = &inner.raw[start..];
			if rest.starts_with(" discard: ") {
				Some(RejectReason::Discard)
			} else if rest.starts_with(" reject: ") {
				Some(RejectReason::Reject)
			} else if rest.starts_with(" warn: ") {
				Some(RejectReason::Warn)
			} else {
				None
			}
		};
		match bad {
			None => (),
			Some(reason) =>	return Reject::parse(inner, start, reason),
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
		let (orig_queue_id_s, orig_queue_id_e, orig_client_s, orig_client_e, sasl_username_s, sasl_username_e, method) = {
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
				(orig_queue_id_s, orig_queue_id_e, orig_client_s, orig_client_e, 0, 0, None)
			} else if rest.starts_with(", sasl_method=") {
				let method_s = client_e + 14;
				let rest = &rest[14..];
				let method_len = match rest.find(',') {
					None => return Err(ParseError::SmtpdNonEndingMethod),
					Some(l) => l
				};
				let method = match &rest[..method_len] {
					"LOGIN" => SmtpdMethod::Login,
					"PLAIN" => SmtpdMethod::Plain,
					_ => return Err(ParseError::SmtpdUnknownMethod)
				};
				let rest = &rest[method_len..];
				if !rest.starts_with(", sasl_username=") {
					return Err(ParseError::SmtpdNoUsername);
				}
				let sasl_username_s = method_s + method_len + 16;
				let sasl_username_e = inner.raw.len();
				(0, 0, 0, 0, sasl_username_s, sasl_username_e, Some(method))
			} else {
				return Err(ParseError::SmtpdUnknownFormat);
			}
		};
		let smtpd = Smtpd { inner:inner, client_s: client_s, client_e: client_e };
		match method {
			None => Ok(Some(Message::SmtpdForward { m: SmtpdForward { smtpd: smtpd, orig_queue_id_s: orig_queue_id_s, orig_queue_id_e: orig_queue_id_e, orig_client_s: orig_client_s, orig_client_e:orig_client_e } })),
			Some(method) => Ok(Some(Message::SmtpdLogin { m: SmtpdLogin { smtpd: smtpd, method: method, sasl_username_s:sasl_username_s, sasl_username_e:sasl_username_e } }))
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
	use super::super::RejectReason;
	use super::super::RejectProto;
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
	fn bad_orig_queue() {
		let s ="Aug  4 00:00:08 yuuai postfix/smtpd.local[20039]: 84ED020916: client=localhost[127.0.0.1], orig_queue_id".to_string();
		match parse_smtpd(s) {
			Err(ParseError::SmtpdUnknownFormat) => (),
			Err(x) => panic!("Wrong error, should have been SmtpdUnknownFormat {}", x),
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
			Ok(Some(Message::Reject{m:x})) => x,
			Ok(Some(x)) => panic!("Wrong message parsed: {:?}", x)
		};
		match smtpd.reason {
			RejectReason::Discard => (),
			x => panic!("Parsed wrong reason: {}", x)
		}
		assert_eq!(smtpd.message(), "DATA from scm.seog.co.kr[61.36.79.99]: <DATA>: Data command Recipient list contains a blacklisted address");
		assert_eq!(smtpd.from(), "massnewsletter4654654xel@gmail.com");
		match smtpd.to() {
			None => (),
			Some(s) => panic!("Parsed a non existing to: {}", s)
		};
		match smtpd.proto {
			RejectProto::SMTP => (),
			x => panic!("Parsed wrong proto: {}", x)
		}
		assert_eq!(smtpd.helo(), "gmail.com");
		match smtpd.explanation() {
			None => (),
			Some(s) => panic!("Parsed a non existing explanation: {}", s)
		}
		assert_eq!(fmt::format(format_args!("{:?}", smtpd)), "Reject { inner: Inner { raw: \"Aug  4 00:00:12 yuuai postfix/smtpd[3199]: 89EF32091D: discard: DATA from scm.seog.co.kr[61.36.79.99]: <DATA>: Data command Recipient list contains a blacklisted address; from=<massnewsletter4654654xel@gmail.com> proto=SMTP helo=<gmail.com>\", host_e: 21, queue_s: 22, queue_e: 29, process: Smtpd, pid: 3199, queue_id_s: 43, queue_id_e: 53 }, reason: Discard, message_s: 64, message_e: 169, from_s: 177, from_e: 211, to_s: 0, to_e: 0, proto: SMTP, helo_s: 230, helo_e: 239, explanation_s: 0, explanation_e: 0 }");
	}

	#[test]
	fn valid_reject() {
		let s = "Jul 25 00:00:09 svoboda postfix/smtpd[5884]: 87E611409B022: reject: DATA from 99-46-141-195.lightspeed.sntcca.sbcglobal.net[99.46.141.195]: 421 4.7.1 <DATA>: Data command rejected: Tu (firstname.lastname) as envoye trop de mails recement. Merci de contacter le support s'il s'agit d'une erreur; from=<firstname.lastname@m4x.org> to=<firstname.lastname@m4x.org> proto=ESMTP helo=<DiskStation>".to_string();
		let smtpd = match parse_smtpd(s) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some(Message::Reject{m:x})) => x,
			Ok(Some(x)) => panic!("Wrong message parsed: {:?}", x)
		};
		match smtpd.reason {
			RejectReason::Reject => (),
			x => panic!("Parsed wrong reason: {}", x)
		}
		assert_eq!(smtpd.message(), "DATA from 99-46-141-195.lightspeed.sntcca.sbcglobal.net[99.46.141.195]: 421 4.7.1 <DATA>: Data command rejected: Tu (firstname.lastname) as envoye trop de mails recement. Merci de contacter le support s'il s'agit d'une erreur");
		assert_eq!(smtpd.from(), "firstname.lastname@m4x.org");
		match smtpd.to() {
			None => panic!("Failed to parse to"),
			Some(s) => assert_eq!(s, "firstname.lastname@m4x.org")
		};
		match smtpd.proto {
			RejectProto::ESMTP => (),
			x => panic!("Parsed wrong proto: {}", x)
		}
		assert_eq!(smtpd.helo(), "DiskStation");
		match smtpd.explanation() {
			None => (),
			Some(s) => panic!("Parsed a non existing explanation: {}", s)
		}
		assert_eq!(fmt::format(format_args!("{:?}", smtpd)), "Reject { inner: Inner { raw: \"Jul 25 00:00:09 svoboda postfix/smtpd[5884]: 87E611409B022: reject: DATA from 99-46-141-195.lightspeed.sntcca.sbcglobal.net[99.46.141.195]: 421 4.7.1 <DATA>: Data command rejected: Tu (firstname.lastname) as envoye trop de mails recement. Merci de contacter le support s\\\'il s\\\'agit d\\\'une erreur; from=<firstname.lastname@m4x.org> to=<firstname.lastname@m4x.org> proto=ESMTP helo=<DiskStation>\", host_e: 23, queue_s: 24, queue_e: 31, process: Smtpd, pid: 5884, queue_id_s: 45, queue_id_e: 58 }, reason: Reject, message_s: 68, message_e: 293, from_s: 301, from_e: 327, to_s: 333, to_e: 359, proto: ESMTP, helo_s: 379, helo_e: 390, explanation_s: 0, explanation_e: 0 }");
	}

	#[test]
	fn valid_warn() {
		let s = "Aug  4 00:49:53 yuuai postfix/smtpd[30778]: 0D71E208B6: warn: RCPT from unknown[190.62.150.179]: Literal IP in HELO hostnames not allowed here, please check your configuration; from=<firstname.lastname@m4x.org> to=<firstname.lastname@m4x.org> proto=ESMTP helo=<[127.0.0.2]>".to_string();
		let smtpd = match parse_smtpd(s) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some(Message::Reject{m:x})) => x,
			Ok(Some(x)) => panic!("Wrong message parsed: {:?}", x)
		};
		match smtpd.reason {
			RejectReason::Warn => (),
			x => panic!("Parsed wrong reason: {}", x)
		}
		assert_eq!(smtpd.message(), "RCPT from unknown[190.62.150.179]: Literal IP in HELO hostnames not allowed here, please check your configuration");
		assert_eq!(smtpd.from(), "firstname.lastname@m4x.org");
		match smtpd.to() {
			None => panic!("Failed to parse to"),
			Some(s) => assert_eq!(s, "firstname.lastname@m4x.org")
		};
		match smtpd.proto {
			RejectProto::ESMTP => (),
			x => panic!("Parsed wrong proto: {}", x)
		}
		assert_eq!(smtpd.helo(), "[127.0.0.2]");
		match smtpd.explanation() {
			None => (),
			Some(s) => panic!("Parsed a non existing explanation: {}", s)
		}
		assert_eq!(fmt::format(format_args!("{:?}", smtpd)), "Reject { inner: Inner { raw: \"Aug  4 00:49:53 yuuai postfix/smtpd[30778]: 0D71E208B6: warn: RCPT from unknown[190.62.150.179]: Literal IP in HELO hostnames not allowed here, please check your configuration; from=<firstname.lastname@m4x.org> to=<firstname.lastname@m4x.org> proto=ESMTP helo=<[127.0.0.2]>\", host_e: 21, queue_s: 22, queue_e: 29, process: Smtpd, pid: 30778, queue_id_s: 44, queue_id_e: 54 }, reason: Warn, message_s: 62, message_e: 175, from_s: 183, from_e: 209, to_s: 215, to_e: 241, proto: ESMTP, helo_s: 261, helo_e: 272, explanation_s: 0, explanation_e: 0 }");
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
		assert_eq!(fmt::format(format_args!("{:?}", smtpd)), "SmtpdLogin { smtpd: Smtpd { inner: Inner { raw: \"Jul 25 00:00:09 svoboda postfix/smtpd[5884]: 87E611409B022: client=99-46-141-195.lightspeed.sntcca.sbcglobal.net[99.46.141.195], sasl_method=LOGIN, sasl_username=firstname.lastname\", host_e: 23, queue_s: 24, queue_e: 31, process: Smtpd, pid: 5884, queue_id_s: 45, queue_id_e: 58 }, client_s: 67, client_e: 127 }, method: LOGIN, sasl_username_s: 162, sasl_username_e: 180 }");
	}
}
