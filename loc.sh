#!/bin/sh
# reminder that Dash and MKSH are faster than Bash
set -eu
IFS='
'

impl() {
	# GNU grep, not POSIX
	grep -rIv '^[[:space:]]*$' -- "$1" | wc -l
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
