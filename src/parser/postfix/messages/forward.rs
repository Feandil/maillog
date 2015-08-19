use std::ops::Deref;
use super::super::ParseError;
use super::Inner;
use super::Message;
use super::MessageParser;

#[derive(Debug)]
pub struct Forward {
	inner: Inner,
	to_s: usize,
	to_e: usize,
	orig_to_s: usize,
	orig_to_e: usize,
	relay_s: usize,
	relay_e: usize,
// We ignore the delays
	dsn: [u8; 3],
	status_s: usize,
	status_e: usize,
	child_queue_id_s: usize,	
	child_queue_id_e: usize,
}

#[derive(Debug)]
pub struct ForwardError {
	inner: Inner,
	host_s: usize,
	host_e: usize,
	message_s: usize,
	message_e: usize,
}

impl Deref for Forward {
	type Target = Inner;
	fn deref(&self) -> &Inner {
		&self.inner
	}
}

impl Deref for ForwardError {
	type Target = Inner;
	fn deref(&self) -> &Inner {
		&self.inner
	}
}

impl Forward {
	pub fn to <'a>(&'a self) -> &'a str {
		&self.raw[self.to_s..self.to_e]
	}

	pub fn orig_to<'a>(&'a self) -> Option<&'a str> {
		match self.orig_to_e {
			0 => None,
			_ => Some(&self.raw[self.orig_to_s..self.orig_to_e])
		}
	}

	pub fn relay <'a>(&'a self) -> &'a str {
		&self.raw[self.relay_s..self.relay_e]
	}

	pub fn status <'a>(&'a self) -> &'a str {
		&self.raw[self.status_s..self.status_e]
	}

	pub fn child_queue<'a>(&'a self) -> Option<&'a str> {
		match self.child_queue_id_e {
			0 => None,
			_ => Some(&self.raw[self.child_queue_id_s..self.child_queue_id_e])
		}
	}
}

impl ForwardError {
	pub fn host <'a>(&'a self) -> &'a str {
		&self.raw[self.host_s..self.host_e]
	}

	pub fn message <'a>(&'a self) -> &'a str {
		&self.raw[self.message_s..self.message_e]
	}
}

impl MessageParser for Forward {
	fn parse(inner: Inner, start: usize) -> Result<Option<Message>, ParseError> {
		{
			match inner.queue_id() {
				None => return Ok(None),
				Some(_) => ()
			};
		}
		let (host_s, host_e, message_s, message_e) = {
			let rest = &inner.raw[start..];
			let error = rest.starts_with(" host ");
			if error {
				let rest = &rest[6..];
				let host_s = start + 6;
				let len = match rest.find(' ') {
					None => return Err(ParseError::ForwardBadHost),
					Some(p) => p
				};
				let rest = &rest[len..];
				let host_e = host_s + len;
				if !rest.starts_with(" said: ") {
					return Err(ParseError::ForwardNoMessage);
				}
				let rest = &rest[7..];
				let message_s = host_e + 7;
				let message_e = message_s + rest.len();
				(host_s, host_e, message_s, message_e)
			} else {
				(0, 0, 0, 0)
			}
		};
		if message_e != 0 {
			return Ok(Some(Message::ForwardError { m: ForwardError { inner: inner, host_s:host_s, host_e:host_e, message_s:message_s, message_e: message_e } }));
		}			
		let (to_s, to_e, orig_to_s, orig_to_e, relay_s, relay_e, dsn, status_s, status_e, child_queue_id_s, child_queue_id_e) = {
			let rest = &inner.raw[start..];
			let (rest, start, to_s, to_e) = {
				if !rest.starts_with(" to=<") {
					return Err(ParseError::ForwardNoTo);
				}
				let rest = &rest[5..];
				let start = start + 5;
				let len = match rest.find('>') {
					None => return Err(ParseError::ForwardBadTo),
					Some(l) => l
				};
				(&rest[len+1..], start + len + 1, start, start + len)
			};
			let (rest, start, orig_to_s, orig_to_e) = {
				if rest.starts_with(", orig_to=<") {
					let rest = &rest[11..];
					let start = start + 11;
					let len = match rest.find('>') {
						None => return Err(ParseError::ForwardBadOrigTo),
						Some(l) => l
					};
					(&rest[len+1..], start + len + 1, start, start + len)
				} else {
					(rest, start, 0, 0)
				}
			};
			let (rest, start, relay_s, relay_e) = {
				if !rest.starts_with(", relay=") {
					return Err(ParseError::ForwardNoRelay);
				}
				let rest = &rest[8..];
				let start = start + 8;
				let len = match rest.find(',') {
					None => return Err(ParseError::ForwardBadRelay),
					Some(l) => l
				};
				let relay = &rest[..len];
				let ip_s = match relay.find('[') {
					None => 0,
					Some(p) => p,
				};
				let ip_e = match relay[ip_s..].find(']') {
					None => 0,
					Some(p) => p
				};
				if ip_s != 0 && ip_e != 0 {
					(&rest[len..], start + len, start + ip_s + 1, start + ip_s + ip_e)
				} else {
					(&rest[len..], start + len, start, start + len)
				}
			};
			let mut pos = 0;
			if rest.starts_with(", conn_use=") {
				pos = match rest[1..].find(',') {
					None => return Err(ParseError::ForwardBadConn),
					Some(p) => 1 + p
				};
			};
			pos = match rest[1..].find(',') {
				None => return Err(ParseError::ForwardNoDelay),
				Some(p) => pos + 1 + p
			};
			pos = match rest[pos+1..].find(',') {
				None => return Err(ParseError::ForwardNoDelays),
				Some(p) => pos + 1 + p
			};
			let rest = &rest[pos..];
			let start = start + pos;
			let (rest, start, dsn) = {
				if !rest.starts_with(", dsn=") {
					return Err(ParseError::ForwardNoDSN);
				}
				let rest = &rest[6..];
				let start = start + 6;
				let len = match rest.find(',') {
					None => return Err(ParseError::ForwardBadDSN),
					Some(l) => l
					};
				let raw_dsn = &rest[..len].split('.').collect::<Vec<&str>>();
				if raw_dsn.len() != 3 {
					return Err(ParseError::ForwardDSNBadLen);
				}
				let mut dsn = [0u8; 3];
				for (i, x) in raw_dsn.iter().enumerate() {
					dsn[i] = match x.parse::<u8>() {
						Err(_) => return Err(ParseError::PickupDSNNotInt),
						Ok(val) => val
					}
				};
				(&rest[len..], start + len, [dsn[0], dsn[1], dsn[2]])
			};
			let (status_s, status_e, child_queue_id_s, child_queue_id_e) = {
				if !rest.starts_with(", status=") {
					return Err(ParseError::ForwardNoStatus);
				}
				let status = &rest[9..];
				let start = start + 9;
				let len = status.len();
				if status.starts_with("sent (250 2.0.0 Ok: queued as ") && &status[len-1..len] == ")" {
					(start, start + len, start + 30, start + len - 1)
				} else {
					(start, start + len, 0, 0)
				}
			};
			(to_s, to_e, orig_to_s, orig_to_e, relay_s, relay_e, dsn, status_s, status_e, child_queue_id_s, child_queue_id_e)
		};
		Ok(Some(Message::Forward { m: Forward { inner: inner, to_s:to_s, to_e:to_e, orig_to_s:orig_to_s, orig_to_e:orig_to_e, relay_s:relay_s, relay_e:relay_e, dsn:dsn, status_s:status_s, status_e:status_e, child_queue_id_s:child_queue_id_s, child_queue_id_e:child_queue_id_e } }))
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

	fn parse_forward(s: String) -> Result<Option<Message>, ParseError> {
		let conf = ParserConfig { process_noise: vec!["clamsmtpd".to_string()] };
		let (inner, start) = match Inner::parse(&conf, s) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some((x,y))) => (x,y)
		};
		Forward::parse(inner, start)
	}

	#[test]
	fn broken_host() {
		match parse_forward("Aug  4 00:01:08 yuuai postfix/smtp[10627]: C217620B0B: host ".to_string()) {
			Err(ParseError::ForwardBadHost) => (),
			Err(x) => panic!("Wrong Error (should have been ForwardBadHost): {}", x),
			_ => panic!("Should have failed")
		}
	}

	#[test]
	fn no_message() {
		match parse_forward("Aug  4 00:01:08 yuuai postfix/smtp[10627]: C217620B0B: host gmail-smtp-in.l.google.com[64.233.167.26] ".to_string()) {
			Err(ParseError::ForwardNoMessage) => (),
			Err(x) => panic!("Wrong Error (should have been ForwardNoMessage): {}", x),
			_ => panic!("Should have failed")
		}
	}

	#[test]
	fn broken_to() {
		match parse_forward("Jul 25 00:00:01 yuuai postfix/smtp[3703]: 0345620AE4:".to_string()) {
			Err(ParseError::ForwardNoTo) => (),
			Err(x) => panic!("Wrong Error (should have been ForwardNoTo): {}", x),
			_ => panic!("Should have failed")
		}
		match parse_forward("Jul 25 00:00:01 yuuai postfix/smtp[3703]: 0345620AE4: to=".to_string()) {		
			Err(ParseError::ForwardNoTo) => (),
			Err(x) => panic!("Wrong Error (should have been ForwardNoTo): {}", x),
			_ => panic!("Should have failed")
		}
		match parse_forward("Jul 25 00:00:01 yuuai postfix/smtp[3703]: 0345620AE4: to=<".to_string()) {
			Err(ParseError::ForwardBadTo) => (),
			Err(x) => panic!("Wrong Error (should have been ForwardBadTo): {}", x),
			_ => panic!("Should have failed")
		}
	}

	#[test]
	fn broken_orig_to() {
		match parse_forward("Jul 25 00:00:01 yuuai postfix/smtp[3703]: 0345620AE4: to=<xxxx@melix.net>, orig_to=<".to_string()) {
			Err(ParseError::ForwardBadOrigTo) => (),
			Err(x) => panic!("Wrong Error (should have been ForwardBadOrigTo): {}", x),
			_ => panic!("Should have failed")
		}
	}

	#[test]
	fn broken_relay() {
		match parse_forward("Jul 25 00:00:01 yuuai postfix/smtp[3703]: 0345620AE4: to=<>".to_string()) {
			Err(ParseError::ForwardNoRelay) => (),
			Err(x) => panic!("Wrong Error (should have been ForwardNoRelay): {}", x),
			_ => panic!("Should have failed")
		}
		match parse_forward("Jul 25 00:00:01 yuuai postfix/smtp[3703]: 0345620AE4: to=<>, orig_to=<>".to_string()) {
			Err(ParseError::ForwardNoRelay) => (),
			Err(x) => panic!("Wrong Error (should have been ForwardNoRelay): {}", x),
			_ => panic!("Should have failed")
		}
		match parse_forward("Jul 25 00:00:01 yuuai postfix/smtp[3703]: 0345620AE4: to=<>, orig_to=<>, relay=".to_string()) {
			Err(ParseError::ForwardBadRelay) => (),
			Err(x) => panic!("Wrong Error (should have been ForwardBadRelay): {}", x),
			_ => panic!("Should have failed")
		}
	}

	#[test]
	fn broken_conn_use() {
		match parse_forward("Jul 25 00:00:01 yuuai postfix/smtp[3703]: 0345620AE4: to=<>, orig_to=<>, relay=, conn_use=2".to_string()) {
			Err(ParseError::ForwardBadConn) => (),
			Err(x) => panic!("Wrong Error (should have been ForwardBadConn): {}", x),
			_ => panic!("Should have failed")
		}
	}

	#[test]
	fn broken_delays() {
		match parse_forward("Jul 25 00:00:01 yuuai postfix/smtp[3703]: 0345620AE4: to=<>, orig_to=<>, relay=,".to_string()) {
			Err(ParseError::ForwardNoDelay) => (),
			Err(x) => panic!("Wrong Error (should have been ForwardNoDelay): {}", x),
			_ => panic!("Should have failed")
		}
		match parse_forward("Jul 25 00:00:01 yuuai postfix/smtp[3703]: 0345620AE4: to=<>, orig_to=<>, relay=,,".to_string()) {
			Err(ParseError::ForwardNoDelays) => (),
			Err(x) => panic!("Wrong Error (should have been ForwardNoDelays): {}", x),
			_ => panic!("Should have failed")
		}
	}

	#[test]
	fn broken_dsn() {
		match parse_forward("Jul 25 00:00:01 yuuai postfix/smtp[3703]: 0345620AE4: to=<>, orig_to=<>, relay=,,,".to_string()) {
			Err(ParseError::ForwardNoDSN) => (),
			Err(x) => panic!("Wrong Error (should have been ForwardNoDSN): {}", x),
			_ => panic!("Should have failed")
		}
		match parse_forward("Jul 25 00:00:01 yuuai postfix/smtp[3703]: 0345620AE4: to=<>, orig_to=<>, relay=,,, dsn=".to_string()) {
			Err(ParseError::ForwardBadDSN) => (),
			Err(x) => panic!("Wrong Error (should have been ForwardBadDSN): {}", x),
			_ => panic!("Should have failed")
		}
		match parse_forward("Jul 25 00:00:01 yuuai postfix/smtp[3703]: 0345620AE4: to=<>, orig_to=<>, relay=,,, dsn=,".to_string()) {
			Err(ParseError::ForwardDSNBadLen) => (),
			Err(x) => panic!("Wrong Error (should have been ForwardDSNBadLen): {}", x),
			_ => panic!("Should have failed")
		}
		match parse_forward("Jul 25 00:00:01 yuuai postfix/smtp[3703]: 0345620AE4: to=<>, orig_to=<>, relay=,,, dsn=1.2.3.4,".to_string()) {
			Err(ParseError::ForwardDSNBadLen) => (),
			Err(x) => panic!("Wrong Error (should have been ForwardDSNBadLen): {}", x),
			_ => panic!("Should have failed")
		}
		match parse_forward("Jul 25 00:00:01 yuuai postfix/smtp[3703]: 0345620AE4: to=<>, orig_to=<>, relay=,,, dsn=x.y.z,".to_string()) {
			Err(ParseError::PickupDSNNotInt) => (),
			Err(x) => panic!("Wrong Error (should have been PickupDSNNotInt): {}", x),
			_ => panic!("Should have failed")
		}
		match parse_forward("Jul 25 00:00:01 yuuai postfix/smtp[3703]: 0345620AE4: to=<>, orig_to=<>, relay=,,, dsn=1024.0.1,".to_string()) {
			Err(ParseError::PickupDSNNotInt) => (),
			Err(x) => panic!("Wrong Error (should have been PickupDSNNotInt): {}", x),
			_ => panic!("Should have failed")
		}
	}

	#[test]
	fn broken_status() {
		match parse_forward("Jul 25 00:00:01 yuuai postfix/smtp[3703]: 0345620AE4: to=<>, orig_to=<>, relay=,,, dsn=0.0.0,".to_string()) {
			Err(ParseError::ForwardNoStatus) => (),
			Err(x) => panic!("Wrong Error (should have been ForwardNoStatus): {}", x),
			_ => panic!("Should have failed")
		}
	}

	#[test]
	fn no_queue_id() {
		match parse_forward("Jul 25 00:00:01 yuuai postfix/smtp[8311]: connect to gmail-smtp-in.l.google.com[2a00:1450:400c:c02::1a]:25: Network is unreachable".to_string()) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => (),
			Ok(Some(_)) => panic!("This should have been ignored")
		}
	}

	#[test]
	fn valid_error() {
		let s = "Aug  4 00:01:08 yuuai postfix/smtp[10627]: C217620B0B: host gmail-smtp-in.l.google.com[64.233.167.26] said: 421-4.7.0 [129.104.30.35      15] Our system has detected an unusual rate of 421-4.7.0 unsolicited mail originating from your IP address. To protect our 421-4.7.0 users from spam, mail sent from your IP address has been temporarily 421-4.7.0 rate limited. Please visit 421-4.7.0  https://support.google.com/mail/answer/81126 to review our Bulk Email 421 4.7.0 Senders Guidelines. md4si16637671wic.106 - gsmtp (in reply to end of DATA command)".to_string();
		let forward = match parse_forward(s) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some(Message::ForwardError{m:x})) => x,
			Ok(Some(x)) => panic!("Wrong message parsed: {:?}", x)
		};
		assert_eq!(forward.host(), "gmail-smtp-in.l.google.com[64.233.167.26]");
		assert_eq!(forward.message(), "421-4.7.0 [129.104.30.35      15] Our system has detected an unusual rate of 421-4.7.0 unsolicited mail originating from your IP address. To protect our 421-4.7.0 users from spam, mail sent from your IP address has been temporarily 421-4.7.0 rate limited. Please visit 421-4.7.0  https://support.google.com/mail/answer/81126 to review our Bulk Email 421 4.7.0 Senders Guidelines. md4si16637671wic.106 - gsmtp (in reply to end of DATA command)");
		assert_eq!(fmt::format(format_args!("{:?}", forward)), "ForwardError { inner: Inner { raw: \"Aug  4 00:01:08 yuuai postfix/smtp[10627]: C217620B0B: host gmail-smtp-in.l.google.com[64.233.167.26] said: 421-4.7.0 [129.104.30.35      15] Our system has detected an unusual rate of 421-4.7.0 unsolicited mail originating from your IP address. To protect our 421-4.7.0 users from spam, mail sent from your IP address has been temporarily 421-4.7.0 rate limited. Please visit 421-4.7.0  https://support.google.com/mail/answer/81126 to review our Bulk Email 421 4.7.0 Senders Guidelines. md4si16637671wic.106 - gsmtp (in reply to end of DATA command)\", host_e: 21, queue_s: 22, queue_e: 29, process: Smtp, pid: 10627, queue_id_s: 43, queue_id_e: 53 }, host_s: 60, host_e: 101, message_s: 108, message_e: 550 }");
	}

	#[test]
	fn valid() {
		let s = "Jul 25 00:00:01 yuuai postfix/smtp[3703]: 0345620AE4: to=<xxxx@melix.net>, orig_to=<yyy@melix.net>, relay=127.0.0.1[127.0.0.1]:10024, conn_use=2, delay=0.57, delays=0.4/0/0.04/0.13, dsn=2.0.0, status=sent (250 2.0.0 Ok: queued as 60F6120AF9)".to_string();
		let forward = match parse_forward(s) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some(Message::Forward{m:x})) => x,
			Ok(Some(x)) => panic!("Wrong message parsed: {:?}", x)
		};
		assert_eq!(forward.to(), "xxxx@melix.net");
		assert_eq!(forward.orig_to(), Some("yyy@melix.net"));
		assert_eq!(forward.relay(), "127.0.0.1");
		assert_eq!(forward.dsn, [2, 0, 0]);
		assert_eq!(forward.status(), "sent (250 2.0.0 Ok: queued as 60F6120AF9)");
		assert_eq!(forward.child_queue(), Some("60F6120AF9"));
		assert_eq!(fmt::format(format_args!("{:?}", forward)), "Forward { inner: Inner { raw: \"Jul 25 00:00:01 yuuai postfix/smtp[3703]: 0345620AE4: to=<xxxx@melix.net>, orig_to=<yyy@melix.net>, relay=127.0.0.1[127.0.0.1]:10024, conn_use=2, delay=0.57, delays=0.4/0/0.04/0.13, dsn=2.0.0, status=sent (250 2.0.0 Ok: queued as 60F6120AF9)\", host_e: 21, queue_s: 22, queue_e: 29, process: Smtp, pid: 3703, queue_id_s: 42, queue_id_e: 52 }, to_s: 58, to_e: 72, orig_to_s: 84, orig_to_e: 97, relay_s: 116, relay_e: 125, dsn: [2, 0, 0], status_s: 200, status_e: 241, child_queue_id_s: 230, child_queue_id_e: 240 }");
		let s = "Jul 25 00:00:01 yuuai postfix/smtp[3703]: 0345620AE4: to=<xxxx@melix.net>, relay=bogofilter, delay=0.57, delays=0.4/0/0.04/0.13, dsn=2.0.0, status=sent (delivered via bogofilter service)".to_string();
		let forward = match parse_forward(s) {
			Err(x) => panic!("Parser Error: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some(Message::Forward{m:x})) => x,
			Ok(Some(x)) => panic!("Wrong message parsed: {:?}", x)
		};
		assert_eq!(forward.to(), "xxxx@melix.net");
		assert_eq!(forward.orig_to(), None);
		assert_eq!(forward.relay(), "bogofilter");
		assert_eq!(forward.dsn, [2, 0, 0]);
		assert_eq!(forward.status(), "sent (delivered via bogofilter service)");
		assert_eq!(forward.child_queue(), None);
		assert_eq!(fmt::format(format_args!("{:?}", forward)), "Forward { inner: Inner { raw: \"Jul 25 00:00:01 yuuai postfix/smtp[3703]: 0345620AE4: to=<xxxx@melix.net>, relay=bogofilter, delay=0.57, delays=0.4/0/0.04/0.13, dsn=2.0.0, status=sent (delivered via bogofilter service)\", host_e: 21, queue_s: 22, queue_e: 29, process: Smtp, pid: 3703, queue_id_s: 42, queue_id_e: 52 }, to_s: 58, to_e: 72, orig_to_s: 0, orig_to_e: 0, relay_s: 81, relay_e: 91, dsn: [2, 0, 0], status_s: 147, status_e: 186, child_queue_id_s: 0, child_queue_id_e: 0 }");
	}
}
