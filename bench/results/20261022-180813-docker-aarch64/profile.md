# Profile

## Binaries

```
-rwxr-xr-x 1 root root 8436728 Mar  5  2026 /work/root/bin/fnm
-rwxr-xr-x 1 root root  725272 Oct 22 16:08 /work/root/fast/nvm-core
-rwxr-xr-x 1 root root 1970544 Oct 22 16:08 /work/root/fast/nvm-fetch
/work/root/fast/nvm-core: ELF 64-bit LSB executable, ARM aarch64, version 1 (SYSV), statically linked, BuildID[sha1]=74c6d66f13f26e0317298c9087fb8628e13559f7, stripped
not a dynamic executable
```

## Processes and syscalls (strace -f, bash)

| case | execve | forks | syscalls |
|---|---:|---:|---:|
| nvm: source + use | 20 | 90 | 15771 |
| nvm: source + ls (102 versions) | 147 | 212 | 463499 |
| nvm: startup with auto-use | 23 | 88 | 18546 |
| fast: source + use | 2 | 2 | 3151 |
| fast: source + ls (102 versions) | 3 | 4 | 4524 |
| fast: startup with auto-use | 2 | 2 | 3161 |
| fnm: startup (env --use-on-cd) | 2 | 1 | 417 |
| fnm: startup + use | 3 | 2 | 602 |
| bare bash | 1 | 0 | 173 |

Programs executed by upstream `nvm use`: sed 8, awk 6, grep 4, tr 3, uname 3, tail 2, ls 2, bash 1, sort 1, find 1, npm 1, node 1

## `nvm-core __auto use` syscall summary

```
% time     seconds  usecs/call     calls    errors syscall
------ ----------- ----------- --------- --------- ----------------
  0.00    0.000000           0         1           getcwd
  0.00    0.000000           0         4           fcntl
  0.00    0.000000           0        10         6 openat
  0.00    0.000000           0         4           close
  0.00    0.000000           0         4           getdents64
  0.00    0.000000           0         4           read
  0.00    0.000000           0         1           ppoll
  0.00    0.000000           0        18        14 newfstatat
  0.00    0.000000           0         2           fstat
  0.00    0.000000           0         1           set_tid_address
  0.00    0.000000           0         2           sigaltstack
  0.00    0.000000           0         5           rt_sigaction
  0.00    0.000000           0         1           rt_sigprocmask
  0.00    0.000000           0         2           brk
  0.00    0.000000           0         4           munmap
  0.00    0.000000           0         1           execve
  0.00    0.000000           0         9           mmap
  0.00    0.000000           0         1           mprotect
  0.00    0.000000           0         1           getrandom
------ ----------- ----------- --------- --------- ----------------
100.00    0.000000           0        75        20 total
```

## perf stat (`nvm-core current`, 50 runs)

```
Performance counter stats for '/work/root/fast/nvm-core current' (50 runs):

              0.11 msec task-clock:u                     #    0.398 CPUs utilized            ( +-  9.32% )
                 0      context-switches:u               #    0.000 /sec                   
                20      page-faults:u                    #  131.577 K/sec                    ( +-  0.52% )

         0.0002686 +- 0.0000193 seconds time elapsed  ( +-  7.20% )
```

## Peak RSS

| command | max RSS |
|---|---:|
| nvm-core current | 532 KB |
| nvm-core ls (102) | 532 KB |
| fnm ls | 2328 KB |
| fnm ls (102) | 2328 KB |

