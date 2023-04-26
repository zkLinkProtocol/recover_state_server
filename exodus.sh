#!/bin/bash

if [ ! -d "log" ]; then
  mkdir log
fi

if [ "$1" == "recover" ]; then
  nohup ./target/release/recover_state --genesis >> log/recover_state.log 2>&1 &
elif [ "$1" == "continue" ]; then
  nohup ./target/release/recover_state --continue >> log/recover_state.log 2>&1 &
elif [ "$1" == "server" ]; then
  nohup ./target/release/exodus_server >> log/server.log 2>&1 &
elif [ "$1" == "prover" ]; then
  nohup ./target/release/exodus_prover tasks -w 4 >> log/prover.log 2>&1 &
elif [ "$1" == "stop" ]; then
  pkill -f exodus_server
  pkill -f exodus_prover
elif [ "$1" == "clean" ]; then
  cd storage
  diesel database reset
else
  echo "Usage: $0 {recover|continue|server|prover|stop}"
  exit 1
fi

exit 0
