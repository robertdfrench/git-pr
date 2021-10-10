#!/bin/sh
#
# A fake git implementation that we can use for unit tests.
case $1 in

	"--version")
		echo "fake_git version 1"
		;;

	*)
		exit 1
		;;
esac
