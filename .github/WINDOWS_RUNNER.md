# On-demand Windows runner

This repo uses the private `snapetech/packer` Windows QEMU runner for the
`windows-on-demand` CI job.

Jobs that set:

```yaml
runs-on: [self-hosted, Windows, X64, packer-windows]
```

are picked up by the dispatcher on `kspld0`, which starts a disposable Windows
VM overlay, registers one ephemeral GitHub runner, runs the job, and powers the
VM down.
