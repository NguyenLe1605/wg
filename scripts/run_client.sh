#!/bin/bash

sudo setcap cap_net_admin=eip target/release/wg
target/release/wg --peer 172.18.0.2:19988 &
pid=$!

sudo ip addr add 10.8.0.3/24 dev tun0
sudo ip link set up dev tun0
sudo ip link set dev tun0 mtu 1400

trap "kill $pid" INT TERM
wait $pid
