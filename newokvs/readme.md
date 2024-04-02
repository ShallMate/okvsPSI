# OKVS 

How to use
    ```
    cd newokvs
    cargo build -r --example perf
    cd target/release/examples/
    ./perf -n 1048576 -w 448 -e 0.01
    # -n how many, default 1048576
    # -e epsilon, default 0.01 for OKVS
    # -w width, default 448
    ```

