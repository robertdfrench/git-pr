#!/bin/sh
#
# A git implementation that always fails.
case $1 in

	"--version")
		echo "failing_git version 1"
		exit 1
		;;

	*)
		exit 1
		;;
esac
