use super::super::config::ParserConfig;
use super::super::errors::ParseError;

pub const DATE_LEN : usize = 15;

#[derive(Debug)]
pub struct Inner {
	pub raw: String,
	host_e: usize,
	queue_s: usize,
	queue_e: usize,
	process_s: usize,
	process_e: usize,
	pid: u32,
	queue_id_s: usize,
	queue_id_e: usize
}

pub trait PostfixMessage {
	fn date<'a>(&'a self) -> &'a str;
	fn host<'a>(&'a self) -> &'a str;
	fn queue<'a>(&'a self) -> &'a str;
	fn process<'a>(&'a self) -> &'a str;
	fn queue_id<'a>(&'a self) -> &'a str;
}

impl PostfixMessage for Inner {
	fn date<'a>(&'a self) -> &'a str {
		&self.raw[..DATE_LEN]
	}

	fn host<'a>(&'a self) -> &'a str {
		&self.raw[DATE_LEN+1..self.host_e]
	}

	fn queue<'a>(&'a self) -> &'a str {
		&self.raw[self.queue_s..self.queue_e]
	}

	fn process<'a>(&'a self) -> &'a str {
		&self.raw[self.process_s..self.process_e]
	}

	fn queue_id<'a>(&'a self) -> &'a str {
		&self.raw[self.queue_id_s..self.queue_id_e]
	}
}

impl Inner {
	pub fn parse(config: ParserConfig, s: String) -> Result<Option<(Inner, usize)>, ParseError> {
		let length = s.len();
		if length < DATE_LEN + 2 {
			return Err(ParseError::DateTooShort);
		}
		let (host_e, queue_s, queue_e, process_s, process_e, pid,
		     queue_id_s, queue_id_e) = {
			let rest = &s[DATE_LEN+1..];
			let (host_e, rest) = match rest.find(' ') {
				None => return Err(ParseError::NonEndingHost),
				Some(pos) => (DATE_LEN + 1 + pos, &rest[pos+1..])
			};
			let queue_s = host_e + 1;
			let pos = match rest.find(':') {
				None => return Err(ParseError::MissingProcess),
				Some(pos) => pos
			};
			for prog in config.process_noise.iter() {
				if prog == &rest[..pos] {
					return Ok(None);
				}
			}
			let (queue_e, rest) = match rest.find('/') {
				None => return Err(ParseError::NonEndingQueue),
				Some(pos) => (queue_s + pos, &rest[pos+1..])
			};
			let process_s = queue_e + 1;
			let (process_e, rest) = match rest.find('[') {
				None => return Err(ParseError::NonEndingProcess),
				Some(pos) => (process_s + pos, &rest[pos+1..])
			};
			let pid_e = pos - (process_e - queue_s) - 2;
			if !rest[pid_e..].starts_with("]: ") {
				return Err(ParseError::BadProcessID)
			}
			let pid = match rest[..pid_e].parse::<u32>() {
				Err(_) => return Err(ParseError::BadProcessID),
				Ok(val) => val
			};
			let queue_id_s = process_e + 1 + pid_e + 3;
			let rest = &rest[pid_e + 3..];
			let queue_id_e = match rest.find(':') {
				None => return Err(ParseError::NonEndingQueueID),
				Some(pos) => queue_id_s + pos
			};
			(host_e, queue_s, queue_e, process_s, process_e, pid,
			 queue_id_s, queue_id_e)
		};
		Ok(Some((Inner {raw: s, host_e: host_e, queue_s: queue_s,
		                queue_e: queue_e, process_s: process_s,
		                process_e: process_e, pid: pid,
		                queue_id_s:queue_id_s, queue_id_e: queue_id_e},
		         queue_id_e + 1)))
	}
}

#[cfg(test)]
mod tests {
	use std::fmt;
	use super::*;
	use super::super::super::config::ParserConfig;
	use super::super::super::errors::ParseError;

	fn init() -> Inner {
		Inner {
			raw: "Sep  3 00:00:03 yuuai postfix-in/cleanup[31247]: 12C172090B:".to_string(),
			host_e: 21,
			queue_s: 22,
			queue_e: 32,
			process_s: 33,
			process_e: 40,
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
	fn process() {
		let i = init();
		assert_eq!(i.process(), "cleanup");
	}

	#[test]
	fn pid() {
		let i = init();
		assert_eq!(i.pid, 31247);
	}

	#[test]
	fn queue_id() {
		let i = init();
		assert_eq!(i.queue_id(), "12C172090B");
	}

	fn conf() -> ParserConfig {
		ParserConfig { process_noise: vec!["clamsmtpd".to_string()] }
	}

	#[test]
	fn dates_too_short() {
		match Inner::parse(conf(), "".to_string()) {
			Err(ParseError::DateTooShort) => (),
			Err(x) => panic!("Wrong Error (should have been DateTooShort): {}", x),
			_ => panic!("Should have failed")
		}
		match Inner::parse(conf(), "Sep  3 00:00:03 ".to_string()) {
			Err(ParseError::DateTooShort) => (),
			Err(x) => panic!("Wrong Error (should have been DateTooShort): {}", x),
			_ => panic!("Should have failed")
		}
	}

	#[test]
	fn non_ending_host() {
		match Inner::parse(conf(), "Sep  3 00:00:03 yuuai".to_string()) {
			Err(ParseError::NonEndingHost) => (),
			Err(x) => panic!("Wrong Error (should have been NonEndingHost): {}", x),
			_ => panic!("Should have failed")
		}
	}

	#[test]
	fn missing_process() {
		match Inner::parse(conf(), "Sep  3 00:00:03 yuuai ".to_string()) {
			Err(ParseError::MissingProcess) => (),
			Err(x) => panic!("Wrong Error (should have been MissingProcess): {}", x),
			_ => panic!("Should have failed")
		}
		match Inner::parse(conf(), "Sep  3 00:00:03 yuuai postfix".to_string()) {
			Err(ParseError::MissingProcess) => (),
			Err(x) => panic!("Wrong Error (should have been MissingProcess): {}", x),
			_ => panic!("Should have failed")
		}
	}

	#[test]
	fn process_noise(){
		match Inner::parse(conf(), "Sep  3 00:00:03 yuuai clamsmtpd:".to_string()) {
			Ok(None) => (),
			Err(x) => panic!("Wrong Error (Should have been ignored): {}", x),
			_ => panic!("Should have been ignored")
		}
	}
	#[test]
	fn non_ending_queue(){
		match Inner::parse(conf(), "Sep  3 00:00:03 yuuai postfix-in:".to_string()) {
			Err(ParseError::NonEndingQueue) => (),
			Err(x) => panic!("Wrong Error (should have been NonEndingQueue): {}", x),
			_ => panic!("Should have failed")
		}
	}

	#[test]
	fn non_ending_process(){
		match Inner::parse(conf(), "Sep  3 00:00:03 yuuai postfix-in/cleanup:".to_string()) {
			Err(ParseError::NonEndingProcess) => (),
			Err(x) => panic!("Wrong Error (should have been NonEndingProcess): {}", x),
			_ => panic!("Should have failed")
		}
	}

	#[test]
	fn bad_pid(){
		match Inner::parse(conf(), "Sep  3 00:00:03 yuuai postfix-in/cleanup[31247:".to_string()) {
			Err(ParseError::BadProcessID) => (),
			Err(x) => panic!("Wrong Error (should have been NonEndingQueueID): {}", x),
			_ => panic!("Should have failed")
		}
		match Inner::parse(conf(), "Sep  3 00:00:03 yuuai postfix-in/cleanup[31247]:".to_string()) {
			Err(ParseError::BadProcessID) => (),
			Err(x) => panic!("Wrong Error (should have been NonEndingQueueID): {}", x),
			_ => panic!("Should have failed")
		}
		match Inner::parse(conf(), "Sep  3 00:00:03 yuuai postfix-in/cleanup[abcd]: ".to_string()) {
			Err(ParseError::BadProcessID) => (),
			Err(x) => panic!("Wrong Error (should have been BadProcessID): {}", x),
			_ => panic!("Should have failed")
		}
	}

	#[test]
	fn non_ending_queue_id(){
		match Inner::parse(conf(), "Sep  3 00:00:03 yuuai postfix-in/cleanup[31247]: ".to_string()) {
			Err(ParseError::NonEndingQueueID) => (),
			Err(x) => panic!("Wrong Error (should have been NonEndingQueueID): {}", x),
			_ => panic!("Should have failed")
		}
		match Inner::parse(conf(), "Sep  3 00:00:03 yuuai postfix-in/cleanup[31247]: 12C172090B".to_string()) {
			Err(ParseError::NonEndingQueueID) => (),
			Err(x) => panic!("Wrong Error (should have been NonEndingQueueID): {}", x),
			_ => panic!("Should have failed")
		}
	}

	#[test]
	fn compare() {
		let expected = init();
		let (parsed, end) = match Inner::parse(conf(), "Sep  3 00:00:03 yuuai postfix-in/cleanup[31247]: 12C172090B:".to_string()){
			Err(x) => panic!("Failed to parse: {}", x),
			Ok(None) => panic!("This should not have been ignored"),
			Ok(Some(inner)) => inner
		};
		assert_eq!(expected.raw, parsed.raw);
		assert_eq!(expected.host_e, parsed.host_e);
		assert_eq!(expected.queue_s, parsed.queue_s);
		assert_eq!(expected.queue_e, parsed.queue_e);
		assert_eq!(expected.process_s, parsed.process_s);
		assert_eq!(expected.process_e, parsed.process_e);
		assert_eq!(expected.pid, parsed.pid);
		assert_eq!(expected.queue_id_s, parsed.queue_id_s);
		assert_eq!(expected.queue_id_e, parsed.queue_id_e);
		assert_eq!(end, 60);
		assert_eq!(fmt::format(format_args!("{:?}", parsed)), "Inner { raw: \"Sep  3 00:00:03 yuuai postfix-in/cleanup[31247]: 12C172090B:\", host_e: 21, queue_s: 22, queue_e: 32, process_s: 33, process_e: 40, pid: 31247, queue_id_s: 49, queue_id_e: 59 }");
	}

	#[test]
	fn ignore() {
		match Inner::parse(conf(), "Sep  3 00:00:03 yuuai clamsmtpd:".to_string()) {
			Ok(None) => (),
			Err(x) => panic!("Wrong Error (Should have been ignored): {}", x),
			_ => panic!("Should have been ignored")
		}
	}						
}
