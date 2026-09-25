#!/bin/sh
# Set the tablet clock from an HTTP Date header.
#
# The reMarkable 2's clock battery can be dead, so after a reboot the time is
# whatever it was when the tablet was last used, and the time-sync service on
# this firmware never reports network connectivity. A wrong clock makes every
# TLS certificate look "not yet valid", so the coach could not reach the API.
# Plain HTTP works regardless of the clock, and every web server sends the date.
# Runs at service start (ExecStartPre) and, if the clock still looks wrong,
# before each request (see coordinator.rs). BusyBox sh only.

for host in http://1.1.1.1/ http://neverssl.com/ http://www.google.com/; do
  hdr=$(wget -S -T 5 -O /dev/null "$host" 2>&1 | grep -i '^ *Date:' | head -n 1)
  [ -n "$hdr" ] && break
done
[ -n "$hdr" ] || { echo "fix-clock: no Date header (offline?)"; exit 1; }

# "  Date: Fri, 25 Sep 2026 14:38:03 GMT"
set -- $hdr
day=$3; mon=$4; year=$5; time=$6
case "$mon" in
  Jan) m=01;; Feb) m=02;; Mar) m=03;; Apr) m=04;; May) m=05;; Jun) m=06;;
  Jul) m=07;; Aug) m=08;; Sep) m=09;; Oct) m=10;; Nov) m=11;; Dec) m=12;;
  *) echo "fix-clock: unexpected month '$mon' in '$hdr'"; exit 1;;
esac

date -u -s "$year-$m-$day $time" >/dev/null || { echo "fix-clock: date -s failed"; exit 1; }
echo "fix-clock: clock set to $(date -u)"
