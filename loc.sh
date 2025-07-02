#!/bin/sh
# reminder that Dash and MKSH are faster than Bash
set -eu #-o pipefail

loc() {
	grep -rIv '^\s*$' -- "$1" | wc -l
}

if [ $# -gt 0 ]; then
	for p in "$@"; do
		echo "$(loc "$p") $p"
	done
else
	for p in .[!.]* ./* ; do
		# emulate `nullglob`
		[ -e "$p" ] && echo "$(loc "$p") $p"
	done | sort -rn
fi
