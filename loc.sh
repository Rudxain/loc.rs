#!/bin/sh
# reminder that Dash and MKSH are faster than Bash
set -eu
IFS='
'

impl() {
	# GNU grep, not POSIX
	grep -rIhc '[^[:space:]]' -- "$1" | awk '{s += $1} END {print s+0}'
	# ASK: faster than `awk`?
	# `{ tr '\n' '+'; printf 0; } | bc`.
	# `awk` is binary fixed-precision floating-point;
	# `bc` is decimal arbitrary-precision fixed-point;
	# `bc` impl might do small-int optimization, no guarantee;
	# `bc` pipe+exec overhead is constant,
	# so parsing+adding is dominant
}

if [ $# -gt 0 ]; then
	for p in "$@"; do
		echo "$(impl "$p") $p"
	done
else
	for p in .[!.]* ./* ; do
		# emulate `nullglob`
		[ -e "$p" ] && echo "$(impl "$p") $p"
	done | sort -rn
fi
