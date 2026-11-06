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
| nvm: source + use | 20 | 74 | 15638 |
| nvm: source + ls (102 versions) | 144 | 207 | 463269 |
| nvm: startup with auto-use | 24 | 88 | 18333 |
| nvmr: source + use | 2 | 1 | 332 |
| nvmr: source + ls (102 versions) | 3 | 2 | 1462 |
| nvmr: startup with auto-use | 2 | 1 | 346 |
| fnm: startup (env --use-on-cd) | 2 | 1 | 415 |
| fnm: startup + use | 3 | 2 | 602 |
| bare bash | 1 | 0 | 173 |

Programs executed by upstream `nvm use`: sed 8, awk 6, grep 4, tr 3, uname 3, tail 2, ls 2, bash 1, sort 1, find 1, npm 1, node 1

## `nvmr __auto use` syscall summary

```
% time     seconds  usecs/call     calls    errors syscall
------ ----------- ----------- --------- --------- ----------------
 27.78    0.000020           1        18        14 newfstatat
 20.83    0.000015           1        10         6 openat
 16.67    0.000012           1         9           mmap
  6.94    0.000005           1         4           getdents64
  6.94    0.000005           5         1           rt_sigprocmask
  6.94    0.000005           1         4           munmap
  4.17    0.000003           0         5           rt_sigaction
  2.78    0.000002           1         2           brk
  1.39    0.000001           1         1           getcwd
  1.39    0.000001           0         4           fcntl
  1.39    0.000001           0         4           close
  1.39    0.000001           0         4           read
  1.39    0.000001           1         1           getrandom
  0.00    0.000000           0         1           ppoll
  0.00    0.000000           0         2           fstat
  0.00    0.000000           0         1           set_tid_address
  0.00    0.000000           0         2           sigaltstack
  0.00    0.000000           0         1           execve
  0.00    0.000000           0         1           mprotect
------ ----------- ----------- --------- --------- ----------------
100.00    0.000072           0        75        20 total
```

## perf stat (`nvmr current`, 50 runs)

```
Performance counter stats for '/work/root/nvmr/nvmr current' (50 runs):

              0.08 msec task-clock:u                     #    0.447 CPUs utilized            ( +-  6.67% )
                 0      context-switches:u               #    0.000 /sec
                20      page-faults:u                    #  183.950 K/sec                    ( +-  0.61% )

        0.00018204 +- 0.00000717 seconds time elapsed  ( +-  3.94% )
```

## Peak RSS

| command | max RSS |
|---|---:|
| nvmr current | 660 KB |
| nvmr ls (102) | 660 KB |
| fnm ls | 2328 KB |
| fnm ls (102) | 2328 KB |
