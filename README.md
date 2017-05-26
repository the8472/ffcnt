[![Version](https://img.shields.io/crates/v/ffcnt.svg)](https://crates.io/crates/ffcnt)

# ffcnt 

Fast file counting and listing for spinning rust, in rust.

ffcnt's purpose is to provide a faster alternatives to some common filesystem operations as a frontend for the [platter-walk](https://github.com/the8472/platter-walk) crate.


* `ffcnt --type f` replaces `find -type f | wc -l`
* `ffcnt --type f --ls --leaf-order content` replaces `find -type f` and returns files in optimized order for reading 
* `ffcnt -s` replaces `du -s --apparent-size`



## Requirements

* Linux
* A filesystem that supports the `fiemap` ioctl on directories.<br>
Currently **ext4** is known to provide that. If you know other ones, please report!<br>
Incompatible filesystems will work but gain no speedup over `find`.


You can test filesystem support with the `filefrag` tool.  

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
* install liblzo2 and libz (build-time dependencies) 
* install rust and cargo
* `cargo build --release`

## Usage

```
    ffcnt [FLAGS] [OPTIONS] [dirs]...

FLAGS:
    -h, --help       Prints help information
        --ls         list files
    -s               sum apparent length of matched files. Implies --leaf-order inode.
    -V, --version    Prints version information

OPTIONS:
        --leaf-order <ord>    optimize order for listing/stat/reads [values: inode, content, dentry]
        --type <type>         filter type [values: f, l, d, s, b, c, p]

ARGS:
    <dirs>...    directories to traverse [default: cwd]
```

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
