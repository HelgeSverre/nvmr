# Profile

## Binaries

```
-rwxr-xr-x 1 root root 8436728 Mar  5  2026 /work/root/bin/fnm
-rwxr-xr-x 1 root root  790808 Nov  6 14:13 /work/root/nvmr/nvmr
-rwxr-xr-x 1 root root 1970584 Nov  6 14:13 /work/root/nvmr/nvmr-fetch
/work/root/nvmr/nvmr: ELF 64-bit LSB executable, ARM aarch64, version 1 (SYSV), statically linked, BuildID[sha1]=ca6df5c363c9f6ddc7e95b6bbb5e92846c7f0dd1, stripped
not a dynamic executable
```

## Processes and syscalls (strace -f, bash)

| case | execve | forks | syscalls |
|---|---:|---:|---:|
| nvm: source + use | 20 | 80 | 15551 |
| nvm: source + ls (102 versions) | 138 | 230 | 463778 |
| nvm: startup with auto-use | 25 | 100 | 18273 |
| nvmr: source + use | 2 | 1 | 2224 |
| nvmr: source + ls (102 versions) | 3 | 2 | 3715 |
| nvmr: startup with auto-use | 2 | 1 | 2234 |
| fnm: startup (env --use-on-cd) | 2 | 1 | 414 |
| fnm: startup + use | 3 | 2 | 601 |
| bare bash | 1 | 0 | 173 |

Programs executed by upstream `nvm use`: sed 8, awk 6, grep 4, tr 3, uname 3, tail 2, ls 2, bash 1, sort 1, find 1, npm 1, node 1

## `nvmr __auto use` syscall summary

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

## perf stat (`nvmr current`, 50 runs)

```
Performance counter stats for '/work/root/nvmr/nvmr current' (50 runs):

              0.10 msec task-clock:u                     #    0.550 CPUs utilized            ( +-  5.34% )
                 0      context-switches:u               #    0.000 /sec
                19      page-faults:u                    #  177.523 K/sec                    ( +-  0.53% )

        0.00017788 +- 0.00000689 seconds time elapsed  ( +-  3.87% )
```

## Peak RSS

| command | max RSS |
|---|---:|
| nvmr current | 660 KB |
| nvmr ls (102) | 660 KB |
| fnm ls | 2324 KB |
| fnm ls (102) | 2328 KB |
