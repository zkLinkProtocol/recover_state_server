#!/bin/bash

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd )"

cd $DIR

if [ ! -d "log" ]; then
  mkdir log
fi

if [ "$PORT" = "" ]; then
    PORT=80
fi
  

if [ "$1" == "start" ]; then
  cd storage
  diesel database reset
  cd ..
  cargo build --release
  nohup ./target/release/recover_state --genesis >> log/recover_state.log 2>&1 &
  echo "start recovering state"
  nohup ./target/release/exodus_server >> log/server.log 2>&1 &
  echo "start exodus server"
  nohup ./target/release/exodus_prover tasks -w 4 >> log/prover.log 2>&1 &
  echo "start exodus prover"
  cd exodus-interface
  npm install
  npm run build:devnet
  npm run serve:devnet
elif [ "$1" == "continue" ]; then
  cargo build --release
  # If there is an interruption, you can run the `continue` command
  nohup ./target/release/recover_state --continue >> log/recover_state.log 2>&1 &
  echo "Continue recovering state"
  nohup ./target/release/exodus_server >> log/server.log 2>&1 &
  echo "Continue exodus server"
  nohup ./target/release/exodus_prover tasks -w 2 >> log/prover.log 2>&1 &
  echo "Continue exodus prover"
  cd exodus-interface
  npm run build:devnet
  npm run serve:devnet
elif [ "$1" == "server" ]; then
  cargo build --release
  nohup ./target/release/exodus_server >> log/server.log 2>&1 &
elif [ "$1" == "prover" ]; then
  cargo build --release
  # Please refer to prover [README.md](prover/README.md) for detailed command details
  nohup ./target/release/exodus_prover tasks -w 2 >> log/prover.log 2>&1 &
elif [ "$1" == "stop" ]; then
  pkill -f recover_state
  pkill -f exodus_server
  pkill -f exodus_prover
  cd exodus-interface
  npm run stop:devnet
elif [ "$1" == "clean" ]; then
  cd storage
  diesel database reset
else
  echo "Usage: $0 {recover|continue|server|prover|stop}"
  exit 1
fi

exit 0
