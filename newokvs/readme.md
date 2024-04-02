# OKVS 

### How to use
```
cd newokvs
cargo build -r --example perf
cd target/release/examples/
./perf -n 1048576 -w 448 -e 0.01
-n how many, default 1048576
-e epsilon, default 0.01 for OKVS
-w width, default 448
```



### Build
okvsPSI
```
git clone https://github.com/ShallMate/okvsPSI
cd okvsPSI
python3 build.py -DVOLE_PSI_ENABLE_BOOST=ON

cd newokvs
cargo build -r --example perf
```
### Installing

```
python3 build.py --install
```
or 
```
python3 build.py --install=install/prefix/path
```

### Use
```
cd out/build/linux/okvspsi/
./okvspsi -nnr 20 -nns 20 -v -m  
-nnr the log_2^n of the input size of receiver
-nns the log_2^n of the input size of sender
-v print run time
-m malicious security
-nt threadnum

cd target/release/examples/
./perf -n 1048576 -w 448 -e 0.01
-n how many, default 1048576
-e epsilon, default 0.01 for OKVS
-w width, default 448
```

### Okvs Result example
![OKVS结果](./okvs_result.png)

