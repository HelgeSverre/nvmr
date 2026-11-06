# Other Node version managers

The benchmark harness now includes [mise](https://mise.jdx.dev/cli/activate.html) and
[proto](https://moonrepo.dev/docs/proto/commands/activate) alongside nvm, nvmr, and fnm. The
release binaries and their checksums are pinned in [`bench/pins.env`](../../bench/pins.env).
All measured tools use the same two Node archives, default version, and shell environment.
Listing cases use 102 installed versions. The Docker container has no network during measurement.

| Tool | Shell setup measured | Project version behavior | Fairly comparable cases |
| --- | --- | --- | --- |
| nvm / nvmr | Source `nvm.sh`; optional identical README auto-switch recipe | `.nvmrc` | startup, `use`, `current`, `ls`, `cd`, Node execution |
| fnm | `eval "$(fnm env --use-on-cd --shell …)"` | `.nvmrc` | startup, `use`, `current`, `ls`, `cd`, Node execution |
| mise | `eval "$(mise activate …)"`; enable documented idiomatic Node version files | `.nvmrc` with that setting | startup, installed listing, `cd`, Node execution |
| proto | `eval "$(proto activate … --no-shim --config-mode all)"` or documented shims | `.prototools`, with `.nvmrc` detection by shims | startup and Node execution |

`proto activate bash` uses a prompt hook. A noninteractive Bash `cd` loop does not exercise its
version switch, so the harness does not label one as a comparable `cd` result. Its activation and
shim configurations are reported separately. Proto's Node and internal schema plugins are
preloaded during the image build; its version check is disabled in the offline runtime.
The direct Node `PATH` configuration is a lower
bound for execution overhead. `mise ls node` and `nvm ls` also have different output, so their
listing timings describe each tool's own installed-list workflow, not byte-for-byte equivalent
output.

[Volta](https://docs.volta.sh/guide/) is another useful design comparator: its per-command shim
and exact `package.json` pins make it valuable for studying Node execution overhead, but it does
not implement nvm's `.nvmrc` and shell `use` workflow. The maintainers currently direct users to
consider mise in [the Volta repository](https://github.com/volta-cli/volta). [`n`](https://github.com/tj/n)
keeps one active Node installation in a prefix, so it is a useful simple-install baseline, but
it cannot represent the same side-by-side version state. These should be labelled separately if
added to the harness.

## Results

The [pinned Docker run](../../bench/results/20261106-161047-docker-aarch64/report.md)
([metadata](../../bench/results/20261106-161047-docker-aarch64/meta.json),
[profile](../../bench/results/20261106-161047-docker-aarch64/profile.md)) used three seeded,
shuffled rounds and 300 runs per short case. Its one-minute load stayed below the harness
threshold. Many individual cases had more than 10% spread between rounds, so the table below
uses **minimum observed per-operation times**, in milliseconds, and does not treat close values
as a ranking. The full report includes every run count, median, MAD, and round spread.

| Bash operation | nvmr | fnm | mise | proto activation | proto shim |
| --- | ---: | ---: | ---: | ---: | ---: |
| Shell startup, configured mode | 1.08 | 0.46 | 5.46 | 28.68 | ≈0¹ |
| List 102 installed versions | 0.43 | 0.42 | 4.33 | — | — |
| Switch on `cd` into `.nvmrc` projects | 0.44 | 0.35 | 4.06 | — | — |
| `cd` with no version file | 0.08 | 0.01 | 3.77 | — | — |
| Execute `node -e 0` | 9.38 | 9.07 | 9.76 | 10.27 | 36.40 |

¹ The proto shim setup only prepends its shim directory, so it has almost no shell-start cost.
Its cost appears when Node runs. The direct Node baseline for that execution row was 8.86 ms.
Nvmr without its auto-switch hook started in 0.63 ms minimum; the table uses its native hook
because the fnm and mise configurations also switch on directory changes.
Mise's `cd` cases crossed the harness's slow-case threshold and therefore received 16–19 runs
in total rather than 900; their full run counts are in the report.

Nvmr and fnm are close on listing. Startup with the directory hook favors fnm in this run, but
nvmr's startup varied substantially between rounds, so its typical gap is unresolved. Fnm's no-change directory hook
has a clear lower minimum; nvmr still checks `.nvmrc` ancestry and default reversion. Mise's
activation, listing, and directory hook are substantially slower in this workload. Proto's
activation has a large startup cost, while its shim makes each short Node invocation costly.
These are specific Docker workflows, not an overall quality ranking. A faster nvmr no-change
hook would need to preserve parent `.nvmrc` detection and changes made to default aliases or
installed versions between directory changes.
