#!/bin/bash

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd )"

cd $DIR

if [ ! -d "log" ]; then
  mkdir log
fi

if [ "$PORT" = "" ]; then
    export PORT=8081
fi

if [ "$1" == "start" ]; then
  rm log/*

  cd storage
  diesel database reset
  cd ..

  echo "Start recovering state"
  nohup ./run_recover.sh >> log/run_recover.log 2>&1 &

elif [ "$1" == "server" ]; then
  cargo build --release
  nohup ./target/release/exodus_server >> log/server.log 2>&1 &
elif [ "$1" == "prover" ]; then
  cargo build --release
  # Please refer to prover [README.md](prover/README.md) for detailed command details
  nohup ./target/release/exodus_prover tasks -w 1 >> log/prover.log 2>&1 &
elif [ "$1" == "stop" ]; then
  if [ -f script.pid ]; then
    kill "$(cat script.pid)" 2>/dev/null
    rm script.pid
  fi

  pkill -f recover_state
  pkill -f exodus_server
  pkill -f exodus_prover

  cd exodus-interface
  npm run stop
elif [ "$1" == "clean" ]; then
  cd storage
  diesel database reset
else
  echo "Usage: $0 {start|continue|server|prover|stop|clean}"
  exit 1
fi

exit 0
