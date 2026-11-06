# Profile

## Binaries

```
-rwxr-xr-x 1 root root 8436728 Mar  5  2026 /work/root/bin/fnm
-rwxr-xr-x 1 root root  790808 Nov  6 12:49 /work/root/nvmr/nvmr
-rwxr-xr-x 1 root root 1970584 Nov  6 12:49 /work/root/nvmr/nvmr-fetch
/work/root/nvmr/nvmr: ELF 64-bit LSB executable, ARM aarch64, version 1 (SYSV), statically linked, BuildID[sha1]=61b1baddd4568d04c944c919e05b31d0defabb6a, stripped
not a dynamic executable
```

## Processes and syscalls (strace -f, bash)

| case | execve | forks | syscalls |
|---|---:|---:|---:|
| nvm: source + use | 21 | 65 | 15664 |
| nvm: source + ls (102 versions) | 154 | 195 | 462845 |
| nvm: startup with auto-use | 23 | 90 | 18260 |
| nvmr: source + use | 2 | 1 | 5063 |
| nvmr: source + ls (102 versions) | 3 | 2 | 3358 |
| nvmr: startup with auto-use | 2 | 1 | 5079 |
| fnm: startup (env --use-on-cd) | 2 | 1 | 417 |
| fnm: startup + use | 3 | 2 | 605 |
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

              0.07 msec task-clock:u                     #    0.513 CPUs utilized            ( +-  3.69% )
                 0      context-switches:u               #    0.000 /sec
                20      page-faults:u                    #  236.995 K/sec                    ( +-  0.48% )

        0.00013756 +- 0.00000516 seconds time elapsed  ( +-  3.75% )
```

## Peak RSS

| command | max RSS |
|---|---:|
| nvmr current | 660 KB |
| nvmr ls (102) | 660 KB |
| fnm ls | 2324 KB |
| fnm ls (102) | 2328 KB |
