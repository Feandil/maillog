use super::super::ParserConfig;
use super::super::ParseError;

pub const DATE_LEN : usize = 15;

#[derive(Debug, PartialEq)]
pub enum Process {
	Cleanup,
	Local,
	Pickup,
	Pipe,
	Smtp,
}

#[derive(Debug)]
pub struct Inner {
	pub raw: String,
	host_e: usize,
	queue_s: usize,
	queue_e: usize,
	pub process: Process,
	pub pid: u32,
	queue_id_s: usize,
	queue_id_e: usize
}

impl Inner {
	pub fn date<'a>(&'a self) -> &'a str {
		&self.raw[..DATE_LEN]
	}

	pub fn host<'a>(&'a self) -> &'a str {
		&self.raw[DATE_LEN+1..self.host_e]
	}

	pub fn queue<'a>(&'a self) -> &'a str {
		&self.raw[self.queue_s..self.queue_e]
	}

	pub fn queue_id<'a>(&'a self) -> Option<&'a str> {
		if self.queue_id_e != 0 {
			Some(&self.raw[self.queue_id_s..self.queue_id_e])
		} else {
			None
		}
	}

	pub fn parse(config: &ParserConfig, s: String) -> Result<Option<(Inner, usize)>, ParseError> {
		let length = s.len();
		if length < DATE_LEN + 2 {
			return Err(ParseError::DateTooShort);
		}
		let (host_e, queue_s, queue_e, process, pid,
		     queue_id_s, queue_id_e) = {
			let rest = &s[DATE_LEN+1..];
			let (host_e, rest) = match rest.find(' ') {
				None => return Err(ParseError::NonEndingHost),
				Some(pos) => (DATE_LEN + 1 + pos, &rest[pos+1..])
			};
			let queue_s = host_e + 1;
			for prog in config.process_noise.iter() {
				if rest.starts_with(prog) {
					return Ok(None);
				}
			}
			let pos = match rest.find(':') {
				None => return Err(ParseError::MissingProcess),
				Some(pos) => pos
			};
			let (queue_e, rest) = match rest.find('/') {
				None => return Err(ParseError::NonEndingQueue),
				Some(pos) => (queue_s + pos, &rest[pos+1..])
			};
			let process_len = match rest.find('[') {
				None => return Err(ParseError::NonEndingProcess),
				Some(len) => len
			};
			let process = match &rest[..process_len] {
				"cleanup" => Process::Cleanup,
				"local" => Process::Local,
				"pickup" => Process::Pickup,
				"pipe" => Process::Pipe,
				"smtp" => Process::Smtp,
				_ => return Err(ParseError::UnknownProcess),
			};
			let rest = &rest[process_len+1..];
			let process_end = queue_e + 1 + process_len;
			let pid_e = pos - (process_end - queue_s) - 2;
			if !rest[pid_e..].starts_with("]: ") {
				return Err(ParseError::BadProcessID)
			}
			let pid = match rest[..pid_e].parse::<u32>() {
				Err(_) => return Err(ParseError::BadProcessID),
				Ok(val) => val
			};
			let queue_id_s = process_end + 1 + pid_e + 3;
			let rest = &rest[pid_e + 3..];
			let queue_id_e = match rest.find(':') {
				None => 0,
				Some(pos) => {
					let len = pos;
					if rest[..len].bytes().any(|b| ('0' as u8 > b || b > '9' as u8) && ('A' as u8 > b || b > 'F' as u8)) {
						0
					} else {
						queue_id_s + len
					}
				}
			};
			(host_e, queue_s, queue_e, process, pid,
			 queue_id_s, queue_id_e)
		};
		Ok(Some((Inner {raw: s, host_e: host_e, queue_s: queue_s,
		                queue_e: queue_e, process: process, pid: pid,
		                queue_id_s:queue_id_s, queue_id_e: queue_id_e},
		         queue_id_e + 1)))
	}
}

#[cfg(test)]
mod tests {
	use std::fmt;
	use super::*;
	use super::super::super::ParserConfig;
	use super::super::super::ParseError;

	fn init() -> Inner {
		Inner {
			raw: "Sep  3 00:00:03 yuuai postfix-in/cleanup[31247]: 12C172090B:".to_string(),
			host_e: 21,
			queue_s: 22,
			queue_e: 32,
			process: Process::Cleanup,
			pid: 31247,
			queue_id_s: 49,
			queue_id_e: 59
		}
	}

	#[test]
	fn date() {
		let i = init();
		assert_eq!(i.date(), "Sep  3 00:00:03");
	}

	#[test]
	fn host() {
		let i = init();
		assert_eq!(i.host(), "yuuai");
	}

	#[test]
	fn queue() {
		let i = init();
		assert_eq!(i.queue(), "postfix-in");
	}

	#[test]
	fn pid() {
		let i = init();
		assert_eq!(i.pid, 31247);
	}

	#[test]
	fn queue_id() {
		let i = init();
		match i.queue_id() {
			None => panic!("Failed to match the queue id"),
			Some(s) => assert_eq!(s, "12C172090B")
		};
	}

	fn conf() -> ParserConfig {
		ParserConfig { process_noise: vec!["clamsmtpd".to_string()] }
	}

	#[test]
	fn dates_too_short() {
		match Inner::parse(&conf(), "".to_string()) {
			Err(ParseError::DateTooShort) => (),
			Err(x) => panic!("Wrong Error (should have been DateTooShort): {}", x),
			_ => panic!("Should have failed")
		}
		match Inner::parse(&conf(), "Sep  3 00:00:03 ".to_string()) {
			Err(ParseError::DateTooShort) => (),
			Err(x) => panic!("Wrong Error (should have been DateTooShort): {}", x),
			_ => panic!("Should have failed")
		}
	}

	#[test]
	fn non_ending_host() {
		match Inner::parse(&conf(), "Sep  3 00:00:03 yuuai".to_string()) {
			Err(ParseError::NonEndingHost) => (),
			Err(x) => panic!("Wrong Error (should have been NonEndingHost): {}", x),
			_ => panic!("Should have failed")
		}
	}

	#[test]
	fn missing_process() {
		match Inner::parse(&conf(), "Sep  3 00:00:03 yuuai ".to_string()) {
			Err(ParseError::MissingProcess) => (),
			Err(x) => panic!("Wrong Error (should have been MissingProcess): {}", x),
			_ => panic!("Should have failed")
		}
		match Inner::parse(&conf(), "Sep  3 00:00:03 yuuai postfix".to_string()) {
			Err(ParseError::MissingProcess) => (),
			Err(x) => panic!("Wrong Error (should have been MissingProcess): {}", x),
			_ => panic!("Should have failed")
		}
	}

	#[test]
	fn process_noise(){
		match Inner::parse(&conf(), "Sep  3 00:00:03 yuuai clamsmtpd:".to_string()) {
			Ok(None) => (),
			Err(x) => panic!("Wrong Error (Should have been ignored): {}", x),
			_ => panic!("Should have been ignored")
		}
	}
	#[test]
	fn non_ending_queue(){
		match Inner::parse(&conf(), "Sep  3 00:00:03 yuuai postfix-in:".to_string()) {
			Err(ParseError::NonEndingQueue) => (),
			Err(x) => panic!("Wrong Error (should have been NonEndingQueue): {}", x),
			_ => panic!("Should have failed")
		}
	}

	#[test]
	fn non_ending_process(){
		match Inner::parse(&conf(), "Sep  3 00:00:03 yuuai postfix-in/cleanup:".to_string()) {
			Err(ParseError::NonEndingProcess) => (),
			Err(x) => panic!("Wrong Error (should have been NonEndingProcess): {}", x),
			_ => panic!("Should have failed")
		}
	}

	#[test]
	fn unknown_process() {
		match Inner::parse(&conf(), "Sep  3 00:00:03 yuuai postfix-in/xxx[:".to_string()) {
			Err(ParseError::UnknownProcess) => (),
			Err(x) => panic!("Wrong Error (should have been UnknownProcess): {}", x),
			_ => panic!("Should have failed")
		}
	}

	#[test]
	fn bad_pid(){
		match Inner::parse(&conf(), "Sep  3 00:00:03 yuuai postfix-in/cleanup[31247:".to_string()) {
			Err(ParseError::BadProcessID) => (),
			Err(x) => panic!("Wrong Error (should have been BadProcessID): {}", x),
			_ => panic!("Should have failed")
		}
		match Inner::parse(&conf(), "Sep  3 00:00:03 yuuai postfix-in/cleanup[31247]:".to_string()) {
			Err(ParseError::BadProcessID) => (),
			Err(x) => panic!("Wrong Error (should have been BadProcessID): {}", x),
			_ => panic!("Should have failed")
		}
		match Inner::parse(&conf(), "Sep  3 00:00:03 yuuai postfix-in/cleanup[abcd]: ".to_string()) {
			Err(ParseError::BadProcessID) => (),
			Err(x) => panic!("Wrong Error (should have been BadProcessID): {}", x),
			_ => panic!("Should have failed")
		}
	}

	#[test]
	fn no_queue_id(){
		let (inner, _) = match Inner::parse(&conf(), "Sep  3 00:00:03 yuuai postfix-in/cleanup[31247]: ".to_string()) {
			Err(x) => panic!("Failed to parse: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some(inner)) => inner
		};
		match inner.queue_id() {
			None => (),
			Some(s) => panic!("Found inexistant queue ID: {}", s)
		};
		let (inner, _) = match Inner::parse(&conf(), "Sep  3 00:00:03 yuuai postfix-in/cleanup[31247]: NOQUEUE:".to_string()) {
			Err(x) => panic!("Failed to parse: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some(inner)) => inner
		};
		match inner.queue_id() {
			None => (),
			Some(s) => panic!("Found inexistant queue ID: {}", s)
		};
	}

	#[test]
	fn malformed_queue_id(){
		let inner = match Inner::parse(&conf(), "Sep  3 00:00:03 yuuai postfix-in/cleanup[31247]: 12C172090BZ:".to_string()) {
			Err(x) => panic!("Failed to parse: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some((inner,_))) => inner
		};
		match inner.queue_id() {
			None => (),
			Some(s) => panic!("Found inexistant queue ID: {}", s)
		};
	}

	#[test]
	fn compare() {
		let expected = init();
		let (parsed, end) = match Inner::parse(&conf(), "Sep  3 00:00:03 yuuai postfix-in/cleanup[31247]: 12C172090B:".to_string()){
			Err(x) => panic!("Failed to parse: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some(inner)) => inner
		};
		assert_eq!(expected.raw, parsed.raw);
		assert_eq!(expected.host_e, parsed.host_e);
		assert_eq!(expected.queue_s, parsed.queue_s);
		assert_eq!(expected.queue_e, parsed.queue_e);
		assert_eq!(expected.process, parsed.process);
		assert_eq!(expected.pid, parsed.pid);
		assert_eq!(expected.queue_id_s, parsed.queue_id_s);
		assert_eq!(expected.queue_id_e, parsed.queue_id_e);
		assert_eq!(end, 60);
		assert_eq!(fmt::format(format_args!("{:?}", parsed)), "Inner { raw: \"Sep  3 00:00:03 yuuai postfix-in/cleanup[31247]: 12C172090B:\", host_e: 21, queue_s: 22, queue_e: 32, process: Cleanup, pid: 31247, queue_id_s: 49, queue_id_e: 59 }");
	}

	#[test]
	fn ignore() {
		match Inner::parse(&conf(), "Sep  3 00:00:03 yuuai clamsmtpd:".to_string()) {
			Ok(None) => (),
			Err(x) => panic!("Wrong Error (Should have been ignored): {}", x),
			_ => panic!("Should have been ignored")
		}
	}						
}
