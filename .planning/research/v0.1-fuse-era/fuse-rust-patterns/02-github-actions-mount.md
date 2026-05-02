# Mounting inside a GitHub Actions runner

## 2. Mounting inside a GitHub Actions runner

### 2.1 Runner reality check

- `ubuntu-latest` **does not preinstall `fuse3` or the `fusermount3` binary**. Confirmed via community discussion ([#26404](https://github.com/orgs/community/discussions/26404)) and runner-images discussion ([#10528](https://github.com/actions/runner-images/discussions/10528)).
- `/dev/fuse` is **available** (the kernel module is loaded on GitHub-hosted runners) but unusable by the runner user until `fusermount3` is installed setuid-root — which `apt install fuse3` handles automatically.
- The runner user (`runner`) is **non-root but has passwordless `sudo`**. We can run `sudo apt install` freely in CI workflows (this is where it differs from our dev host, which lacks sudo — on the dev host the binaries are already present).

### 2.2 Minimum CI install

```yaml
# .github/workflows/ci.yml
- name: Install FUSE runtime
  run: |
    sudo apt-get update
    sudo apt-get install -y fuse3
    # Verify
    which fusermount3
    ls -l /dev/fuse
```

That's it. **Do not install `libfuse3-dev`** — it's unnecessary for us (we don't link libfuse) and pulls in `pkg-config` which fights with our constraint discipline on the dev host. `fuse3` alone gets us:

- `/usr/bin/fusermount3` (setuid-root)
- `/usr/bin/mount.fuse3`
- Appropriate udev rules for `/dev/fuse`

### 2.3 No special permissions needed

The `runner` user is already in the right groups, and `/dev/fuse` is world-readable (mode `0666`) on Ubuntu 22.04/24.04 runners. Once fuse3 is installed, `cargo run -- mount /tmp/mnt` just works without sudo.

One gotcha: if the workflow uses a container (`container:` key), the container needs `--device /dev/fuse --cap-add SYS_ADMIN` (or `--privileged`). Sticking to the host runner is simpler.

### 2.4 Integration test harness skeleton

```yaml
- name: Integration test (FUSE mount)
  timeout-minutes: 5
  run: |
    # Start simulator in background
    cargo run -p reposix-sim -- serve --port 7878 &
    SIM_PID=$!
    sleep 1

    # Mount FUSE in background
    mkdir -p /tmp/reposix-mnt
    cargo run -p reposix-cli -- mount \
        --backend http://127.0.0.1:7878 \
        /tmp/reposix-mnt &
    FUSE_PID=$!

    # Wait for mount to be live (poll `ls`, max 10s)
    for i in {1..20}; do
      if mountpoint -q /tmp/reposix-mnt; then break; fi
      sleep 0.5
    done
    mountpoint -q /tmp/reposix-mnt

    # Exercise it with POSIX tools
    ls /tmp/reposix-mnt
    cat /tmp/reposix-mnt/DEMO-1.md
    echo "status: done" | sed -i ... # writes
    grep -r "bug" /tmp/reposix-mnt

    # Tear down
    fusermount3 -u /tmp/reposix-mnt
    kill $FUSE_PID $SIM_PID || true
```

Key defensive touches:

- `timeout-minutes: 5` — a hung FUSE daemon will pin the runner for 6h default; a tight timeout recovers the minutes budget.
- `mountpoint -q` as the readiness gate (race-free).
- `fusermount3 -u` — **not** `umount` — because only `fusermount3` is setuid-root.
- Always background the mount and kill it explicitly; a panicking daemon leaves a dangling mount that `ls` hangs on.
