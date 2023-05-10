if [ -f script.pid ]; then
  echo "recover already exist."
  exit 0;
fi

cargo build --release
echo $$ > script.pid
# Never force shut down the recover program, otherwise data needs to be resynchronized.
./target/release/recover_state --$1 >> log/recover_state.log 2>&1 &
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