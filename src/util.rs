#[must_use]
pub fn loc_counter(s: &str) -> usize {
	let mut acc: usize = 0;
	let mut is_relevant = false;

	// written as explicit state-machine,
	// to guarantee single-pass processing
	for c in s.chars() {
		if is_relevant {
			/*
			Only Unix line-terminators are supported.
			Mac and Windows can GTFO.
			Incidentally, this does work with CRLF and LFCR,
			but not with lone CR (`'\r'`)
			*/
			if c == '\n' {
				#[cfg(not(debug_assertions))]
				{
					/* SAFETY:
					`s.len()` and `acc` are both `usize`s that count bytes,
					this is a conditional increment,
					therefore `acc <= s.len`,
					thus there is no overflow.
					Even if `acc` counted `chars` rather than bytes,
					this would be "more true",
					as `s.chars().count() <= s.len()` for all `str`s.
					*/
					acc = unsafe { acc.unchecked_add(1) };
				};
				#[cfg(debug_assertions)]
				{
					// in release-mode,
					// the optimizer doesn't eliminate the `panic`.
					// That's why I put this in a CFG
					acc = acc.checked_add(1).unwrap_or_else(|| unreachable!());
				};

				// WARN: remember to reset to default!
				is_relevant = false;
			}
		} else {
			/*
			I'm aware that `'\n'` is a Unicode space,
			so this check "overlaps" with the other branch.
			However, since this line is "irrelevant",
			the '\n' can be ignored.
			*/
			if c.is_whitespace() {
				continue;
			}
			is_relevant = true;
		}
	}
	// SAFETY: see the other SAFETY comment
	#[cfg(not(debug_assertions))]
	unsafe {
		std::hint::assert_unchecked(acc <= s.len() / 2)
	};
	debug_assert!(acc <= s.len() / 2);
	acc
}

#[cfg(test)]
mod tests {
	use super::*;

	#[must_use]
	fn loc_counter_i(s: &str) -> usize {
		//s.split('\n') // delimiter
		s.lines() // terminator
			//.filter(|line| line.chars().any(|c| !c.is_whitespace()))
			.filter(|line| !line.chars().all(char::is_whitespace))
			.count()
	}

	#[test]
	fn lnc() {
		for (s, c) in [
			("a\nb", 1),
			("a\nb\n", 2),
			("a\nb\n\n\n", 2),
			("\n  \n \na\nb\n \n  \n", 2),
		] {
			assert_eq!(loc_counter(s), c);
			assert_eq!(loc_counter_i(s), c);
		}
	}
}
