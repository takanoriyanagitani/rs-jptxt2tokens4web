#!/bin/sh

npx tsc || exec sh -c '
	echo compile error.
	exit 1
'

npx @biomejs/biome lint test.mjs || exec sh -c '
	echo lint error.
	exit 1
'
