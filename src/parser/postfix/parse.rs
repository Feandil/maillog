use super::ParserConfig;
use super::ParseError;
use super::messages::*;

pub fn parse_line(raw: String, conf: &ParserConfig) -> Result<Option<Message>, ParseError> {
	let (inner, start) = match Inner::parse(conf, raw) {
		Err(error) => return Err(error),
		Ok(None) => return Ok(None),
		Ok(Some((x,y))) => (x,y)
	};
	match inner.process {
		Process::Anvil => Ok(None),
		Process::Pickup => Pickup::parse(inner, start),
		Process::Pipe => Forward::parse(inner, start),
		Process::Smtp => Forward::parse(inner, start),
		Process::Local => Forward::parse(inner, start),
		Process::Smtpd => Smtpd::parse(inner, start),
		Process::Cleanup => Cleanup::parse(inner, start),
		Process::Qmgr => Qmgr::parse(inner, start),
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use super::super::ParserConfig;
	use super::super::ParseError;
	use super::super::messages::*;

	fn parse(s: String) -> Result<Option<Message>, ParseError> {
		let conf = ParserConfig { process_noise: vec!["clamsmtpd".to_string()] };
		parse_line(s, &conf)
	}

	#[test]
	fn ignore() {
		match parse("Sep  3 00:00:03 yuuai clamsmtpd:".to_string()) {
			Ok(None) => (),
			Err(x) => panic!("Wrong Error (Should have been ignored): {}", x),
			_ => panic!("Should have been ignored")
		}
	}
}

