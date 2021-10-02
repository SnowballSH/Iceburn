ENV["RUSTFLAGS"] = "-C target-cpu=native"
run(`cargo build --release`)
if length(ARGS) > 0 && ARGS[1] == "run"
    run(`cargo run --release`)
end
ENV["RUSTFLAGS"] = ""
