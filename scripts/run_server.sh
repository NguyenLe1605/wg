#!/bin/bash

setcap 'cap_net_admin=eip'  ./wg

./wg &
pid=$!

ip addr add 10.8.0.2/24 dev tun0
ip link set up dev tun0
ip link set dev tun0 mtu 1400

nc -l 10.8.0.2 8080 &
ncpid=$!

trap "kill $pid $ncpid" INT TERM

wait $pid
wait $ncpid
