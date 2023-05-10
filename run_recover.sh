  if [ -f script.pid ]; then
    echo "recover already exist."
    exit 0;
  fi
  
  echo $$ > script.pid

  ./target/release/recover_state --genesis >> log/recover_state.log 2>&1 &
  recover_state_pid=$!

  cd exodus-interface
  npm install
  npm run build
  npm run serve
  cd ..

  wait $recover_state_pid
  rm script.pid

  nohup ./target/release/exodus_server >> log/server.log 2>&1 &
  nohup ./target/release/exodus_prover tasks -w 1 >> log/prover.log 2>&1 &