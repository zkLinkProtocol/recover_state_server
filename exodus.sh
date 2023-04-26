#!/bin/bash

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd )"

cd $DIR
cargo build --release

if [ ! -d "log" ]; then
  mkdir log
fi

if [ "$1" == "start" ]; then
  cd storage
  diesel database reset
  cd ..
  nohup ./target/release/recover_state --genesis >> log/recover_state.log 2>&1 &
  echo "start recover state"
  nohup ./target/release/exodus_server >> log/server.log 2>&1 &
  echo "start exodus server"
  nohup ./target/release/exodus_prover tasks -w 4 >> log/prover.log 2>&1 &
  echo "start exodus prover"
  cd exodus-interface

elif [ "$1" == "continue" ]; then
  # If there is an interruption, you can run the `continue` command
  nohup ./target/release/recover_state --continue >> log/recover_state.log 2>&1 &
elif [ "$1" == "server" ]; then
  nohup ./target/release/exodus_server >> log/server.log 2>&1 &
elif [ "$1" == "prover" ]; then
  # Please refer to prover [README.md](prover/README.md) for detailed command details
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
