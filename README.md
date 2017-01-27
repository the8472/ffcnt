# ffcnt 

Fast file counting for spinning rust, in rust.

ffcnt's sole purpose is to provide a faster alternative to `find /some/path/ -type f | wc -l`.
It achieves that by looking up the extent map *of directories* and reordering recursion into the directory tree by the physical offset of the first extent of each directory.
This greatly reduces disk seeks.


## Requirements

* Linux
* A filesystem that supports the `fiemap` ioctl on directories. Otherwise no speedup over `find` will be achieved, but counting will still work. Tested with Ext4, others may work too (Feedback welcome!)

You can test support the `filefrag` tool.  

```
## supported

$ filefrag /tmp/
/tmp/: 3 extents found


## unsupported

$ filefrag /mnt/test/
/mnt/test/: FIBMAP unsupported
```

## Binary

You can find prebuilt x86_64-linux-glibc binaries without debug information under [releases](../../releases).
For troubleshooting and other environments you'll have to build your own.

## Build

* clone repo
* install rust nightly and cargo
* `cargo build --release` 


## Unscientific Benchmark

Idle system:

```
# echo 3 > /proc/sys/vm/drop_caches ; time ffcnt .
196608

real	0m23.889s
user	0m1.233s
sys	0m2.127s

# echo 3 > /proc/sys/vm/drop_caches ; time find . -type f | wc -l
196608

real	2m31.562s
user	0m0.557s
sys	0m3.860s
```

Busy system with mixed read/write workload. Differences in file counts arose due to writes happening in the meantime:

```
# echo 3 > /proc/sys/vm/drop_caches ; time ffcnt . 
4411262

real	10m36.288s
user	0m3.656s
sys	0m7.588s

# echo 3 > /proc/sys/vm/drop_caches ; time find . -type f | wc -l
4412101

real	45m54.955s
user	0m3.212s
sys	0m12.044s
```

Both tests were performed on HDDs and the files were spread over 65536 directories with a nesting depth of 2, i.e. a branching factor of 256.


## Ideas

* 1 thread per block device in tree
* filter by name
* optimized du