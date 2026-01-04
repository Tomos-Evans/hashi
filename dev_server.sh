#!/bin/bash


CMD="trunk serve -a 0.0.0.0 --tls-key-path ./dev_certs/localhost.key --tls-cert-path ./dev_certs/localhost.crt --public-url /hashi/"
echo "Starting dev server:"
echo $CMD
$CMD