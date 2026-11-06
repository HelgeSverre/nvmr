# Profile

## Binaries

```
-rwxr-xr-x 1 root root 8436728 Mar  5  2026 /work/root/bin/fnm
-rwxr-xr-x 1 root root  790808 Nov  6 15:03 /work/root/nvmr/nvmr
-rwxr-xr-x 1 root root 1970584 Nov  6 15:03 /work/root/nvmr/nvmr-fetch
/work/root/nvmr/nvmr: ELF 64-bit LSB executable, ARM aarch64, version 1 (SYSV), statically linked, BuildID[sha1]=ca6df5c363c9f6ddc7e95b6bbb5e92846c7f0dd1, stripped
not a dynamic executable
```

## Processes and syscalls (strace -f, bash)

| case | execve | forks | syscalls |
|---|---:|---:|---:|
| nvm: source + use | 21 | 72 | 15490 |
| nvm: source + ls (102 versions) | 165 | 306 | 454383 |
| nvm: startup with auto-use | 24 | 88 | 18449 |
| nvmr: source + use | 2 | 1 | 5066 |
| nvmr: source + ls (102 versions) | 3 | 2 | 2759 |
| nvmr: startup with auto-use | 2 | 1 | 5075 |
| fnm: startup (env --use-on-cd) | 2 | 1 | 414 |
| fnm: startup + use | 3 | 2 | 608 |
| bare bash | 1 | 0 | 173 |
| mise: startup with activation | 3 | 6 | 2951 |
| mise: ls (102 versions) | 4 | 9 | 6843 |
| proto: startup with activation | 3 | 8 | 3508 |
| proto: startup with shim | 1 | 0 | 174 |

Programs executed by upstream `nvm use`: sed 8, awk 6, grep 4, tr 3, uname 3, tail 2, ls 2, bash 1, find 1, sort 1, npm 1, node 1

## `nvmr __auto use` syscall summary

```
% time     seconds  usecs/call     calls    errors syscall
------ ----------- ----------- --------- --------- ----------------
 45.69    0.000053          53         1           execve
 18.10    0.000021           1        18        14 newfstatat
  9.48    0.000011           1        10         6 openat
  6.90    0.000008           0         9           mmap
  5.17    0.000006           1         4           munmap
  3.45    0.000004           1         4           getdents64
  2.59    0.000003           0         4           read
  0.86    0.000001           1         1           getcwd
  0.86    0.000001           0         4           fcntl
  0.86    0.000001           0         4           close
  0.86    0.000001           1         1           ppoll
  0.86    0.000001           0         2           fstat
  0.86    0.000001           0         2           sigaltstack
  0.86    0.000001           0         5           rt_sigaction
  0.86    0.000001           0         2           brk
  0.86    0.000001           1         1           mprotect
  0.86    0.000001           1         1           getrandom
  0.00    0.000000           0         1           set_tid_address
  0.00    0.000000           0         1           rt_sigprocmask
------ ----------- ----------- --------- --------- ----------------
100.00    0.000116           1        75        20 total
```

## perf stat (`nvmr current`, 50 runs)

```
Performance counter stats for '/work/root/nvmr/nvmr current' (50 runs):

              0.07 msec task-clock:u                     #    0.453 CPUs utilized            ( +-  6.35% )
                 0      context-switches:u               #    0.000 /sec
                21      page-faults:u                    #  239.639 K/sec                    ( +-  0.43% )

        0.00015191 +- 0.00000627 seconds time elapsed  ( +-  4.13% )
```

## Peak RSS

| command | max RSS |
|---|---:|
| nvmr current | 660 KB |
| nvmr ls (102) | 660 KB |
| fnm ls | 2328 KB |
| fnm ls (102) | 2328 KB |
