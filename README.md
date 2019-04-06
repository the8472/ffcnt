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
fast file counting 0.3.0

USAGE:
    ffcnt [FLAGS] [OPTIONS] [dirs]...

FLAGS:
    -h, --help        Prints help information
        --ls          list files
        --prefetch    attempt to prefetch directory indices from underlying mount device. requires read permission on device
    -s                sum apparent length of matched files. Only counts hardlinked files once. Does not follow symlinks. Implies --leaf-order inode.
    -V, --version     Prints version information

OPTIONS:
        --leaf-order <ord>    optimize order for listing/stat/reads [values: inode, content, dentry]
        --type <type>         filter type [values: f, l, d, s, b, c, p]

ARGS:
    <dirs>...    directories to traverse [default: cwd]
```

## Unscientific Benchmark

Idle system:

```
$ echo 3 > /proc/sys/vm/drop_caches ; time find /tmp/foo/ -type f | wc -l
826536

real	0m52.289s
user	0m0.680s
sys	0m4.361s

$ echo 3 > /proc/sys/vm/drop_caches ; time ffcnt /tmp/foo/ --type f
files: 826536

real	0m17.072s
user	0m1.230s
sys	0m2.190s

$ echo 3 > /proc/sys/vm/drop_caches ; time sudo ffcnt /tmp/foo/ --prefetch --type f
files: 826536

real	0m13.311s
user	0m2.029s
sys	0m1.440s
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

Both tests were performed on HDDs with a directory structure of at least 2 nesting levels and a branching factor of 256 




## Ideas

* 1 thread per block device in tree
